use serde::Serialize;

#[derive(Serialize)]
pub struct XlsxMarkdownResult {
    pub source_path: String,
    pub output_path: String,
    pub source_file_name: String,
    pub output_file_name: String,
    pub markdown: String,
}
