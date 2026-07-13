//! Model cho kết quả chuyển đổi Excel (.xlsx) sang Markdown.

use serde::Serialize;

/// Kết quả sau khi chuyển đổi file Excel thành Markdown.
/// Chứa thông tin đường dẫn nguồn/đích và nội dung Markdown đã tạo.
#[derive(Serialize)]
pub struct XlsxMarkdownResult {
    /// Đường dẫn đầy đủ tới file Excel nguồn.
    pub source_path: String,
    /// Đường dẫn đầy đủ tới file Markdown đầu ra.
    pub output_path: String,
    /// Tên file Excel nguồn (chỉ tên file, không có thư mục).
    pub source_file_name: String,
    /// Tên file Markdown đầu ra.
    pub output_file_name: String,
    /// Nội dung Markdown đã chuyển đổi.
    pub markdown: String,
}
