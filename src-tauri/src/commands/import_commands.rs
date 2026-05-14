use crate::domain::import_csv::models::ImportCsvPreviewResult;
use crate::domain::import_csv::service as import_csv_service;
use crate::domain::monthly_report::models::{
    ImportBatchListItem, ImportBatchSearchCriteria, ImportBatchSummary, ImportCsvResult,
};
use crate::domain::monthly_report::service as monthly_report_service;
use tauri::Manager;

#[tauri::command]
pub fn preview_monthly_report_csv(path: String) -> Result<ImportCsvPreviewResult, String> {
    import_csv_service::preview_csv(&path).map_err(|error| error.to_string())
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
    let data = import_csv_service::read_csv_import_data(&path).map_err(|error| error.to_string())?;
    monthly_report_service::save_imported_report(data, &app_data_dir, report_name, note)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_import_batches(app: tauri::AppHandle) -> Result<Vec<ImportBatchSummary>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    monthly_report_service::list_import_batches(&app_data_dir).map_err(|error| error.to_string())
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
    monthly_report_service::search_import_batches(&app_data_dir, criteria).map_err(|error| error.to_string())
}
