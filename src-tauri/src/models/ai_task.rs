use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiTask {
    pub id: i32,
    pub task_code: String,
    pub category: String,
    pub description: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub task_code: String,
    pub category: String,
    pub description: String,
}
