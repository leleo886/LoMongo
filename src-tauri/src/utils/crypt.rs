use super::io_op;
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit};
use argon2::Argon2;
use hex;
use log::{error, warn};
use rand_core::{OsRng, TryRngCore};
use serde_json::json;
use tauri::AppHandle;

// 加密函数
pub fn encrypt(
    password: String,
    plaintext: String,
    app_handle: AppHandle,
    filename: &str,
) -> anyhow::Result<()> {
    let mut salt = [0u8; 16];
    let _ = OsRng.try_fill_bytes(&mut salt); // 生成盐

    let mut nonce = [0u8; 12];
    let _ = OsRng.try_fill_bytes(&mut nonce); // 生成 nonce

    let mut key = [0u8; 32]; // AES-256 需要 32 字节密钥
    let argon2 = Argon2::default(); // 使用默认 Argon2id 参数
    argon2
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|e| {
            error!("failed to encrypt: {}", e);
            anyhow::anyhow!("failed to encrypt: {}", e)
        })?;

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let ciphertext = cipher
        .encrypt(&nonce.into(), plaintext.as_bytes())
        .map_err(|e| {
            error!("failed to encrypt: {}", e);
            anyhow::anyhow!("failed to encrypt: {}", e)
        })?;

    // 合并盐、nonce 和密文
    let mut combined = Vec::new();
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);
    warn!("encrypted: {:?}", hex::encode(&combined));

    Ok(io_op::store_value(
        json!({password:hex::encode(combined)}),
        app_handle,
        filename,
    )?) // 十六进制编码输出 hex::encode(combined)
}

// 解密函数
pub fn decrypt(skey: &str, app_handle: AppHandle, filename: &str) -> anyhow::Result<String> {
    let encrypted_data = io_op::get_value(skey, app_handle, filename)?;
    if encrypted_data.is_empty() {
        return Ok("".to_string());
    }

    let data = hex::decode(encrypted_data)?;
    let salt: [u8; 16] = data[0..16].try_into()?;
    let nonce: [u8; 12] = data[16..28].try_into()?;
    let ciphertext = &data[28..];

    let mut key = [0u8; 32];
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(skey.as_bytes(), &salt, &mut key)
        .map_err(|e| {
            error!("failed to decrypt: {}", e);
            anyhow::anyhow!("failed to decrypt: {}", e)
        })?;

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let plaintext: Vec<u8> = cipher.decrypt(&nonce.into(), ciphertext).map_err(|e| {
        error!("failed to decrypt: {}", e);
        anyhow::anyhow!("failed to decrypt: {}", e)
    })?;

    Ok(String::from_utf8(plaintext)?)
}
