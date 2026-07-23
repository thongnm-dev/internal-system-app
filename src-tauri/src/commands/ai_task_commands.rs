use crate::models::ai_task::{
    AiTask, AiTaskWfProc, AiTaskWfProcStep,
    CreateTaskRequest, CreateWfProcRequest, CreateWfProcStepRequest,
    UpdateTaskRequest,
};
use crate::services::ai_task_service;

// ── ai_tasks ────────────────────────────────────────────────────────────

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
    is_complete: Option<bool>,
) -> Result<Vec<AiTask>, String> {
    ai_task_service::list_tasks(keyword, is_complete)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_task_update(
    id: i32,
    username: String,
    request: UpdateTaskRequest,
) -> Result<AiTask, String> {
    ai_task_service::update_task(id, &username, request)
        .await
        .map_err(crate::app::error::log_err)
}

// ── ai_task_wf_proc ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn ai_task_wf_proc_create(
    username: String,
    request: CreateWfProcRequest,
) -> Result<AiTaskWfProc, String> {
    ai_task_service::create_wf_proc(&username, request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_task_wf_proc_list(
    task_id: i32,
) -> Result<Vec<AiTaskWfProc>, String> {
    ai_task_service::list_wf_procs(task_id)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_task_wf_proc_update(
    id: i32,
    latest_step_id: i32,
    username: String,
) -> Result<AiTaskWfProc, String> {
    ai_task_service::update_wf_proc(id, latest_step_id, &username)
        .await
        .map_err(crate::app::error::log_err)
}

// ── ai_task_wf_proc_step ────────────────────────────────────────────────

#[tauri::command]
pub async fn ai_task_wf_proc_step_create(
    username: String,
    request: CreateWfProcStepRequest,
) -> Result<AiTaskWfProcStep, String> {
    ai_task_service::create_wf_proc_step(&username, request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_task_wf_proc_step_list(
    wf_proc_id: i32,
) -> Result<Vec<AiTaskWfProcStep>, String> {
    ai_task_service::list_wf_proc_steps(wf_proc_id)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_task_wf_proc_step_update(
    id: i32,
    status: String,
    username: String,
) -> Result<AiTaskWfProcStep, String> {
    ai_task_service::update_wf_proc_step(id, &status, &username)
        .await
        .map_err(crate::app::error::log_err)
}
