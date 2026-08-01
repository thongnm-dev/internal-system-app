//! Tauri command handler cho công cụ so sánh khác biệt giữa 2 file.

use crate::models::file_compare::CompareResult;
use crate::services::file_compare_service;

/// So sánh 2 file (Markdown, Text, Word `.docx`, Excel). 2 file phải cùng loại.
#[tauri::command]
pub fn file_compare_run(file_a: String, file_b: String) -> Result<CompareResult, String> {
    file_compare_service::compare(&file_a, &file_b).map_err(crate::app::error::log_err)
}

/// Xuất kết quả so sánh 2 file ra file Excel (.xlsx) tại `output_path`.
#[tauri::command]
pub fn file_compare_export(
    file_a: String,
    file_b: String,
    output_path: String,
) -> Result<String, String> {
    file_compare_service::export_excel(&file_a, &file_b, &output_path)
        .map_err(crate::app::error::log_err)
}
