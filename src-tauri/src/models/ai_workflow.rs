use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiWorkflow {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub layout: String,
    pub created_by: String,
    pub step_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiWorkflowStep {
    pub id: i32,
    pub workflow_id: i32,
    pub name: String,
    pub step_type: String,
    pub skill_name: String,
    pub description: String,
    pub icon: String,
    pub step_order: i32,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateStepRequest {
    pub name: String,
    pub step_type: String,
    pub skill_name: String,
    pub description: String,
    pub icon: String,
    pub step_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStepRequest {
    pub name: String,
    pub step_type: String,
    pub skill_name: String,
    pub description: String,
    pub icon: String,
    pub step_order: i32,
}
