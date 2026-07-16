use crate::models::collect::{CollectConfig, CollectRunResult};
use crate::services::{collect_folders_service, collect_service};

#[tauri::command]
pub fn collect_load_ini() -> Result<CollectConfig, String> {
    collect_service::load_ini()
}

#[tauri::command]
pub fn collect_run(config: CollectConfig) -> Result<CollectRunResult, String> {
    collect_service::run(config)
}

#[tauri::command]
pub async fn collect_by_folders(config: CollectConfig) -> Result<CollectRunResult, String> {
    tauri::async_runtime::spawn_blocking(move || collect_folders_service::run(config))
        .await
        .map_err(|e| format!("Lỗi thread nền: {e}"))?
}
