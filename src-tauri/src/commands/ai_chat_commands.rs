//! Tauri command handlers cho module AI Chat.

use crate::models::ai_chat::{AiChatRequest, AiChatResponse};
use crate::services::ai_chat_service;

#[tauri::command]
pub async fn ai_chat_complete(request: AiChatRequest) -> Result<AiChatResponse, String> {
    ai_chat_service::complete(request)
        .await
        .map_err(|e| e.to_string())
}
