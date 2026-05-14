use serde::Serialize;

#[derive(Serialize)]
pub struct ImportPreviewRow {
    pub date: String,
    pub project_code: String,
    pub project_name: String,
    pub process_code: String,
    pub phase_name: String,
    pub work_content: String,
    pub total_minutes: i64,
}

#[derive(Serialize)]
pub struct ImportCsvResult {
    pub batch_id: i64,
    pub source_path: String,
    pub source_file_name: String,
    pub imported_at: String,
    pub row_count: usize,
    pub total_minutes: i64,
    pub preview_rows: Vec<ImportPreviewRow>,
    pub raw_headers: Vec<String>,
    pub raw_rows: Vec<Vec<String>>,
    pub minute_column_indexes: Vec<usize>,
}

#[derive(Serialize)]
pub struct ImportCsvPreviewResult {
    pub source_path: String,
    pub source_file_name: String,
    pub row_count: usize,
    pub total_minutes: i64,
    pub preview_rows: Vec<ImportPreviewRow>,
    pub raw_headers: Vec<String>,
    pub raw_rows: Vec<Vec<String>>,
    pub minute_column_indexes: Vec<usize>,
}

#[derive(Serialize)]
pub struct ImportBatchSummary {
    pub id: i64,
    pub source_file_name: String,
    pub imported_at: String,
    pub row_count: i64,
    pub total_minutes: i64,
}
