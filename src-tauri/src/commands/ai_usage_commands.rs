//! Tauri command handlers cho module AI Usage.

use crate::models::ai_usage::{
    AddAiAccountRequest, AiAccount, AiUsageSettings, ReportUsageSignalRequest,
    UpdateAiAccountRequest,
};
use crate::services::ai_usage_service;

#[tauri::command]
pub async fn ai_usage_add_account(request: AddAiAccountRequest) -> Result<AiAccount, String> {
    ai_usage_service::add_account(request).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_usage_list_accounts() -> Result<Vec<AiAccount>, String> {
    ai_usage_service::list_accounts().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_usage_update_account(
    request: UpdateAiAccountRequest,
) -> Result<AiAccount, String> {
    ai_usage_service::update_account(request).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_usage_delete_account(id: i64) -> Result<(), String> {
    ai_usage_service::delete_account(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_usage_set_active(id: i64) -> Result<(), String> {
    ai_usage_service::set_active(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_usage_get_token(id: i64) -> Result<String, String> {
    ai_usage_service::get_token(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_usage_report_signal(request: ReportUsageSignalRequest) -> Result<(), String> {
    ai_usage_service::report_signal(request).map_err(|e| e.to_string())
}

/// Ép probe usage ngay lập tức (toàn bộ account), trả về danh sách sau khi cập nhật.
#[tauri::command]
pub async fn ai_usage_refresh(app: tauri::AppHandle) -> Result<Vec<AiAccount>, String> {
    ai_usage_service::poll_once(&app)
        .await
        .map_err(|e| e.to_string())?;
    ai_usage_service::list_accounts().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_usage_get_settings() -> Result<AiUsageSettings, String> {
    ai_usage_service::get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_usage_save_settings(settings: AiUsageSettings) -> Result<(), String> {
    ai_usage_service::save_settings(settings).map_err(|e| e.to_string())
}
