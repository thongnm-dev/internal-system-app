//! Các kiểu dữ liệu cho module lịch sử import báo cáo tháng.
//!
//! `ImportCsvResult` (kết quả import), `ImportBatchSummary` (danh sách).

use crate::models::import_csv::ImportPreviewRow;
use serde::Serialize;

/// Kết quả trả về frontend ngay sau khi import CSV thành công.
/// Bao gồm cả dữ liệu thô để hiển thị bảng xác nhận.
#[derive(Serialize)]
pub struct ImportCsvResult {
    /// ID batch vừa được tạo.
    pub batch_id: i64,
    pub source_path: String,
    pub source_file_name: String,
    /// Thời điểm import (format: YYYY-MM-DD HH:MM:SS).
    pub imported_at: String,
    pub row_count: usize,
    pub total_minutes: i64,
    pub preview_rows: Vec<ImportPreviewRow>,
    pub raw_headers: Vec<String>,
    pub raw_rows: Vec<Vec<String>>,
    pub minute_column_indexes: Vec<usize>,
}

/// Tóm tắt một batch import, dùng cho danh sách gần đây.
#[derive(Serialize)]
pub struct ImportBatchSummary {
    pub id: i64,
    pub source_file_name: String,
    pub imported_at: String,
    /// Tên báo cáo do người dùng đặt (hoặc tự sinh từ tên file).
    pub report_name: String,
    /// Ghi chú tùy chọn.
    pub note: String,
    /// Tháng bắt đầu của dữ liệu (format: YYYY-MM).
    pub target_month_from: String,
    /// Tháng kết thúc của dữ liệu.
    pub target_month_to: String,
    /// Người thực hiện import (lấy từ biến môi trường USERNAME).
    pub imported_by: String,
    pub row_count: i64,
    pub total_minutes: i64,
}
