//! Tauri command handlers cho Backlog API integration.
//!
//! Thông tin kết nối (URL + API key) được đọc tự động từ bảng `api_keys`
//! trong database với `name = 'ALX_BACKLOG'`.

use crate::models::backlog::*;
use crate::services::backlog_service;

#[tauri::command]
pub async fn backlog_get_space() -> Result<BacklogSpace, String> {
    backlog_service::get_space()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backlog_get_project(project_key: String) -> Result<BacklogProject, String> {
    backlog_service::get_project(&project_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backlog_list_issue_types(
    project_key: String,
) -> Result<Vec<BacklogIssueType>, String> {
    backlog_service::list_issue_types(&project_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backlog_list_statuses(project_key: String) -> Result<Vec<BacklogStatus>, String> {
    backlog_service::list_statuses(&project_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backlog_list_categories(
    project_key: String,
) -> Result<Vec<BacklogCategory>, String> {
    backlog_service::list_categories(&project_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backlog_list_milestones(
    project_key: String,
) -> Result<Vec<BacklogMilestone>, String> {
    backlog_service::list_milestones(&project_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backlog_list_project_users(
    project_key: String,
) -> Result<Vec<BacklogUser>, String> {
    backlog_service::list_project_users(&project_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backlog_list_issues(query: BacklogIssueQuery) -> Result<BacklogIssueList, String> {
    backlog_service::list_issues(query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backlog_get_issue(issue_key: String) -> Result<BacklogIssue, String> {
    backlog_service::get_issue(&issue_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backlog_get_project_lookup(
    project_key: String,
) -> Result<BacklogProjectLookup, String> {
    backlog_service::get_project_lookup(&project_key)
        .await
        .map_err(|e| e.to_string())
}
