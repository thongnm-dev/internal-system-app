use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::ai_workflow_store;
use crate::models::ai_workflow::{
    AiWorkflow, AiWorkflowStep, CreateStepRequest, CreateWorkflowRequest, UpdateStepRequest,
    UpdateWorkflowRequest,
};

const VALID_STEP_TYPES: [&str; 5] = ["skill", "implement", "review", "release", "custom"];

pub async fn create_workflow(
    username: &str,
    request: CreateWorkflowRequest,
) -> AppResult<AiWorkflow> {
    let name = request.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::new("Workflow name is required."));
    }
    ai_workflow_store::insert(&name, request.description.trim(), username).await
}

pub async fn list_workflows(username: &str) -> AppResult<Vec<AiWorkflow>> {
    ai_workflow_store::select_list(username).await
}

pub async fn update_workflow(
    id: i32,
    username: &str,
    request: UpdateWorkflowRequest,
) -> AppResult<AiWorkflow> {
    let name = request.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::new("Workflow name is required."));
    }
    ai_workflow_store::update(id, &name, request.description.trim(), username)
        .await?
        .ok_or_else(|| AppError::new(format!("Workflow '{id}' not found.")))
}

pub async fn delete_workflow(id: i32, username: &str) -> AppResult<()> {
    if !ai_workflow_store::delete(id, username).await? {
        return Err(AppError::new(format!("Workflow '{id}' not found.")));
    }
    Ok(())
}

pub async fn list_steps(workflow_id: i32) -> AppResult<Vec<AiWorkflowStep>> {
    ai_workflow_store::select_steps(workflow_id).await
}

pub async fn create_step(
    workflow_id: i32,
    request: CreateStepRequest,
) -> AppResult<AiWorkflowStep> {
    let name = request.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::new("Step name is required."));
    }
    let step_type = request.step_type.trim().to_lowercase();
    if !VALID_STEP_TYPES.contains(&step_type.as_str()) {
        return Err(AppError::new(format!(
            "Invalid step type '{step_type}'. Must be one of: skill, implement, review, release, custom."
        )));
    }
    ai_workflow_store::insert_step(
        workflow_id,
        &name,
        &step_type,
        request.description.trim(),
        request.icon.trim(),
        request.step_order,
    )
    .await
}

pub async fn update_step(
    id: i32,
    request: UpdateStepRequest,
) -> AppResult<AiWorkflowStep> {
    let name = request.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::new("Step name is required."));
    }
    let step_type = request.step_type.trim().to_lowercase();
    if !VALID_STEP_TYPES.contains(&step_type.as_str()) {
        return Err(AppError::new(format!(
            "Invalid step type '{step_type}'. Must be one of: skill, implement, review, release, custom."
        )));
    }
    ai_workflow_store::update_step(
        id,
        &name,
        &step_type,
        request.description.trim(),
        request.icon.trim(),
        request.step_order,
    )
    .await?
    .ok_or_else(|| AppError::new(format!("Step '{id}' not found.")))
}

pub async fn delete_step(id: i32) -> AppResult<()> {
    if !ai_workflow_store::delete_step(id).await? {
        return Err(AppError::new(format!("Step '{id}' not found.")));
    }
    Ok(())
}

pub async fn reorder_steps(workflow_id: i32, step_ids: Vec<i32>) -> AppResult<()> {
    ai_workflow_store::reorder_steps(workflow_id, &step_ids).await
}

pub async fn save_layout(id: i32, username: &str, layout_json: &str) -> AppResult<()> {
    if !ai_workflow_store::update_layout(id, layout_json, username).await? {
        return Err(AppError::new(format!("Workflow '{id}' not found.")));
    }
    Ok(())
}
