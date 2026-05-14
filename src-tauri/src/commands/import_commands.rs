use crate::domain::monthly_report_import::models::{
    ImportBatchListItem, ImportBatchSearchCriteria, ImportBatchSummary, ImportCsvPreviewResult,
    ImportCsvResult,
};
use crate::domain::monthly_report_import::service;
use tauri::Manager;

#[tauri::command]
pub fn preview_monthly_report_csv(path: String) -> Result<ImportCsvPreviewResult, String> {
    service::preview_csv(&path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_monthly_report_csv(
    app: tauri::AppHandle,
    path: String,
    report_name: Option<String>,
    note: Option<String>,
) -> Result<ImportCsvResult, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    service::import_csv_to_database(&path, &app_data_dir, report_name, note).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_import_batches(app: tauri::AppHandle) -> Result<Vec<ImportBatchSummary>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    service::list_import_batches(&app_data_dir).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn search_import_batches(
    app: tauri::AppHandle,
    criteria: ImportBatchSearchCriteria,
) -> Result<Vec<ImportBatchListItem>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    service::search_import_batches(&app_data_dir, criteria).map_err(|error| error.to_string())
}
