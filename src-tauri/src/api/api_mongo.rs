use crate::{
    core::mongo::{
        clear_connection, connect_server, mongodb_collection, MongoConnections, MongoData,
        PaginatedResult,
    },
    error::Error,
    utils::{crypt::*, io_op::*},
    MONGO_DATA_FILE, SSH_KEY_FILE,
};
use mongodb::bson::Document;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn mongo_connect_server(
    id: String,
    r#type: String,
    auth_method: String,
    ssh_host: String,
    ssh_port: u16,
    ssh_username: String,
    mongo_host: String,
    mongo_port: u16,
    mongo_username: String,
    db_name: String,
    app_handle: AppHandle,
    connections: State<'_, MongoConnections>, //manage自动注入，'_匿名生命周期自动推断
) -> Result<Vec<Document>, Error> {
    return connect_server(
        &MongoData::new(
            id,
            r#type,
            auth_method,
            ssh_host,
            ssh_port,
            ssh_username,
            mongo_host,
            mongo_port,
            mongo_username,
            db_name,
        ),
        connections,
        app_handle,
    )
    .await;
}

#[tauri::command]
pub async fn mongo_collection(
    id: String,
    r#type: String,
    auth_method: String,
    ssh_host: String,
    ssh_port: u16,
    ssh_username: String,
    mongo_host: String,
    mongo_port: u16,
    mongo_username: String,
    db_name: String,
    collection_name: String,
    page: Option<u32>,
    page_size: Option<u32>,
    query: String,
    app_handle: AppHandle,
    connections: State<'_, MongoConnections>,
) -> Result<PaginatedResult, Error> {
    return mongodb_collection(
        &MongoData::new(
            id,
            r#type,
            auth_method,
            ssh_host,
            ssh_port,
            ssh_username,
            mongo_host,
            mongo_port,
            mongo_username,
            db_name,
        ),
        collection_name,
        page,
        page_size,
        query,
        connections,
        app_handle,
    )
    .await;
}

#[tauri::command]
pub async fn mongo_clear_connection(
    id: String,
    connections: State<'_, MongoConnections>,
) -> Result<(), Error> {
    return clear_connection(id, connections).await;
}

#[tauri::command]
pub async fn mongo_data_encrypt(
    password: String,
    plaintext: String,
    app_handle: AppHandle,
) -> Result<(), Error> {
    encrypt(password, plaintext, app_handle, MONGO_DATA_FILE)?;
    Ok(())
}

#[tauri::command]
pub async fn mongo_delete_encrypt_data(skey: &str, app_handle: AppHandle) -> Result<(), Error> {
    delete_value(skey, app_handle.clone(), MONGO_DATA_FILE)?;
    delete_value(skey, app_handle, SSH_KEY_FILE)?;
    Ok(())
}
