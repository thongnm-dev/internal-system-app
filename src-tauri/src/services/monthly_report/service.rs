use crate::app::{error::AppError, result::AppResult};
use crate::database::csv_reader::CsvImportData;
use crate::database::report_store::{self, StoredImportBatch};
use crate::services::import_csv::models::WorkRecord;
use crate::services::import_csv::service::build_preview_rows;
use crate::services::monthly_report::models::{
    ImportBatchDetail, ImportBatchListItem, ImportBatchSearchCriteria, ImportBatchSummary,
    ImportCsvResult,
};
use crate::utils::time::current_timestamp;
use std::env;
use std::path::Path;

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

pub fn search_import_batches(
    app_data_dir: &Path,
    criteria: ImportBatchSearchCriteria,
) -> AppResult<Vec<ImportBatchListItem>> {
    let target_month_from = trimmed_filter(criteria.target_month_from);
    let target_month_to = trimmed_filter(criteria.target_month_to);
    let report_name = trimmed_filter(criteria.report_name);
    let keyword = trimmed_filter(criteria.keyword).map(|value| value.to_lowercase());

    let mut items = report_store::load_store(app_data_dir)?
        .batches
        .into_iter()
        .filter(|batch| {
            target_month_from
                .as_ref()
                .map_or(true, |value| batch.target_month_to >= *value)
        })
        .filter(|batch| {
            target_month_to
                .as_ref()
                .map_or(true, |value| batch.target_month_from <= *value)
        })
        .filter(|batch| {
            report_name
                .as_ref()
                .map_or(true, |value| batch.report_name.contains(value))
        })
        .filter(|batch| {
            keyword.as_ref().map_or(true, |value| {
                contains_case_insensitive(&batch.report_name, value)
                    || contains_case_insensitive(&batch.note, value)
                    || contains_case_insensitive(&batch.source_file_name, value)
                    || contains_case_insensitive(&batch.imported_by, value)
            })
        })
        .map(|batch| ImportBatchListItem {
            id: batch.id,
            report_name: batch.report_name,
            note: batch.note,
            source_file_name: batch.source_file_name,
            imported_at: batch.imported_at,
            imported_by: batch.imported_by,
            target_month_from: batch.target_month_from,
            target_month_to: batch.target_month_to,
            row_count: batch.row_count,
            total_minutes: batch.total_minutes,
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        right
            .imported_at
            .cmp(&left.imported_at)
            .then_with(|| right.id.cmp(&left.id))
    });
    Ok(items)
}

pub fn get_import_batch_detail(app_data_dir: &Path, batch_id: i64) -> AppResult<ImportBatchDetail> {
    let batch = report_store::load_store(app_data_dir)?
        .batches
        .into_iter()
        .find(|batch| batch.id == batch_id)
        .ok_or_else(|| AppError::new(format!("Import report #{batch_id} was not found.")))?;
    let preview_rows = build_preview_rows(&batch.records);

    Ok(ImportBatchDetail {
        id: batch.id,
        report_name: batch.report_name,
        note: batch.note,
        source_path: batch.source_path,
        source_file_name: batch.source_file_name,
        imported_at: batch.imported_at,
        imported_by: batch.imported_by,
        target_month_from: batch.target_month_from,
        target_month_to: batch.target_month_to,
        row_count: batch.row_count,
        total_minutes: batch.total_minutes,
        preview_rows,
    })
}

fn contains_case_insensitive(text: &str, needle_lowercase: &str) -> bool {
    text.to_lowercase().contains(needle_lowercase)
}

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
