use crate::app::result::AppResult;
use crate::database::csv_reader;
use crate::models::import_csv::{ImportCsvPreviewResult, ImportPreviewRow, WorkRecord};

pub fn preview_csv(path: &str) -> AppResult<ImportCsvPreviewResult> {
    let data = csv_reader::read_csv_import_data(path)?;

    Ok(ImportCsvPreviewResult {
        source_path: data.source_path,
        source_file_name: data.source_file_name,
        row_count: data.records.len(),
        total_minutes: data.total_minutes,
        preview_rows: build_preview_rows(&data.records),
        raw_headers: data.raw_csv.headers,
        raw_rows: data.raw_csv.rows,
        minute_column_indexes: data.raw_csv.minute_column_indexes,
    })
}

pub fn read_csv_import_data(
    path: &str,
) -> AppResult<csv_reader::CsvImportData> {
    csv_reader::read_csv_import_data(path)
}

pub fn build_preview_rows(records: &[WorkRecord]) -> Vec<ImportPreviewRow> {
    records
        .iter()
        .map(|record| ImportPreviewRow {
            date: record.date.clone(),
            project_code: record.project_code.clone(),
            project_name: record.project_name.clone(),
            process_code: record.process_code.clone(),
            phase_name: record.phase_name.clone(),
            work_content: record.work_content.clone(),
            total_minutes: record.totals.total_minutes(),
        })
        .collect()
}
