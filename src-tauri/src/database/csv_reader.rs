use crate::app::result::AppResult;
use crate::models::import_csv::{MinuteTotals, WorkRecord};
use crate::app::consts::phase_label;
use crate::utils::csv_reader::{display_path, resolve_input_path, CsvReader};
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

pub struct RawCsv {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub minute_column_indexes: Vec<usize>,
}

pub fn read_work_records(path: &str) -> AppResult<(PathBuf, Vec<WorkRecord>)> {
    let input_path = resolve_input_path(path)?;
    let csv = CsvReader::from_path(&input_path)?;

    let date_idx = csv.column_index(DATE)?;
    let project_code_idx = csv.column_index(PROJECT_CODE)?;
    let project_name_idx = csv.column_index(PROJECT_NAME)?;
    let process_code_idx = csv.column_index(PROCESS_CODE)?;
    let process_name_idx = csv.column_index(PROCESS_NAME)?;
    let work_content_idx = csv.column_index(WORK_CONTENT)?;
    let regular_idx = csv.column_index(REGULAR_MINUTES)?;
    let normal_overtime_idx = csv.column_index(NORMAL_OVERTIME)?;
    let legal_holiday_idx = csv.column_index(LEGAL_HOLIDAY_OVERTIME)?;
    let legal_public_holiday_idx = csv.column_index(LEGAL_PUBLIC_HOLIDAY_OVERTIME)?;
    let late_night_idx = csv.column_index(LATE_NIGHT_OVERTIME)?;

    let mut records = Vec::new();

    for row in csv.rows() {
        let date = csv.get_value(row, date_idx);
        let project_code = csv.get_value(row, project_code_idx);
        let project_name = csv.get_value(row, project_name_idx);
        let process_code = csv.get_value(row, process_code_idx);
        let process_name = csv.get_value(row, process_name_idx);
        let work_content = csv.get_value(row, work_content_idx);
        let phase_name = phase_label(process_code, process_name);
        let totals = MinuteTotals {
            regular_minutes: CsvReader::parse_i64(csv.get_value(row, regular_idx)),
            normal_overtime_minutes: CsvReader::parse_i64(csv.get_value(row, normal_overtime_idx)),
            legal_holiday_overtime_minutes: CsvReader::parse_i64(
                csv.get_value(row, legal_holiday_idx),
            ),
            legal_public_holiday_overtime_minutes: CsvReader::parse_i64(
                csv.get_value(row, legal_public_holiday_idx),
            ),
            late_night_overtime_minutes: CsvReader::parse_i64(
                csv.get_value(row, late_night_idx),
            ),
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

pub fn read_raw_csv(input_path: &Path) -> AppResult<RawCsv> {
    let csv = CsvReader::from_path(input_path)?;

    let minute_column_indexes = [
        REGULAR_MINUTES,
        NORMAL_OVERTIME,
        LEGAL_HOLIDAY_OVERTIME,
        LEGAL_PUBLIC_HOLIDAY_OVERTIME,
        LATE_NIGHT_OVERTIME,
    ]
    .iter()
    .map(|name| csv.column_index(name))
    .collect::<AppResult<Vec<usize>>>()?;

    Ok(RawCsv {
        headers: csv.headers().to_vec(),
        rows: csv.rows().to_vec(),
        minute_column_indexes,
    })
}

pub fn read_csv_import_data(path: &str) -> AppResult<CsvImportData> {
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

pub struct CsvImportData {
    pub source_path: String,
    pub source_file_name: String,
    pub records: Vec<WorkRecord>,
    pub total_minutes: i64,
    pub raw_csv: RawCsv,
}
