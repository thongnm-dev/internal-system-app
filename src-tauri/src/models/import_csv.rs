//! Các kiểu dữ liệu cho module import CSV công việc.
//!
//! Dùng để parse dữ liệu từ file CSV chấm công (format tiếng Nhật, Shift-JIS)
//! thành các bản ghi công việc với phân loại giờ làm.

use serde::{Deserialize, Serialize};

/// Tổng hợp số phút làm việc theo từng loại (giờ thường, tăng ca, v.v.).
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct MinuteTotals {
    /// Số phút làm việc trong giờ hành chính.
    pub regular_minutes: i64,
    /// Số phút tăng ca thông thường.
    pub normal_overtime_minutes: i64,
    /// Số phút tăng ca ngày nghỉ lễ theo luật.
    pub legal_holiday_overtime_minutes: i64,
    /// Số phút tăng ca ngày lễ công cộng theo luật.
    pub legal_public_holiday_overtime_minutes: i64,
    /// Số phút tăng ca ban đêm (sau 22h).
    pub late_night_overtime_minutes: i64,
}

impl MinuteTotals {
    /// Tính tổng tất cả các loại phút làm việc.
    pub fn total_minutes(&self) -> i64 {
        self.regular_minutes
            + self.normal_overtime_minutes
            + self.legal_holiday_overtime_minutes
            + self.legal_public_holiday_overtime_minutes
            + self.late_night_overtime_minutes
    }
}

/// Một bản ghi công việc đã parse từ CSV.
/// Chứa thông tin ngày, dự án, phase, nội dung công việc và số phút.
#[derive(Clone, Deserialize, Serialize)]
pub struct WorkRecord {
    /// Ngày công việc (format gốc từ CSV, ví dụ: "2026/01/15").
    pub date: String,
    /// Mã dự án.
    pub project_code: String,
    /// Tên dự án.
    pub project_name: String,
    /// Mã phase/process (ví dụ: "10" = PG, "11" = UT).
    pub process_code: String,
    /// Tên process gốc từ CSV.
    pub process_name: String,
    /// Tên phase đã được map (PG, UT, Bug, v.v.).
    pub phase_name: String,
    /// Nội dung công việc cụ thể.
    pub work_content: String,
    /// Phân loại số phút theo từng loại giờ làm.
    pub totals: MinuteTotals,
}

/// Một dòng preview (tóm tắt) để hiển thị trên giao diện trước khi import.
#[derive(Serialize)]
pub struct ImportPreviewRow {
    pub date: String,
    pub project_code: String,
    pub project_name: String,
    pub process_code: String,
    pub phase_name: String,
    pub work_content: String,
    /// Tổng số phút (tất cả loại giờ cộng lại).
    pub total_minutes: i64,
}

/// Kết quả preview file CSV trước khi import.
/// Chứa cả dữ liệu đã parse lẫn dữ liệu thô (raw) để hiển thị bảng gốc.
#[derive(Serialize)]
pub struct ImportCsvPreviewResult {
    /// Đường dẫn đầy đủ tới file CSV nguồn.
    pub source_path: String,
    /// Tên file CSV.
    pub source_file_name: String,
    /// Tổng số dòng dữ liệu (không tính header).
    pub row_count: usize,
    /// Tổng số phút của toàn bộ file.
    pub total_minutes: i64,
    /// Danh sách dòng preview đã parse.
    pub preview_rows: Vec<ImportPreviewRow>,
    /// Headers gốc từ file CSV (tiếng Nhật).
    pub raw_headers: Vec<String>,
    /// Dữ liệu thô từng dòng.
    pub raw_rows: Vec<Vec<String>>,
    /// Chỉ số các cột chứa giá trị phút (dùng để highlight trên UI).
    pub minute_column_indexes: Vec<usize>,
}
