//! Tauri command handlers cho màn hình AI Translate Cowork.

use crate::database::ai_translate_cowork_store::AiTranslateCoworkState;
use crate::services::ai_translate_cowork_service;

/// Lấy state làm việc gần nhất (project directory đã chọn lần trước).
#[tauri::command]
pub fn ai_translate_cowork_get_state() -> Result<AiTranslateCoworkState, String> {
    ai_translate_cowork_service::get_state().map_err(crate::app::error::log_err)
}

/// Lưu lại project directory đang làm việc.
#[tauri::command]
pub fn ai_translate_cowork_save_state(project_dir: String) -> Result<(), String> {
    ai_translate_cowork_service::save_state(project_dir).map_err(crate::app::error::log_err)
}
