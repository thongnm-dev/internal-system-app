use crate::services::xlsx_markdown::models::XlsxMarkdownResult;
use crate::services::xlsx_markdown::service;

#[tauri::command]
pub fn convert_xlsx_spec_to_markdown(
    input_path: String,
    output_path: Option<String>,
) -> Result<XlsxMarkdownResult, String> {
    service::convert(input_path, output_path).map_err(|error| error.to_string())
}
