//! Tauri command handlers cho Backlog API integration.
//!
//! Thông tin kết nối (URL + API key) được đọc từ section `[backlog]` trong `config.ini`.

use crate::models::backlog::*;
use crate::services::backlog_service;

#[tauri::command]
pub fn backlog_check_config() -> Result<(), String> {
    backlog_service::check_config().map_err(crate::app::error::log_err)
}

#[tauri::command]
pub fn backlog_get_config() -> Result<BacklogConfig, String> {
    backlog_service::get_config().map_err(crate::app::error::log_err)
}

#[tauri::command]
pub fn backlog_save_config(config: BacklogConfig) -> Result<(), String> {
    backlog_service::save_config(&config).map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_get_base_url() -> Result<String, String> {
    backlog_service::get_base_url()
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_get_project(project_key: String) -> Result<BacklogProject, String> {
    backlog_service::get_project(&project_key)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_list_issue_types(
    project_key: String,
) -> Result<Vec<BacklogIssueType>, String> {
    backlog_service::list_issue_types(&project_key)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_list_statuses(project_key: String) -> Result<Vec<BacklogStatus>, String> {
    backlog_service::list_statuses(&project_key)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_list_categories(
    project_key: String,
) -> Result<Vec<BacklogCategory>, String> {
    backlog_service::list_categories(&project_key)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_list_priorities() -> Result<Vec<BacklogPriority>, String> {
    backlog_service::list_priorities()
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_list_project_users(
    project_key: String,
) -> Result<Vec<BacklogUser>, String> {
    backlog_service::list_project_users(&project_key)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_list_issues(query: BacklogIssueQuery) -> Result<BacklogIssueList, String> {
    backlog_service::list_issues(query)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_get_issue(issue_key: String) -> Result<BacklogIssue, String> {
    backlog_service::get_issue(&issue_key)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_get_project_lookup(
    project_key: String,
) -> Result<BacklogProjectLookup, String> {
    backlog_service::get_project_lookup(&project_key)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn backlog_create_issue(
    request: BacklogCreateIssueRequest,
) -> Result<BacklogIssue, String> {
    backlog_service::create_issue(request)
        .await
        .map_err(crate::app::error::log_err)
}
