//! Tauri command handler cho chức năng chuyển đổi Excel → Markdown.

use crate::models::excel2md::XlsxMarkdownResult;
use crate::services::excel2md_service;

/// Chuyển đổi file Excel (.xlsx) thành file Markdown.
///
/// Gọi script Python `xlsx_spec_to_markdown.py` để thực hiện chuyển đổi.
/// Nếu `output_path` không được chỉ định, file Markdown sẽ được tạo
/// cùng thư mục với file Excel, cùng tên nhưng đuôi `.md`.
#[tauri::command]
pub fn excel2md(
    input_path: String,
    output_path: Option<String>,
) -> Result<XlsxMarkdownResult, String> {
    excel2md_service::convert(input_path, output_path).map_err(crate::app::error::log_err)
}
