use crate::models::issue_csv::IssueCsvRow;
use crate::services::issue_csv_service;
use std::path::Path;

#[tauri::command]
pub fn parse_issue_csv(path: String) -> Result<Vec<IssueCsvRow>, String> {
    issue_csv_service::parse_issue_csv(Path::new(&path)).map_err(|e| e.to_string())
}
