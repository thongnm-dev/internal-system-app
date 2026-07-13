//! Đọc và parse file CSV chấm công (format tiếng Nhật) thành dữ liệu work record.
//!
//! File CSV từ hệ thống quản lý công việc Nhật Bản có encoding Shift-JIS
//! và header bằng tiếng Nhật. Module này map các cột tiếng Nhật sang struct Rust.

use crate::app::result::AppResult;
use crate::models::import_csv::{MinuteTotals, WorkRecord};
use crate::app::consts::phase_label;
use crate::utils::csv_reader::{display_path, resolve_input_path, CsvReader};
use std::path::{Path, PathBuf};

// === Tên cột tiếng Nhật trong file CSV ===
const DATE: &str = "日付";
const PROJECT_CODE: &str = "プロジェクトコード";
const PROJECT_NAME: &str = "プロジェクト名";
const PROCESS_CODE: &str = "プロセスコード";
const PROCESS_NAME: &str = "プロセス";
const WORK_CONTENT: &str = "作業内容";

// === Tên cột phút (public cho module khác tham chiếu chỉ số cột) ===
pub(crate) const REGULAR_MINUTES: &str = "時間内(分)";
pub(crate) const NORMAL_OVERTIME: &str = "普通残業時間(分)";
pub(crate) const LEGAL_HOLIDAY_OVERTIME: &str = "法定休日残業(分)";
pub(crate) const LEGAL_PUBLIC_HOLIDAY_OVERTIME: &str = "法定祝日残業時間(分)";
pub(crate) const LATE_NIGHT_OVERTIME: &str = "深夜残業(分)";

/// Dữ liệu thô từ file CSV (headers + rows + chỉ số cột phút).
pub struct RawCsv {
    /// Tên cột gốc (tiếng Nhật).
    pub headers: Vec<String>,
    /// Dữ liệu thô từng dòng.
    pub rows: Vec<Vec<String>>,
    /// Chỉ số các cột chứa giá trị phút (5 cột).
    pub minute_column_indexes: Vec<usize>,
}

/// Đọc file CSV và parse thành danh sách `WorkRecord`.
/// Trả về cả đường dẫn đã resolve để dùng cho display.
pub fn read_work_records(path: &str) -> AppResult<(PathBuf, Vec<WorkRecord>)> {
    let input_path = resolve_input_path(path)?;
    let csv = CsvReader::from_path(&input_path)?;

    // Tìm chỉ số từng cột theo tên tiếng Nhật
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
        // Chuyển mã process sang tên phase hiển thị
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

/// Đọc dữ liệu thô từ file CSV (headers, rows, chỉ số cột phút).
/// Dùng khi cần hiển thị bảng CSV gốc trên giao diện.
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

/// Đọc toàn bộ dữ liệu import từ file CSV: work records + raw CSV.
/// Gộp kết quả từ `read_work_records` và `read_raw_csv` vào một struct.
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

/// Gói dữ liệu đã parse từ file CSV, dùng để truyền giữa các service.
pub struct CsvImportData {
    /// Đường dẫn canonical tới file CSV nguồn.
    pub source_path: String,
    /// Tên file CSV (chỉ tên, không có đường dẫn).
    pub source_file_name: String,
    /// Danh sách bản ghi công việc đã parse.
    pub records: Vec<WorkRecord>,
    /// Tổng số phút của toàn bộ file.
    pub total_minutes: i64,
    /// Dữ liệu thô (headers + rows + chỉ số cột phút).
    pub raw_csv: RawCsv,
}
