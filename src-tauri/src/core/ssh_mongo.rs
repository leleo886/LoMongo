use crate::SSH_KEY_FILE;
use crate::utils::crypt::decrypt;
use super::server_key::key_check;
use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use log::{error, info, warn, debug};
use mongodb::{bson, options::ClientOptions, Client as MongoClient, Database};
use std::net::SocketAddr;
use tauri::AppHandle;
use tokio::io::{copy_bidirectional, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::{JoinHandle, AbortHandle};
use tokio::time::{sleep, Duration, timeout};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::sync::Mutex;


// 本地应用 → 本地端口 → SSH隧道循环 → SSH连接 → 远程MongoDB
//     │          │           │           │
//     │          │           ├─ 接受新连接 → 创建转发任务
//     │          │           │
//     │          │           └─ 监听终止信号 → 清理所有连接
//     │          │
//     └─ 通信保持 ←─ 独立转发任务处理每个连接


// SSH隧道生命周期结构体
pub struct SshTunnelManager {
    tunnel_handle: AbortHandle,      // 隧道任务中止句柄
    shutdown_tx: mpsc::Sender<()>,   // 终止信号发送器
    active_connections: Arc<Mutex<Vec<JoinHandle<()>>>>, // 活动连接列表
}

impl SshTunnelManager {
    pub async fn stop(&self) {
        // 发送终止信号
        let _ = self.shutdown_tx.send(()).await;
        
        sleep(Duration::from_millis(100)).await;
        
        // 中止所有活动连接
        let mut connections = self.active_connections.lock().await;
        for handle in connections.drain(..) {
            handle.abort();
        }
        
        // 最后中止隧道任务
        self.tunnel_handle.abort();
    }
}

//实现 `Drop` trait，确保即使没有显式调用 `stop`，也会在管理器被丢弃时中止隧道任务，防止资源泄漏。
impl Drop for SshTunnelManager {
    fn drop(&mut self) {
        // 确保资源被清理
        if !self.tunnel_handle.is_finished() {
            warn!("SshTunnelManager is discarded but the tunnel is still running, aborting.");
            self.tunnel_handle.abort();
        }
    }
}

/// 启动SSH隧道
/// 1. 建立SSH连接到跳板机
/// 2. 绑定本地端口
/// 3. 循环接受本地连接并转发到远程MongoDB
/// 4. 监听终止信号，清理资源
async fn start_ssh_tunnel(
    ssh_host: String,
    ssh_port: u16,
    ssh_user: String,
    auth_method: AuthMethod,
    server_check_method: ServerCheckMethod,
    remote_mongo_addr: SocketAddr,
    mut shutdown_rx: mpsc::Receiver<()>,
    success_tx: mpsc::Sender<SocketAddr>, // 成功时发送实际绑定的地址
    active_connections: Arc<Mutex<Vec<JoinHandle<()>>>>,
) -> Result<()> {
    // 建立 SSH 连接到跳板机
    let client = match Client::connect(
        (ssh_host.as_str(), ssh_port),
        ssh_user.as_str(),
        auth_method,
        server_check_method,
    )
    .await
    {
        Ok(client) => {
            info!("SSH has been established: {}@{}:{}", ssh_user, ssh_host, ssh_port);
            client
        }
        Err(e) => {
            return Err(anyhow!("SSH failed to connect: {}", e));
        }
    };

    // 本地监听端口（重试逻辑）
    let mut current_port = 27017; // 默认MongoDB端口
    let max_retries = 20; // 最多重试20次
    let mut listener = None;

    for i in 0..=max_retries {
        let addr = SocketAddr::new("127.0.0.1".parse().unwrap(), current_port);
        match TcpListener::bind(addr).await {
            Ok(l) => {
                listener = Some((l, addr));
                info!("local port forwarding has been started: {} -> {}", addr, remote_mongo_addr);
                break;
            }
            Err(e) if i < max_retries => {
                warn!(
                    "port {} is ocupied ({}), trying next port: {}",
                    current_port,
                    e,
                    current_port + 1
                );
                current_port += 1;
            }
            Err(e) => {
                return Err(anyhow!(
                    "try {} times but cant't bind port: {}",
                    max_retries + 1,
                    e
                ));
            }
        }
    }

    let (listener, local_bind_addr) = listener.expect("can't bind local port");
    
    // 发送成功信号，包含实际绑定的地址
    if let Err(e) = success_tx.send(local_bind_addr).await {
        error!("failed to send success signal: {}", e);
        return Err(anyhow!("failed to send success signal: {}", e));
    }

    // 循环接受本地连接，同时监听终止信号
    loop {
        tokio::select! {
            // 等待接受新的本地连接
            accept_result = listener.accept() => {
                let (mut local_stream, _) = match accept_result {
                    Ok(val) => val,
                    Err(e) => {
                        error!("accept local connection failed: {}", e);
                        continue;
                    }
                };
                
                debug!("new local connection: {:?}", local_stream.peer_addr());

				// 通过SSH创建到远程MongoDB的通道
                let ssh_channel = match client
                    .open_direct_tcpip_channel(remote_mongo_addr, None)
                    .await
                {
                    Ok(channel) => channel,
                    Err(e) => {
                        error!("failed to create ssh channel: {}", e);
                        continue;
                    }
                };

                let mut ssh_stream = ssh_channel.into_stream();
                
                // y异步创建连接任务
                let handle = tokio::spawn(async move {
					// 在两者之间双向通信复制数据
                     let _ = copy_bidirectional(&mut local_stream, &mut ssh_stream).await;
                    
                    // 确保连接被正确关闭
                    let _ = local_stream.shutdown().await;
                    let _ = ssh_stream.shutdown().await;
                });
                
                // 存储连接句柄
                active_connections.lock().await.push(handle);
            }
            // 收到终止信号，退出循环
            _ = shutdown_rx.recv() => {
                info!("received shutdown signal, closing ssh tunnel");
                // 关闭所有活动连接
                let mut connections = active_connections.lock().await;
                for handle in connections.drain(..) {
                    handle.abort();
                }
                return Ok(());
            }
        }
    }
}

/// 通过SSH隧道连接到MongoDB
/// 1. 解密或获取SSH密钥
/// 2. 启动SSH隧道
/// 3. 构建MongoDB连接字符串并测试连接
/// 4. 返回MongoDB客户端、数据库和隧道管理器
pub async fn ssh_tunnel_to_mongodb(
    id: &str,
    mongo_auth_method: &str,
    ssh_host: &str,
    ssh_port: u16,
    ssh_user: &str,
    ssh_password: &str,
    remote_mongo: &str,
    mongo_user: &str,
    mongo_password: &str,
    db_name: &str,
    app_handle: AppHandle,
) -> Result<(MongoClient, Database, Option<SshTunnelManager>)> {
    let public_key = match decrypt(id, app_handle.clone(), SSH_KEY_FILE) {
        Ok(decrypt_key) if !decrypt_key.is_empty() => {
            // 直接使用存储的密钥
            if let Some(key) = decrypt_key.split_whitespace().nth(1) {
                key.to_string()
            } else {
                return Err(anyhow!("SSH key format error".to_string()));
            }
        }
        _ => {
            // 首次连接，需要获取并验证密钥
            let key = key_check(
                id,
                ssh_host,
                ssh_port,
                ssh_user,
                ssh_password,
                app_handle.clone(),
            )
            .await?;

			// 第一次连接密钥保存后返回为空，图省事以特定代号告诉前台不是错误，重新连接即可
            if key.is_empty() {
                return Err(anyhow::anyhow!("Code-3581"));
            }
            key
        }
    };

    let remote_mongo_addr: SocketAddr = remote_mongo
        .parse()
        .map_err(|e| anyhow!("remote MongoDB address parse failed: {}", e))?;

    // 创建认证方法
    let auth_method = AuthMethod::with_password(ssh_password);
    let server_check_method = ServerCheckMethod::with_public_key(&public_key);

    // 创建信号通道
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let (error_tx, mut error_rx) = mpsc::channel(1);
    let (success_tx, mut success_rx) = mpsc::channel(1);
    
    // 活动连接列表
    let active_connections = Arc::new(Mutex::new(Vec::new()));

    // 在后台启动SSH隧道
    let tunnel_handle = tokio::spawn({
        let ssh_host = ssh_host.to_string();
        let ssh_user = ssh_user.to_string();
        let active_connections = Arc::clone(&active_connections);
        
        async move {
            if let Err(e) = start_ssh_tunnel(
                ssh_host,
                ssh_port,
                ssh_user,
                auth_method,
                server_check_method,
                remote_mongo_addr,
                shutdown_rx,
                success_tx,
                active_connections,
            )
            .await
            {
                error!("SSH tunnel start failed: {}", e);
                let _ = error_tx.send(e).await;
            }
        }
    }).abort_handle();

    // 等待隧道启动成功或失败，设置20秒超时
    let local_bind_addr = match timeout(Duration::from_secs(20), async {
        tokio::select! {
            Some(addr) = success_rx.recv() => Ok(addr),
            Some(e) = error_rx.recv() => Err(e),
        }
    })
    .await
    {
        Ok(Ok(addr)) => addr,
        Ok(Err(e)) => {
            tunnel_handle.abort();
            return Err(e);
        }
        Err(_) => {
            tunnel_handle.abort();
            return Err(anyhow!("SSH tunnel start timeout".to_string()));
        }
    };

    info!("SSH tunnel has been started, local address: {}", local_bind_addr);

    // 构建MongoDB连接字符串
    let mongo_uri = if mongo_auth_method != "userpass" {
        format!("mongodb://{}/{}", local_bind_addr, db_name)
    } else {
        format!(
            "mongodb://{}:{}@{}/{}",
            mongo_user, mongo_password, local_bind_addr, db_name
        )
    };

    // 连接MongoDB
    let client_options = ClientOptions::parse(&mongo_uri)
        .await
        .map_err(|e| anyhow!("MongoDB URI parse failed: {}", e))?;
        
    let client = MongoClient::with_options(client_options)
        .map_err(|e| anyhow!("MongoDB client create failed: {}", e))?;
        
    // 测试连接
    client
        .database("admin")
        .run_command(bson::doc! {"ping": 1})
        .await
        .map_err(|e| anyhow!("MongoDB connect test failed: {}", e))?;

    let db = client.database(db_name);
    
    // 创建隧道管理器
    let tunnel_manager = SshTunnelManager {
        tunnel_handle,
        shutdown_tx,
        active_connections,
    };

    Ok((client, db, Some(tunnel_manager)))
}