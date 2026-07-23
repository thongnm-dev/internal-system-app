use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiTask {
    pub id: i32,
    pub task_cd: String,
    pub task_name: String,
    pub category: String,
    pub is_complete: bool,
    pub completed_at: String,
    pub created_at: String,
    pub created_by: String,
    pub updated_at: String,
    pub updated_by: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub task_cd: String,
    pub task_name: String,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub task_cd: String,
    pub task_name: String,
    pub category: String,
    pub is_complete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiTaskWfProc {
    pub id: i32,
    pub task_id: i32,
    pub wf_id: i32,
    pub latest_step_id: Option<i32>,
    pub created_at: String,
    pub created_by: String,
    pub updated_at: String,
    pub updated_by: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWfProcRequest {
    pub task_id: i32,
    pub wf_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiTaskWfProcStep {
    pub id: i32,
    pub wf_proc_id: i32,
    pub wf_step_id: i32,
    pub status: String,
    pub created_at: String,
    pub created_by: String,
    pub updated_at: String,
    pub updated_by: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWfProcStepRequest {
    pub wf_proc_id: i32,
    pub wf_step_id: i32,
    pub status: String,
}
