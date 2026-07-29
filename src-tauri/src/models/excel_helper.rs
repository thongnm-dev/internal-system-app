//! Model cho kết quả resize ảnh evidence (hardcopy) trong file Excel.

use serde::{Deserialize, Serialize};

/// Các tuỳ chọn bổ sung khi resize evidence: view/zoom của sheet, cột bắt đầu để đặt lại
/// vị trí ngang của ảnh, và font mặc định của workbook.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcelHelperOptions {
    /// Mở mọi sheet ở chế độ Page Break Preview.
    pub page_break_preview: bool,
    /// Zoom (%) áp dụng cho mọi sheet, nếu có.
    pub zoom_percent: Option<u32>,
    /// Cột bắt đầu để ghi đè vị trí ngang của mọi ảnh (ví dụ "B2" → cột B), nếu có.
    pub start_column: Option<String>,
    /// Tên font mặc định của workbook, nếu có.
    pub font_name: Option<String>,
    /// Cỡ chữ mặc định của workbook, nếu có.
    pub font_size: Option<f64>,
    /// Đưa ô đang chọn (active cell) và vị trí cuộn về A1 trên mọi sheet.
    pub reset_active_cell: bool,
    /// Khi ảnh phình to sẽ đè lên 1 dòng có nội dung, cố gắng chèn thêm 1 dòng để tránh đè lên
    /// (nếu workbook không có formula/table/chart/pivot table — an toàn để đánh số lại dòng).
    /// Nếu không an toàn hoặc bị tắt, ảnh sẽ bị giới hạn chiều cao để không đè lên nội dung.
    pub avoid_covering_content: bool,
}

/// Kết quả sau khi resize toàn bộ ảnh (picture) trong workbook.
#[derive(Serialize)]
pub struct ExcelHelperResult {
    /// Đường dẫn đầy đủ tới file Excel nguồn.
    pub source_path: String,
    /// Đường dẫn đầy đủ tới file Excel đầu ra.
    pub output_path: String,
    /// Tên file Excel nguồn (chỉ tên file, không có thư mục).
    pub source_file_name: String,
    /// Tên file Excel đầu ra.
    pub output_file_name: String,
    /// Số lượng ảnh đã được resize.
    pub images_resized: u32,
    /// Số lượng drawing part (sheet có chứa ảnh/shape) đã được xử lý.
    pub drawings_processed: u32,
    /// Cảnh báo không chặn (ví dụ ảnh liên kết ngoài hoặc không đọc được kích thước gốc).
    pub warnings: Vec<String>,
}
