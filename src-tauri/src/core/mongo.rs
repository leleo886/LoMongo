use super::ssh_mongo::*;
use crate::{
    error::Error,
    utils::{crypt::*, q2b::*},
    MONGO_DATA_FILE,
};
use futures_util::stream::TryStreamExt;
use json5;
use log::info;
use mongodb::{
    bson::{self, doc, to_document, Document},
    options::{ClientOptions, FindOptions},
    Client, Database,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, State};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct MongoData {
    id: String,
    connect_type: String,
    mongo_auth_method: String,
    ssh_host: String,
    ssh_port: u16,
    ssh_username: String,
    mongo_host: String,
    mongo_port: u16,
    mongo_username: String,
    db_name: String,
}

// 查询数据
#[derive(Serialize, Deserialize)]
pub struct PaginatedResult {
    documents: Vec<Value>,
    total_count: u64,
    page: u32,
    page_size: u32,
    total_pages: u64,
}

// 连接池
#[derive(Default)]
pub struct MongoConnections {
    // 用HashMap存储不同id对应的连接，并使用读写锁保护数据结构
    connections: RwLock<HashMap<String, (Client, Database, Option<SshTunnelManager>)>>,
}

impl MongoData {
    pub fn new(
        id: String,
        connect_type: String,
        mongo_auth_method: String,
        ssh_host: String,
        ssh_port: u16,
        ssh_username: String,
        mongo_host: String,
        mongo_port: u16,
        mongo_username: String,
        db_name: String,
    ) -> Self {
        MongoData {
            id,
            connect_type,
            mongo_auth_method,
            ssh_host,
            ssh_port,
            ssh_username,
            mongo_host,
            mongo_port,
            mongo_username,
            db_name,
        }
    }
}

pub fn format_uri(mongo_data: &MongoData, app_handle: AppHandle) -> anyhow::Result<String> {
    if mongo_data.mongo_auth_method == "userpass" {
        let decrypt_server_str = decrypt(&mongo_data.id, app_handle, MONGO_DATA_FILE)?;
        let decrypt_server: Value = serde_json::from_str(&decrypt_server_str)?;
        let mongo_password = decrypt_server["mongoPassword"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("mongoPassword is empty"))?;

        return Ok(format!(
            "mongodb://{}:{}@{}:{}/{}",
            mongo_data.mongo_username,
            mongo_password,
            mongo_data.mongo_host,
            mongo_data.mongo_port,
            mongo_data.db_name
        ));
    }
    Ok(format!(
        "mongodb://{}:{}/{}",
        mongo_data.mongo_host, mongo_data.mongo_port, mongo_data.db_name
    ))
}

async fn get_mongodb_connection<'a>(
    // 添加显式生命周期标注
    mongo_data: &MongoData,
    uri: &str,
    connections: &'a MongoConnections,
    app_handle: AppHandle,
) -> Result<(Client, Database), Error> {
    // 先尝试读取已存在的连接
    let connections_read = connections.connections.read().await;
    if let Some((client, db, _ssh_tunnel_manager)) = connections_read.get(&mongo_data.id) {
        return Ok((client.clone(), db.clone()));
    }

    // 释放读锁，准备获取写锁
    drop(connections_read);

    let (client, db, ssh_tunnel_manager) = if mongo_data.connect_type == "ssh" {
        let decrypt_server_str = decrypt(&mongo_data.id, app_handle.clone(), MONGO_DATA_FILE)?;
        let decrypt_server: Value = serde_json::from_str(&decrypt_server_str)?;
        let mongo_password = if mongo_data.mongo_auth_method == "userpass" {
            decrypt_server["mongoPassword"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("mongoPassword is empty"))?
        } else {
            ""
        };

        let ssh_password = decrypt_server["sshPassword"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("sshPassword is empty"))?;

        ssh_tunnel_to_mongodb(
            &mongo_data.id,
            &mongo_data.mongo_auth_method,
            &mongo_data.ssh_host,
            mongo_data.ssh_port,
            &mongo_data.ssh_username,
            &ssh_password,
            // &format!("{}:{}", mongo_data.mongo_host, 27018),
            &format!("{}:{}", mongo_data.mongo_host, mongo_data.mongo_port),
            &mongo_data.mongo_username,
            &mongo_password,
            &mongo_data.db_name,
            app_handle,
        )
        .await?
    } else {
        // 创建新连接
        let mut client_options = ClientOptions::parse(uri).await?;
        client_options.server_selection_timeout = Some(std::time::Duration::from_secs(15)); // 服务器连接超时时间
        let dclient = Client::with_options(client_options)?;
        let ddb = dclient.database(&mongo_data.db_name);
        (dclient, ddb, None)
    };

    // 将新连接存入连接池
    let mut connections_write = connections.connections.write().await;
    connections_write.insert(
        mongo_data.id.to_string(),
        (client.clone(), db.clone(), ssh_tunnel_manager),
    );

    Ok((client, db))
}

pub async fn connect_server(
    mongo_data: &MongoData,
    connections: State<'_, MongoConnections>, // 显式指定生命周期
    app_handle: AppHandle,
) -> Result<Vec<Document>, Error> {
    let uri = format_uri(mongo_data, app_handle.clone())?;

    // 从连接池获取或创建连接
    let (_client, db) = get_mongodb_connection(mongo_data, &uri, &connections, app_handle).await?;

    let collections = db.list_collection_names().await?;

    let mut all_stats = Vec::new();

    for collection_name in collections {
        // 获取信息
        let stats = db
            .run_command(doc! {
                "collStats": &collection_name,
                "scale": 1,
            })
            .await?;

        // 提取需要的字段
        let mut coll_stats = Document::new();
        coll_stats.insert("collection", collection_name);

        if let Ok(storage_size) = stats.get_i32("storageSize") {
            coll_stats.insert("storageSize", storage_size);
        }

        if let Ok(count) = stats.get_i32("count") {
            coll_stats.insert("count", count);
        }

        if let Ok(nindexes) = stats.get_i32("nindexes") {
            coll_stats.insert("indexCount", nindexes);
        }

        if let Ok(index_sizes) = stats.get_document("indexSizes") {
            let index_names: Vec<_> = index_sizes.keys().collect();
            coll_stats.insert("indexes", index_names);
        }

        all_stats.push(coll_stats);
    }

    Ok(all_stats)
}

pub async fn mongodb_collection(
    mongo_data: &MongoData,
    collection_name: String,
    page: Option<u32>,
    page_size: Option<u32>,
    query: String,
    connections: State<'_, MongoConnections>,
    app_handle: AppHandle,
) -> Result<PaginatedResult, Error> {
    let uri = format_uri(mongo_data, app_handle.clone())?;

    let (_client, db) = get_mongodb_connection(mongo_data, &uri, &connections, app_handle).await?;
    let collection: mongodb::Collection<Document> = db.collection(&collection_name);

    // 解析查询条件，使用json5将不规范的json格式解析为严格规范的serde_json::Value再转为Document
    let filter: Document = if !query.is_empty() {
        let json: Value = json5::from_str(&preprocess_query(&query))?;
        info!("Parsed: {:?}", json);
        match to_document(&json) {
            Ok(query) => query,
            Err(_) => {
                doc! {}
            }
        }
    } else {
        doc! {}
    };

    // 字段类型转换
    let convert_query = convert_document(filter);
    info!("Converted: {:?}", convert_query);
    // 获取总文档数 - 使用过滤条件
    let total_count = collection.count_documents(convert_query.clone()).await?;

    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(25);
    let skip = page * page_size;
    let total_pages = (total_count as f64 / page_size as f64).ceil() as u64;

    // 查询当前页数据 - 使用过滤条件
    let options = FindOptions::builder()
        .skip(Some(skip as u64))
        .limit(Some(page_size as i64))
        .build();

    let mut cursor = collection.find(convert_query).with_options(options).await?;

    let mut documents = Vec::new();
    while let Some(doc) = cursor.try_next().await? {
        let bson: Value = bson::to_bson(&doc)?.into();
        documents.push(bson);
    }

    Ok(PaginatedResult {
        documents,
        total_count,
        page,
        page_size,
        total_pages,
    })
}

pub async fn clear_connection(
    id: String,
    connections: State<'_, MongoConnections>, //manage自动注入，'_匿名生命周期自动推断
) -> Result<(), Error> {
    let mut connections_write = connections.connections.write().await;

    //利用remove() 从 HashMap 中移除整个条目，可以拥有整个 collecion 的所有权，包括 ssh_tunnel_manage
    if let Some(collecion) = connections_write.remove(&id) {
        if let Some(ssh_tunnel_manage) = collecion.2 {
            ssh_tunnel_manage.stop().await;
        }
    }
    Ok(())
}


#[tokio::test]
async fn test_mongodb() {
	let uri = "mongodb://url";
	let mut client_options = ClientOptions::parse(uri).await.unwrap();
	client_options.server_selection_timeout = Some(std::time::Duration::from_secs(10)); // 服务器连接超时时间
	let client = Client::with_options(client_options).unwrap();
	let db = client.database("sample_mflix");
	let collections = db.list_collection_names().await.unwrap();
	println!("collections: {:?}", collections);
}