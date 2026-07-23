use crate::models::ai_task::{AiTask, CreateTaskRequest};
use crate::services::ai_task_service;

#[tauri::command]
pub async fn ai_task_create(
    username: String,
    request: CreateTaskRequest,
) -> Result<AiTask, String> {
    ai_task_service::create_task(&username, request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_task_list(
    keyword: Option<String>,
) -> Result<Vec<AiTask>, String> {
    ai_task_service::list_tasks(keyword)
        .await
        .map_err(crate::app::error::log_err)
}
