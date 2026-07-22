//! Tauri command handlers cho module import CSV báo cáo tháng.
//!
//! Cung cấp các chức năng: preview CSV trước khi import, thực hiện import,
//! liệt kê lịch sử import.

use crate::models::import_csv::ImportCsvPreviewResult;
use crate::models::monthly_report::CompareRow;
use crate::services::monthly_report_service;

#[tauri::command]
pub async fn fetch_csv_from_system(
    date_from: String,
    date_to: String,
    staff: String,
) -> Result<String, String> {
    monthly_report_service::fetch_csv_from_url(&date_from, &date_to, &staff)
        .await
        .map_err(crate::app::error::log_err)
}

/// Preview nội dung file CSV trước khi import.
/// Trả về dữ liệu đã parse và dữ liệu thô để hiển thị trên giao diện.
#[tauri::command]
pub fn preview_monthly_report_csv(path: String) -> Result<ImportCsvPreviewResult, String> {
    monthly_report_service::preview_csv(&path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn compare_monthly_report(
    csv_path: String,
    schedule_path: String,
    target_year: i32,
    target_month: u32,
    user_filter: Option<String>,
) -> Result<Vec<CompareRow>, String> {
    monthly_report_service::compare_csv_with_schedule(
        &csv_path,
        &schedule_path,
        target_year,
        target_month,
        &user_filter.unwrap_or_default(),
    )
    .map_err(|error| error.to_string())
}
