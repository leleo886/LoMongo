use anyhow::Context;
use serde_json::{from_str, json, to_writer_pretty, Value};
use std::fs::{self, File};
use tauri::AppHandle;
use tauri::Manager;

pub fn store_value(new_data: Value, app_handle: AppHandle, filename: &str) -> anyhow::Result<()> {
    let data_path = app_handle.path().app_data_dir()?.join(filename);
    let combined_data: Value = if fs::metadata(&data_path).is_ok() {
        // 文件存在：读取并合并数据
        let content = fs::read_to_string(&data_path).context("Invalid data")?;
        let mut existing: Value = from_str(&content).context("Invalid data")?;

        // 合并数据
        if let (Value::Object(existing_map), Value::Object(new_map)) = (&mut existing, new_data) {
            for (k, v) in new_map {
                existing_map.insert(k, v);
            }
        }
        existing
    } else {
        // 文件不存在：直接使用新数据
        new_data
    };

    let file = File::create(&data_path).context("Failed to create data file")?;
    to_writer_pretty(file, &combined_data).context("Failed to write data")?;

    Ok(())
}

pub fn get_value(skey: &str, app_handle: AppHandle, filename: &str) -> anyhow::Result<String> {
    let data_path = app_handle.path().app_data_dir()?.join(filename);

    // 如果文件不存在，创建默认格式文件
    if !fs::metadata(&data_path).is_ok() {
        let default_data = json!({});
        let file = File::create(&data_path).context("Failed to create data file")?;
        to_writer_pretty(file, &default_data).context("Failed to write default JSON")?;
    }

    // 文件存在，尝试读取和解析
    let content = fs::read_to_string(&data_path).context("Failed to read data file")?;

    // 处理空文件或无效 JSON 的情况
    if content.trim().is_empty() {
        // 文件为空，使用默认值重建
        let default_data = json!({});
        let file = File::create(&data_path).context("Failed to recreate data file")?;
        to_writer_pretty(file, &default_data).context("Failed to write default JSON")?;
    }

    let data: Value = if fs::metadata(&data_path).is_ok() {
        let content = fs::read_to_string(&data_path).context("Invalid data")?;
        let existing: Value = from_str(&content).context("Invalid data")?;
        existing
    } else {
        json!({})
    };
    match data.get(skey) {
        Some(Value::String(s)) => Ok(s.to_string()),
        _ => Ok(String::new()),
    }
}

pub fn delete_value(skey: &str, app_handle: AppHandle, filename: &str) -> anyhow::Result<()> {
    let data_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("failed to delete: {}", e))?
        .join(filename);

    let content = fs::read_to_string(&data_path).context("Failed to read data")?;
    let mut data: Value = from_str(&content).context("Invalid data")?;

    //如果是 Object 类型，删除字段
    if let Value::Object(ref mut map) = data {
        map.remove(skey);
    }

    fs::write(data_path, data.to_string()).context("Failed to write data")?;

    Ok(())
}
