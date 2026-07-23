use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::ai_task_store;
use crate::models::ai_task::{
    AiTask, AiTaskWfProc, AiTaskWfProcStep,
    CreateTaskRequest, CreateWfProcRequest, CreateWfProcStepRequest,
    UpdateTaskRequest,
};

const VALID_CATEGORIES: [&str; 4] = ["screen", "batch", "part", "other"];
const VALID_STEP_STATUSES: [&str; 4] = ["pending", "in_progress", "completed", "skipped"];

// ── ai_tasks ────────────────────────────────────────────────────────────

fn validate_category(raw: &str) -> AppResult<String> {
    let category = raw.trim().to_lowercase();
    let category = if category.is_empty() { "other".to_string() } else { category };
    if !VALID_CATEGORIES.contains(&category.as_str()) {
        return Err(AppError::new(format!(
            "Invalid category '{category}'. Must be one of: screen, batch, part, other."
        )));
    }
    Ok(category)
}

pub async fn create_task(username: &str, request: CreateTaskRequest) -> AppResult<AiTask> {
    let task_cd = request.task_cd.trim().to_string();
    if task_cd.is_empty() {
        return Err(AppError::new("Task code is required."));
    }
    let category = validate_category(&request.category)?;
    ai_task_store::insert(&task_cd, request.task_name.trim(), &category, username).await
}

pub async fn list_tasks(keyword: Option<String>, is_complete: Option<bool>) -> AppResult<Vec<AiTask>> {
    let keyword = keyword.map(|k| k.trim().to_string()).filter(|k| !k.is_empty());
    ai_task_store::select_list(keyword.as_deref(), is_complete).await
}

pub async fn update_task(id: i32, username: &str, request: UpdateTaskRequest) -> AppResult<AiTask> {
    let task_cd = request.task_cd.trim().to_string();
    if task_cd.is_empty() {
        return Err(AppError::new("Task code is required."));
    }
    let category = validate_category(&request.category)?;
    ai_task_store::update(id, &task_cd, request.task_name.trim(), &category, request.is_complete, username).await
}

// ── ai_task_wf_proc ─────────────────────────────────────────────────────

pub async fn create_wf_proc(username: &str, request: CreateWfProcRequest) -> AppResult<AiTaskWfProc> {
    ai_task_store::wf_proc_insert(request.task_id, request.wf_id, username).await
}

pub async fn list_wf_procs(task_id: i32) -> AppResult<Vec<AiTaskWfProc>> {
    ai_task_store::wf_proc_select_by_task(task_id).await
}

pub async fn update_wf_proc(id: i32, latest_step_id: i32, username: &str) -> AppResult<AiTaskWfProc> {
    ai_task_store::wf_proc_update(id, latest_step_id, username).await
}

// ── ai_task_wf_proc_step ────────────────────────────────────────────────

pub async fn create_wf_proc_step(username: &str, request: CreateWfProcStepRequest) -> AppResult<AiTaskWfProcStep> {
    let status = request.status.trim().to_lowercase();
    let status = if status.is_empty() { "pending".to_string() } else { status };
    if !VALID_STEP_STATUSES.contains(&status.as_str()) {
        return Err(AppError::new(format!(
            "Invalid status '{status}'. Must be one of: pending, in_progress, completed, skipped."
        )));
    }
    ai_task_store::wf_proc_step_insert(request.wf_proc_id, request.wf_step_id, &status, username).await
}

pub async fn list_wf_proc_steps(wf_proc_id: i32) -> AppResult<Vec<AiTaskWfProcStep>> {
    ai_task_store::wf_proc_step_select_by_proc(wf_proc_id).await
}

pub async fn update_wf_proc_step(id: i32, status: &str, username: &str) -> AppResult<AiTaskWfProcStep> {
    let status = status.trim().to_lowercase();
    if !VALID_STEP_STATUSES.contains(&status.as_str()) {
        return Err(AppError::new(format!(
            "Invalid status '{status}'. Must be one of: pending, in_progress, completed, skipped."
        )));
    }
    ai_task_store::wf_proc_step_update(id, &status, username).await
}
