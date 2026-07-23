use crate::models::ai_workflow::{
    AiWorkflow, AiWorkflowStep, CreateStepRequest, CreateWorkflowRequest, UpdateStepRequest,
    UpdateWorkflowRequest,
};
use crate::services::ai_workflow_service;

#[tauri::command]
pub async fn ai_workflow_create(
    username: String,
    request: CreateWorkflowRequest,
) -> Result<AiWorkflow, String> {
    ai_workflow_service::create_workflow(&username, request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_workflow_list(
    username: String,
) -> Result<Vec<AiWorkflow>, String> {
    ai_workflow_service::list_workflows(&username)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_workflow_update(
    id: i32,
    username: String,
    request: UpdateWorkflowRequest,
) -> Result<AiWorkflow, String> {
    ai_workflow_service::update_workflow(id, &username, request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_workflow_delete(
    id: i32,
    username: String,
) -> Result<(), String> {
    ai_workflow_service::delete_workflow(id, &username)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_workflow_step_list(
    workflow_id: i32,
) -> Result<Vec<AiWorkflowStep>, String> {
    ai_workflow_service::list_steps(workflow_id)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_workflow_step_create(
    workflow_id: i32,
    request: CreateStepRequest,
) -> Result<AiWorkflowStep, String> {
    ai_workflow_service::create_step(workflow_id, request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_workflow_step_update(
    id: i32,
    request: UpdateStepRequest,
) -> Result<AiWorkflowStep, String> {
    ai_workflow_service::update_step(id, request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_workflow_step_delete(
    id: i32,
) -> Result<(), String> {
    ai_workflow_service::delete_step(id)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_workflow_step_reorder(
    workflow_id: i32,
    step_ids: Vec<i32>,
) -> Result<(), String> {
    ai_workflow_service::reorder_steps(workflow_id, step_ids)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn ai_workflow_save_layout(
    id: i32,
    username: String,
    layout_json: String,
) -> Result<(), String> {
    ai_workflow_service::save_layout(id, &username, &layout_json)
        .await
        .map_err(crate::app::error::log_err)
}
