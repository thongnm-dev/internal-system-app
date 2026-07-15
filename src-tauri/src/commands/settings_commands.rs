//! Tauri command handlers cho module cài đặt ứng dụng.

use crate::models::settings::{AppSettings, SaveSettingsRequest};
use crate::services::settings_service;

#[tauri::command]
pub async fn get_settings(user_id: i32) -> Result<AppSettings, String> {
    settings_service::get_settings(user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(request: SaveSettingsRequest) -> Result<AppSettings, String> {
    settings_service::save_settings(request)
        .await
        .map_err(|e| e.to_string())
}
