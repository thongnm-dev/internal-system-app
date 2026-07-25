//! Tauri command handlers cho màn hình AI Cowork.

use crate::database::ai_cowork_store::AiCoworkState;
use crate::services::ai_cowork_service;

/// Lấy state làm việc gần nhất (project directory đã chọn lần trước).
#[tauri::command]
pub fn ai_cowork_get_state() -> Result<AiCoworkState, String> {
    ai_cowork_service::get_state().map_err(crate::app::error::log_err)
}

/// Lưu lại toàn bộ state làm việc (project directory, task đang hiển thị, workflow áp dụng).
#[tauri::command]
pub fn ai_cowork_save_state(state: AiCoworkState) -> Result<(), String> {
    ai_cowork_service::save_state(state).map_err(crate::app::error::log_err)
}
