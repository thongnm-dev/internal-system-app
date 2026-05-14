use crate::domain::import_csv::models::ImportPreviewRow;
use serde::Serialize;

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
pub struct ImportBatchSummary {
    pub id: i64,
    pub source_file_name: String,
    pub imported_at: String,
    pub report_name: String,
    pub note: String,
    pub target_month_from: String,
    pub target_month_to: String,
    pub imported_by: String,
    pub row_count: i64,
    pub total_minutes: i64,
}

#[derive(serde::Deserialize)]
pub struct ImportBatchSearchCriteria {
    pub target_month_from: Option<String>,
    pub target_month_to: Option<String>,
    pub report_name: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Serialize)]
pub struct ImportBatchListItem {
    pub id: i64,
    pub report_name: String,
    pub note: String,
    pub source_file_name: String,
    pub imported_at: String,
    pub imported_by: String,
    pub target_month_from: String,
    pub target_month_to: String,
    pub row_count: i64,
    pub total_minutes: i64,
}

#[derive(Serialize)]
pub struct ImportBatchDetail {
    pub id: i64,
    pub report_name: String,
    pub note: String,
    pub source_path: String,
    pub source_file_name: String,
    pub imported_at: String,
    pub imported_by: String,
    pub target_month_from: String,
    pub target_month_to: String,
    pub row_count: i64,
    pub total_minutes: i64,
    pub preview_rows: Vec<ImportPreviewRow>,
}
