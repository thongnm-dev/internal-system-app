//! Service preview, đọc dữ liệu import CSV, và lưu trữ lịch sử import báo cáo tháng.
//!
//! Cung cấp chức năng preview (xem trước) file CSV trước khi import,
//! đọc dữ liệu CSV, và lưu batch import vào JSON file trong app data directory.

use crate::app::result::AppResult;
use crate::database::csv_reader::{self, CsvImportData};
use crate::database::report_store::{self, StoredImportBatch};
use crate::models::import_csv::{ImportCsvPreviewResult, ImportPreviewRow, WorkRecord};
use crate::models::monthly_report::{ImportBatchSummary, ImportCsvResult};
use crate::utils::time::current_timestamp;
use std::env;
use std::path::Path;

/// Preview nội dung file CSV: parse dữ liệu và trả về kết quả tóm tắt.
/// Frontend sử dụng kết quả này để hiển thị bảng xác nhận trước khi import.
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

/// Đọc dữ liệu import từ file CSV (delegate sang `csv_reader`).
pub fn read_csv_import_data(path: &str) -> AppResult<CsvImportData> {
    csv_reader::read_csv_import_data(path)
}

/// Chuyển danh sách `WorkRecord` thành danh sách `ImportPreviewRow`.
/// Tính tổng phút cho mỗi dòng thay vì giữ chi tiết từng loại giờ.
fn build_preview_rows(records: &[WorkRecord]) -> Vec<ImportPreviewRow> {
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

/// Lưu dữ liệu import CSV thành một batch mới trong JSON store.
///
/// Tự động sinh batch ID, timestamp, username, và xác định khoảng tháng
/// từ dữ liệu work records. Trả về `ImportCsvResult` để frontend hiển thị xác nhận.
pub fn save_imported_report(
    data: CsvImportData,
    app_data_dir: &Path,
    report_name: Option<String>,
    note: Option<String>,
) -> AppResult<ImportCsvResult> {
    let CsvImportData {
        source_path,
        source_file_name,
        records,
        total_minutes,
        raw_csv,
    } = data;
    let report_name = trimmed_filter(report_name)
        .unwrap_or_else(|| source_file_name.trim_end_matches(".csv").to_string());
    let note = trimmed_filter(note).unwrap_or_default();
    let imported_at = current_timestamp();
    let imported_by = current_username();
    let (target_month_from, target_month_to) = target_month_range(&records);

    let mut store = report_store::load_store(app_data_dir)?;
    let batch_id = store.next_id();
    store.batches.push(StoredImportBatch {
        id: batch_id,
        source_path: source_path.clone(),
        source_file_name: source_file_name.clone(),
        imported_at: imported_at.clone(),
        report_name,
        note,
        target_month_from,
        target_month_to,
        imported_by,
        row_count: records.len() as i64,
        total_minutes,
        records: records.clone(),
    });
    report_store::save_store(app_data_dir, &store)?;

    Ok(ImportCsvResult {
        batch_id,
        source_path,
        source_file_name,
        imported_at,
        row_count: records.len(),
        total_minutes,
        preview_rows: build_preview_rows(&records),
        raw_headers: raw_csv.headers,
        raw_rows: raw_csv.rows,
        minute_column_indexes: raw_csv.minute_column_indexes,
    })
}

/// Lấy danh sách 20 batch gần nhất, sắp xếp theo ID giảm dần.
pub fn list_import_batches(app_data_dir: &Path) -> AppResult<Vec<ImportBatchSummary>> {
    let mut batches = report_store::load_store(app_data_dir)?
        .batches
        .into_iter()
        .map(|batch| ImportBatchSummary {
            id: batch.id,
            source_file_name: batch.source_file_name,
            imported_at: batch.imported_at,
            report_name: batch.report_name,
            note: batch.note,
            target_month_from: batch.target_month_from,
            target_month_to: batch.target_month_to,
            imported_by: batch.imported_by,
            row_count: batch.row_count,
            total_minutes: batch.total_minutes,
        })
        .collect::<Vec<_>>();

    batches.sort_by(|left, right| right.id.cmp(&left.id));
    batches.truncate(20);
    Ok(batches)
}

/// Xác định khoảng tháng (from, to) từ danh sách work records.
fn target_month_range(records: &[WorkRecord]) -> (String, String) {
    let mut months: Vec<String> = records
        .iter()
        .filter_map(|record| target_month(&record.date))
        .collect();
    months.sort();
    months.dedup();

    (
        months.first().cloned().unwrap_or_default(),
        months.last().cloned().unwrap_or_default(),
    )
}

/// Trích xuất tháng (YYYY-MM) từ chuỗi ngày.
fn target_month(date: &str) -> Option<String> {
    let normalized = date.trim().replace('/', "-");
    if normalized.len() < 7 {
        return None;
    }

    let month = &normalized[..7];
    if month.chars().enumerate().all(|(index, value)| {
        if index == 4 {
            value == '-'
        } else {
            value.is_ascii_digit()
        }
    }) {
        Some(month.to_string())
    } else {
        None
    }
}

fn trimmed_filter(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn current_username() -> String {
    env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "unknown".to_string())
}
