use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMember {
    pub username: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDetail {
    pub project_id: String,
    pub project_code: String,
    pub project_name: String,
    pub backlog_project_id: Option<String>,
    pub backlog_project_key: Option<String>,
    pub backlog_project_name: Option<String>,
    pub members: Vec<ProjectMember>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BacklogProjectLookup {
    pub project_id: String,
    pub project_key: String,
    pub project_name: String,
}
