use crate::models::excel2md::XlsxMarkdownResult;
use crate::services::excel2md_service;

#[tauri::command]
pub fn excel2md(
    input_path: String,
    output_path: Option<String>,
) -> Result<XlsxMarkdownResult, String> {
    excel2md_service::convert(input_path, output_path).map_err(|error| error.to_string())
}
