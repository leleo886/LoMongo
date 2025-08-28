pub mod api;
pub mod core;
pub mod error;
pub mod utils;
use api::api_mongo::{
    mongo_clear_connection, mongo_collection, mongo_connect_server, mongo_data_encrypt,
    mongo_delete_encrypt_data,
};
use core::mongo::MongoConnections;

pub static MONGO_DATA_FILE: &str = "data.json";
pub static SSH_KEY_FILE: &str = "key.json";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
			.target(tauri_plugin_log::Target::new(
				tauri_plugin_log::TargetKind::LogDir {
				file_name: Some("logs".to_string()),
				},
			)).build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        // 将连接池注册为全局状态
        .manage(MongoConnections::default())
        .invoke_handler(tauri::generate_handler![
            mongo_connect_server,
            mongo_collection,
            mongo_clear_connection,
            mongo_data_encrypt,
            mongo_delete_encrypt_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
