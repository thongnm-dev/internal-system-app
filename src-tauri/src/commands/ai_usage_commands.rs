//! Tauri command handlers cho module AI Usage.

use crate::models::ai_usage::{
    AddAiAccountRequest, AiAccount, AiUsageSettings, CapturedLogin, DetectedLogin,
    ReportUsageSignalRequest, UpdateAiAccountRequest,
};
use crate::services::ai_usage_service;

#[tauri::command]
pub async fn ai_usage_add_account(request: AddAiAccountRequest) -> Result<AiAccount, String> {
    ai_usage_service::add_account(request).map_err(crate::app::error::log_err)
}

/// Dò các login Claude đã tồn tại trên máy (chưa thêm gì, chỉ trả danh sách).
#[tauri::command]
pub async fn ai_usage_detect_local() -> Result<Vec<DetectedLogin>, String> {
    ai_usage_service::detect_local().map_err(crate::app::error::log_err)
}

/// Dò login local rồi tự thêm những login chưa có; trả về danh sách account mới.
#[tauri::command]
pub async fn ai_usage_import_detected() -> Result<Vec<AiAccount>, String> {
    ai_usage_service::import_detected().map_err(crate::app::error::log_err)
}

/// Xem trước login Claude đang active trên máy (để capture) — không kèm token.
#[tauri::command]
pub async fn ai_usage_capture_preview() -> Result<Option<CapturedLogin>, String> {
    ai_usage_service::capture_preview().map_err(crate::app::error::log_err)
}

/// Capture login Claude đang active → lưu token vào profile riêng + thêm account.
#[tauri::command]
pub async fn ai_usage_capture_add(name: Option<String>) -> Result<AiAccount, String> {
    ai_usage_service::capture_add(name).map_err(crate::app::error::log_err)
}

/// Xem trước login Claude tại một `CLAUDE_CONFIG_DIR` (thêm account thứ 2).
#[tauri::command]
pub async fn ai_usage_config_dir_preview(
    config_dir: String,
) -> Result<Option<CapturedLogin>, String> {
    ai_usage_service::config_dir_preview(config_dir).map_err(crate::app::error::log_err)
}

/// Đăng ký account subscription thứ 2 từ một `CLAUDE_CONFIG_DIR` đã login sẵn.
#[tauri::command]
pub async fn ai_usage_add_config_dir(
    config_dir: String,
    name: Option<String>,
) -> Result<AiAccount, String> {
    ai_usage_service::add_config_dir(config_dir, name).map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_usage_list_accounts() -> Result<Vec<AiAccount>, String> {
    ai_usage_service::list_accounts().map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_usage_update_account(
    request: UpdateAiAccountRequest,
) -> Result<AiAccount, String> {
    ai_usage_service::update_account(request).map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_usage_delete_account(id: i64) -> Result<(), String> {
    ai_usage_service::delete_account(id).map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_usage_set_active(id: i64) -> Result<(), String> {
    ai_usage_service::set_active(id).map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_usage_get_token(id: i64) -> Result<String, String> {
    ai_usage_service::get_token(id).map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_usage_report_signal(request: ReportUsageSignalRequest) -> Result<(), String> {
    ai_usage_service::report_signal(request).map_err(crate::app::error::log_err)
}

/// Ép probe usage ngay lập tức (toàn bộ account), trả về danh sách sau khi cập nhật.
#[tauri::command]
pub async fn ai_usage_refresh(app: tauri::AppHandle) -> Result<Vec<AiAccount>, String> {
    ai_usage_service::poll_once(&app)
        .await
        .map_err(crate::app::error::log_err)?;
    ai_usage_service::list_accounts().map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_usage_get_settings() -> Result<AiUsageSettings, String> {
    ai_usage_service::get_settings().map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_usage_save_settings(settings: AiUsageSettings) -> Result<(), String> {
    ai_usage_service::save_settings(settings).map_err(crate::app::error::log_err)
}
