use crate::{
    utils::crypt::{decrypt, encrypt},
    SSH_KEY_FILE,
};
use anyhow::Context;
use russh::client::{self, Handler};
use russh::keys::{HashAlg, PublicKey};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener};
use log::{error, info, warn};

#[derive(Default)]
struct KeyCheckResult {
    fingerprint: String,
    algorithm: String,
    openssh: String,
}

// 共享的数据结构，用于存储服务器密钥的检查结果
struct ServerKeyCheck {
    key_data: Arc<Mutex<KeyCheckResult>>,
}

// 事件发送数据结构，用于首次密钥检查时发送给前台
#[derive(serde::Serialize, Clone)]
struct FirstKeyCheckData {
    alg: String,
    fp: String,
}

impl Handler for ServerKeyCheck {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        let alg = server_public_key.algorithm();
        let fp = server_public_key.fingerprint(HashAlg::Sha256);

        // 更新共享状态
        if let Ok(mut result) = self.key_data.lock() {
            result.fingerprint = fp.to_string();
            result.algorithm = alg.to_string();
            result.openssh = server_public_key.to_openssh()?;
        }
        Ok(true)
    }
}

/// 执行服务器密钥检查的主函数
/// - `app_handle`: Tauri 应用句柄
/// 返回：密钥的 OpenSSH 格式或错误信息
pub async fn key_check(
    id: &str,
    host: &str,
    port: u16,
    user: &str,
    pass: &str,
    app_handle: AppHandle,
) -> anyhow::Result<String> {
    let cfg = Arc::new(client::Config::default());

    let check_result = Arc::new(Mutex::new(KeyCheckResult::default()));

    let handler = ServerKeyCheck {
        key_data: check_result.clone(),
    };

    let addr = format!("{host}:{port}");
    let mut handle = client::connect(cfg, &addr, handler).await?;

    match handle.authenticate_password(user, pass).await? {
        client::AuthResult::Success => {
            if let Ok(result) = check_result.lock() {
                let decrypt_key = decrypt(id, app_handle.clone(), SSH_KEY_FILE)?;

                if decrypt_key.is_empty() {
                    warn!(
                        "first check the key of host: {}:{} | algorithm: {} | fingerprint: {}",
                        host, port, result.algorithm, result.fingerprint
                    );

                    let id_clone = id.to_string();
                    let app_handle_clone = app_handle.clone();
                    let openssh = result.openssh.clone();
                    let algorithm = result.algorithm.clone();
                    let fingerprint = result.fingerprint.clone();

                    // emit需要所有权 clone
					// 发送事件，等待前台确认保存密钥
                    app_handle
                        .emit(
                            "FirstKeyCheck",
                            FirstKeyCheckData {
                                alg: algorithm,
                                fp: fingerprint,
                            },
                        )
                        .context("Failed to emit event")?;
					
					// 一次监听前台确认事件
                    app_handle.once("FirstKeyChecked", |event| {
                        if event.payload() == "true" {
                            let _ = encrypt(id_clone, openssh, app_handle_clone, SSH_KEY_FILE)
                                .context("failed to encrypt");
                        }
                    });
                } else {
                    if decrypt_key == result.openssh {
                        info!("authentication checked successfully");
                        if let Some(key) = decrypt_key.split_whitespace().nth(1) {
                            return Ok(key.to_string());
                        } else {
                            error!("error sshkey format");
                            return Err(anyhow::anyhow!("error sshkey format"));
                        }
                    } else {
                        error!("Key verification failed");
                        return Err(anyhow::anyhow!("Code-3067"));
                    }
                }
            }
        }
        client::AuthResult::Failure { .. } => {
            error!("Password verification failed");
            return Err(anyhow::anyhow!("Password verification failed"));
        }
    }
    Ok("".to_string())
}



#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = "127.0.0.1";
    let port = 22;
    let pass = "asdfghjkl";
    let user = "root";

    let cfg = Arc::new(client::Config::default());

    let check_result = Arc::new(Mutex::new(KeyCheckResult::default()));

    let handler = ServerKeyCheck {
        key_data: check_result.clone(), 
    };

    let addr = format!("{host}:{port}");
    let mut handle = client::connect(cfg, &addr, handler).await?;

    match handle.authenticate_password(user, pass).await? {
        client::AuthResult::Success => {
            println!("authentication checked successfully");
            if let Ok(result) = check_result.lock() {
                println!("fingerprint: {}", result.fingerprint);
                println!("algorithm: {}", result.algorithm);
                println!("openssh: {}", result.openssh);
                if let Some(key) = result.openssh.split_whitespace().nth(1) {
                    println!("key: {key}");
                }
            }
        }
        client::AuthResult::Failure { .. } => {
            eprintln!("Password verification failed");
        }
    }
    Ok(())
}
