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

#[tauri::command]
pub fn get_project_detail(project_id: String) -> Result<ProjectDetail, String> {
    Ok(ProjectDetail {
        project_id: project_id.clone(),
        project_code: project_id,
        project_name: String::new(),
        backlog_project_id: None,
        backlog_project_key: None,
        backlog_project_name: None,
        members: Vec::new(),
    })
}

#[tauri::command]
pub fn get_backlog_project_by_key(project_key: String) -> Result<BacklogProjectLookup, String> {
    Err(format!(
        "Backlog project lookup is not configured. Key: {}",
        project_key
    ))
}
