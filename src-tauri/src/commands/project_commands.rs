use crate::models::project::{BacklogProjectLookup, ProjectDetail};
use crate::services::project_service;

#[tauri::command]
pub fn get_project_detail(project_id: String) -> Result<ProjectDetail, String> {
    project_service::get_project_detail(project_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_backlog_project_by_key(project_key: String) -> Result<BacklogProjectLookup, String> {
    project_service::get_backlog_project_by_key(project_key).map_err(|error| error.to_string())
}
