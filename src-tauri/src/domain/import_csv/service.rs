use crate::app::result::AppResult;
use crate::domain::import_csv::models::{
    ImportCsvPreviewResult, ImportPreviewRow, MinuteTotals, WorkRecord,
};
use crate::domain::import_csv::phase::phase_label;
use crate::infrastructure::csv::reader::{get_required_index, parse_minutes, read_shift_jis_csv};
use crate::infrastructure::file_system::{display_path, resolve_input_path};
use std::path::{Path, PathBuf};

const DATE: &str = "日付";
const PROJECT_CODE: &str = "プロジェクトコード";
const PROJECT_NAME: &str = "プロジェクト名";
const PROCESS_CODE: &str = "プロセスコード";
const PROCESS_NAME: &str = "プロセス";
const WORK_CONTENT: &str = "作業内容";
pub(crate) const REGULAR_MINUTES: &str = "時間内(分)";
pub(crate) const NORMAL_OVERTIME: &str = "普通残業時間(分)";
pub(crate) const LEGAL_HOLIDAY_OVERTIME: &str = "法定休日残業(分)";
pub(crate) const LEGAL_PUBLIC_HOLIDAY_OVERTIME: &str = "法定祝日残業時間(分)";
pub(crate) const LATE_NIGHT_OVERTIME: &str = "深夜残業(分)";

pub fn preview_csv(path: &str) -> AppResult<ImportCsvPreviewResult> {
    let data = read_csv_import_data(path)?;

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

pub(crate) fn read_csv_import_data(path: &str) -> AppResult<CsvImportData> {
    let (input_path, records) = read_work_records(path)?;
    let raw_csv = read_raw_csv(&input_path)?;
    let source_path = display_path(&input_path);
    let source_file_name = input_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| source_path.clone());
    let total_minutes: i64 = records
        .iter()
        .map(|record| record.totals.total_minutes())
        .sum();

    Ok(CsvImportData {
        source_path,
        source_file_name,
        records,
        total_minutes,
        raw_csv,
    })
}

pub fn read_work_records(path: &str) -> AppResult<(PathBuf, Vec<WorkRecord>)> {
    let input_path = resolve_input_path(path)?;
    let content = read_shift_jis_csv(&input_path)?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());
    let headers = reader.headers()?.clone();
    let date_idx = get_required_index(&headers, DATE)?;
    let project_code_idx = get_required_index(&headers, PROJECT_CODE)?;
    let project_name_idx = get_required_index(&headers, PROJECT_NAME)?;
    let process_code_idx = get_required_index(&headers, PROCESS_CODE)?;
    let process_name_idx = get_required_index(&headers, PROCESS_NAME)?;
    let work_content_idx = get_required_index(&headers, WORK_CONTENT)?;
    let regular_idx = get_required_index(&headers, REGULAR_MINUTES)?;
    let normal_overtime_idx = get_required_index(&headers, NORMAL_OVERTIME)?;
    let legal_holiday_idx = get_required_index(&headers, LEGAL_HOLIDAY_OVERTIME)?;
    let legal_public_holiday_idx = get_required_index(&headers, LEGAL_PUBLIC_HOLIDAY_OVERTIME)?;
    let late_night_idx = get_required_index(&headers, LATE_NIGHT_OVERTIME)?;

    let mut records = Vec::new();

    for record in reader.records() {
        let record = record?;
        let date = record.get(date_idx).unwrap_or_default().trim();
        let project_code = record.get(project_code_idx).unwrap_or_default().trim();
        let project_name = record.get(project_name_idx).unwrap_or_default().trim();
        let process_code = record.get(process_code_idx).unwrap_or_default().trim();
        let process_name = record.get(process_name_idx).unwrap_or_default().trim();
        let work_content = record.get(work_content_idx).unwrap_or_default().trim();
        let phase_name = phase_label(process_code, process_name);
        let totals = MinuteTotals {
            regular_minutes: parse_minutes(record.get(regular_idx)),
            normal_overtime_minutes: parse_minutes(record.get(normal_overtime_idx)),
            legal_holiday_overtime_minutes: parse_minutes(record.get(legal_holiday_idx)),
            legal_public_holiday_overtime_minutes: parse_minutes(
                record.get(legal_public_holiday_idx),
            ),
            late_night_overtime_minutes: parse_minutes(record.get(late_night_idx)),
        };
        records.push(WorkRecord {
            date: date.to_string(),
            project_code: project_code.to_string(),
            project_name: project_name.to_string(),
            process_code: process_code.to_string(),
            process_name: process_name.to_string(),
            phase_name,
            work_content: work_content.to_string(),
            totals,
        });
    }

    Ok((input_path, records))
}

pub(crate) fn build_preview_rows(records: &[WorkRecord]) -> Vec<ImportPreviewRow> {
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

pub(crate) struct RawCsv {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub minute_column_indexes: Vec<usize>,
}

pub(crate) struct CsvImportData {
    pub source_path: String,
    pub source_file_name: String,
    pub records: Vec<WorkRecord>,
    pub total_minutes: i64,
    pub raw_csv: RawCsv,
}

pub(crate) fn read_raw_csv(input_path: &Path) -> AppResult<RawCsv> {
    let content = read_shift_jis_csv(input_path)?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());
    let headers = reader.headers()?.clone();
    let minute_column_indexes = [
        REGULAR_MINUTES,
        NORMAL_OVERTIME,
        LEGAL_HOLIDAY_OVERTIME,
        LEGAL_PUBLIC_HOLIDAY_OVERTIME,
        LATE_NIGHT_OVERTIME,
    ]
    .iter()
    .map(|name| get_required_index(&headers, name))
    .collect::<AppResult<Vec<usize>>>()?;

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        rows.push(
            record
                .iter()
                .map(|value| value.trim().to_string())
                .collect(),
        );
    }

    Ok(RawCsv {
        headers: headers.iter().map(|value| value.to_string()).collect(),
        rows,
        minute_column_indexes,
    })
}
