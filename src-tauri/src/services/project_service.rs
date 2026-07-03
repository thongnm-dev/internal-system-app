use crate::app::result::AppResult;
use crate::models::project::{BacklogProjectLookup, ProjectDetail};

pub fn get_project_detail(project_id: String) -> AppResult<ProjectDetail> {
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

pub fn get_backlog_project_by_key(project_key: String) -> AppResult<BacklogProjectLookup> {
    Err(crate::app::error::AppError::new(format!(
        "Backlog project lookup is not configured. Key: {}",
        project_key
    )))
}
