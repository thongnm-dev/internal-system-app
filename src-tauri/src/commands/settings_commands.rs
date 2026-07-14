//! Tauri command handlers cho module cài đặt ứng dụng.

use crate::models::settings::{AppSettings, SaveSettingsRequest};
use crate::services::settings_service;
use tauri::Manager;

/// Đọc cài đặt hiện tại từ file JSON.
#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    settings_service::get_settings(&app_data_dir).map_err(|e| e.to_string())
}

/// Validate và lưu cài đặt mới.
#[tauri::command]
pub fn save_settings(
    app: tauri::AppHandle,
    request: SaveSettingsRequest,
) -> Result<AppSettings, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    settings_service::save_settings(&app_data_dir, request).map_err(|e| e.to_string())
}
