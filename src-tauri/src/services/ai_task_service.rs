use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::ai_task_store;
use crate::models::ai_task::{AiTask, CreateTaskRequest};

const VALID_CATEGORIES: [&str; 4] = ["screen", "batch", "part", "other"];

pub async fn create_task(username: &str, request: CreateTaskRequest) -> AppResult<AiTask> {
    let task_code = request.task_code.trim().to_string();
    if task_code.is_empty() {
        return Err(AppError::new("Task code is required."));
    }

    let category = request.category.trim().to_lowercase();
    let category = if category.is_empty() { "other".to_string() } else { category };
    if !VALID_CATEGORIES.contains(&category.as_str()) {
        return Err(AppError::new(format!(
            "Invalid category '{category}'. Must be one of: screen, batch, part, other."
        )));
    }

    ai_task_store::insert(&task_code, &category, request.description.trim(), username).await
}

pub async fn list_tasks(keyword: Option<String>) -> AppResult<Vec<AiTask>> {
    let keyword = keyword.map(|k| k.trim().to_string()).filter(|k| !k.is_empty());
    ai_task_store::select_list(keyword.as_deref()).await
}
