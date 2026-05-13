use crate::domain::statistics::models::AnalysisResult;
use crate::domain::statistics::service;

#[tauri::command]
pub fn analyze_csv(path: String) -> Result<AnalysisResult, String> {
    service::analyze_csv(&path).map_err(|error| error.to_string())
}
