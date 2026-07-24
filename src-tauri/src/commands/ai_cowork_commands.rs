//! Tauri command handlers cho màn hình AI Cowork.

use crate::database::ai_cowork_store::AiCoworkState;
use crate::services::ai_cowork_service;

/// Lấy state làm việc gần nhất (project directory đã chọn lần trước).
#[tauri::command]
pub fn ai_cowork_get_state() -> Result<AiCoworkState, String> {
    ai_cowork_service::get_state().map_err(crate::app::error::log_err)
}

/// Lưu lại project directory đang làm việc.
#[tauri::command]
pub fn ai_cowork_save_state(project_dir: String) -> Result<(), String> {
    ai_cowork_service::save_state(project_dir).map_err(crate::app::error::log_err)
}
