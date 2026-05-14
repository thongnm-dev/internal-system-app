use crate::app::result::AppResult;
use crate::domain::monthly_report_import::models::{
    ImportBatchListItem, ImportBatchSearchCriteria, ImportBatchSummary, ImportCsvPreviewResult,
    ImportCsvResult, ImportPreviewRow,
};
use crate::domain::statistics::service::{
    read_work_records, LATE_NIGHT_OVERTIME, LEGAL_HOLIDAY_OVERTIME, LEGAL_PUBLIC_HOLIDAY_OVERTIME,
    NORMAL_OVERTIME, REGULAR_MINUTES,
};
use crate::infrastructure::csv::reader::{get_required_index, read_shift_jis_csv};
use crate::infrastructure::database::connection::open_database;
use crate::infrastructure::file_system::display_path;
use crate::utils::time::current_timestamp;
use rusqlite::{params, params_from_iter};
use std::env;
use std::path::Path;

pub fn preview_csv(path: &str) -> AppResult<ImportCsvPreviewResult> {
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

    Ok(ImportCsvPreviewResult {
        source_path,
        source_file_name,
        row_count: records.len(),
        total_minutes,
        preview_rows: build_preview_rows(&records),
        raw_headers: raw_csv.headers,
        raw_rows: raw_csv.rows,
        minute_column_indexes: raw_csv.minute_column_indexes,
    })
}

pub fn import_csv_to_database(
    path: &str,
    app_data_dir: &Path,
    report_name: Option<String>,
    note: Option<String>,
) -> AppResult<ImportCsvResult> {
    let (input_path, records) = read_work_records(path)?;
    let raw_csv = read_raw_csv(&input_path)?;
    let source_path = display_path(&input_path);
    let source_file_name = input_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| source_path.clone());
    let report_name = trimmed_filter(report_name).unwrap_or_else(|| source_file_name.trim_end_matches(".csv").to_string());
    let note = trimmed_filter(note).unwrap_or_default();
    let imported_at = current_timestamp();
    let imported_by = current_username();
    let total_minutes: i64 = records
        .iter()
        .map(|record| record.totals.total_minutes())
        .sum();
    let (target_month_from, target_month_to) = target_month_range(&records);

    let mut connection = open_database(app_data_dir)?;
    let transaction = connection.transaction()?;
    transaction.execute(
        "
        INSERT INTO import_batches(
            source_path,
            source_file_name,
            imported_at,
            report_name,
            note,
            target_month_from,
            target_month_to,
            imported_by,
            row_count,
            total_minutes
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ",
        params![
            source_path,
            source_file_name,
            imported_at,
            report_name,
            note,
            target_month_from,
            target_month_to,
            imported_by,
            records.len() as i64,
            total_minutes
        ],
    )?;
    let batch_id = transaction.last_insert_rowid();

    {
        let mut statement = transaction.prepare(
            "
            INSERT INTO monthly_report_rows(
                batch_id,
                work_date,
                project_code,
                project_name,
                process_code,
                process_name,
                phase_name,
                work_content,
                regular_minutes,
                normal_overtime_minutes,
                legal_holiday_overtime_minutes,
                legal_public_holiday_overtime_minutes,
                late_night_overtime_minutes,
                total_minutes
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ",
        )?;

        for record in records.iter() {
            statement.execute(params![
                batch_id,
                record.date,
                record.project_code,
                record.project_name,
                record.process_code,
                record.process_name,
                record.phase_name,
                record.work_content,
                record.totals.regular_minutes,
                record.totals.normal_overtime_minutes,
                record.totals.legal_holiday_overtime_minutes,
                record.totals.legal_public_holiday_overtime_minutes,
                record.totals.late_night_overtime_minutes,
                record.totals.total_minutes(),
            ])?;
        }
    }

    transaction.commit()?;

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
    let connection = open_database(app_data_dir)?;
    let mut statement = connection.prepare(
        "
        SELECT
            id,
            source_file_name,
            imported_at,
            report_name,
            note,
            target_month_from,
            target_month_to,
            imported_by,
            row_count,
            total_minutes
        FROM import_batches
        ORDER BY id DESC
        LIMIT 20
        ",
    )?;

    let rows = statement.query_map([], |row| {
        Ok(ImportBatchSummary {
            id: row.get(0)?,
            source_file_name: row.get(1)?,
            imported_at: row.get(2)?,
            report_name: row.get(3)?,
            note: row.get(4)?,
            target_month_from: row.get(5)?,
            target_month_to: row.get(6)?,
            imported_by: row.get(7)?,
            row_count: row.get(8)?,
            total_minutes: row.get(9)?,
        })
    })?;

    let mut batches = Vec::new();
    for row in rows {
        batches.push(row?);
    }

    Ok(batches)
}

pub fn search_import_batches(
    app_data_dir: &Path,
    criteria: ImportBatchSearchCriteria,
) -> AppResult<Vec<ImportBatchListItem>> {
    let connection = open_database(app_data_dir)?;
    let mut sql = String::from(
        "
        SELECT
            id,
            report_name,
            note,
            source_file_name,
            imported_at,
            imported_by,
            target_month_from,
            target_month_to,
            row_count,
            total_minutes
        FROM import_batches
        WHERE 1 = 1
        ",
    );
    let mut values = Vec::new();

    if let Some(value) = trimmed_filter(criteria.target_month_from) {
        sql.push_str(" AND target_month_to >= ?");
        values.push(value);
    }

    if let Some(value) = trimmed_filter(criteria.target_month_to) {
        sql.push_str(" AND target_month_from <= ?");
        values.push(value);
    }

    if let Some(value) = trimmed_filter(criteria.report_name) {
        sql.push_str(" AND report_name LIKE ?");
        values.push(format!("%{value}%"));
    }

    if let Some(value) = trimmed_filter(criteria.keyword) {
        sql.push_str(
            "
            AND (
                report_name LIKE ?
                OR note LIKE ?
                OR source_file_name LIKE ?
                OR imported_by LIKE ?
            )
            ",
        );
        let keyword = format!("%{value}%");
        values.extend([keyword.clone(), keyword.clone(), keyword.clone(), keyword]);
    }

    sql.push_str(" ORDER BY imported_at DESC, id DESC");

    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(values), |row| {
        Ok(ImportBatchListItem {
            id: row.get(0)?,
            report_name: row.get(1)?,
            note: row.get(2)?,
            source_file_name: row.get(3)?,
            imported_at: row.get(4)?,
            imported_by: row.get(5)?,
            target_month_from: row.get(6)?,
            target_month_to: row.get(7)?,
            row_count: row.get(8)?,
            total_minutes: row.get(9)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    Ok(items)
}

fn build_preview_rows(
    records: &[crate::domain::statistics::models::WorkRecord],
) -> Vec<ImportPreviewRow> {
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

fn target_month_range(
    records: &[crate::domain::statistics::models::WorkRecord],
) -> (String, String) {
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

struct RawCsv {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    minute_column_indexes: Vec<usize>,
}

fn read_raw_csv(input_path: &Path) -> AppResult<RawCsv> {
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
