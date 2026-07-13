//! Tauri command handlers cho module import CSV báo cáo tháng.
//!
//! Cung cấp các chức năng: preview CSV trước khi import, thực hiện import,
//! liệt kê và tìm kiếm lịch sử import, xem chi tiết một batch.

use crate::models::import_csv::ImportCsvPreviewResult;
use crate::models::monthly_report::{
    ImportBatchDetail, ImportBatchListItem, ImportBatchSearchCriteria, ImportBatchSummary,
    ImportCsvResult,
};
use crate::services::import_csv_service;
use crate::services::monthly_report_service;
use tauri::Manager;

/// Preview nội dung file CSV trước khi import.
/// Trả về dữ liệu đã parse và dữ liệu thô để hiển thị trên giao diện.
#[tauri::command]
pub fn preview_monthly_report_csv(path: String) -> Result<ImportCsvPreviewResult, String> {
    import_csv_service::preview_csv(&path).map_err(|error| error.to_string())
}

/// Import file CSV vào kho lưu trữ (JSON file trong app data directory).
/// Tự động tính tổng phút, xác định khoảng tháng, và lưu lịch sử.
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
    let data =
        import_csv_service::read_csv_import_data(&path).map_err(|error| error.to_string())?;
    monthly_report_service::save_imported_report(data, &app_data_dir, report_name, note)
        .map_err(|error| error.to_string())
}

/// Lấy danh sách 20 batch import gần nhất, sắp xếp theo ID giảm dần.
#[tauri::command]
pub fn list_import_batches(app: tauri::AppHandle) -> Result<Vec<ImportBatchSummary>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    monthly_report_service::list_import_batches(&app_data_dir).map_err(|error| error.to_string())
}

/// Tìm kiếm batch import theo điều kiện (tháng, tên báo cáo, từ khóa).
#[tauri::command]
pub fn search_import_batches(
    app: tauri::AppHandle,
    criteria: ImportBatchSearchCriteria,
) -> Result<Vec<ImportBatchListItem>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    monthly_report_service::search_import_batches(&app_data_dir, criteria)
        .map_err(|error| error.to_string())
}

/// Lấy thông tin chi tiết một batch import theo `batch_id`.
/// Bao gồm danh sách preview rows để hiển thị lại dữ liệu đã import.
#[tauri::command]
pub fn get_import_batch_detail(
    app: tauri::AppHandle,
    batch_id: i64,
) -> Result<ImportBatchDetail, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    monthly_report_service::get_import_batch_detail(&app_data_dir, batch_id)
        .map_err(|error| error.to_string())
}
