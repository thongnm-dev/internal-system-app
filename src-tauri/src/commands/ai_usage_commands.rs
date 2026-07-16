//! Tauri command handlers cho module AI Usage.

use crate::models::ai_usage::{AddAiAccountRequest, AiAccount};
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
pub async fn ai_usage_delete_account(id: i64) -> Result<(), String> {
    ai_usage_service::delete_account(id).map_err(|e| e.to_string())
}
