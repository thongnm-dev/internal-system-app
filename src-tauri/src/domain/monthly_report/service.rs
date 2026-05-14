use crate::app::{error::AppError, result::AppResult};
use crate::domain::import_csv::models::WorkRecord;
use crate::domain::import_csv::service::{build_preview_rows, CsvImportData};
use crate::domain::monthly_report::models::{
    ImportBatchDetail, ImportBatchListItem, ImportBatchSearchCriteria, ImportBatchSummary, ImportCsvResult,
};
use crate::infrastructure::database::connection::open_database;
use crate::utils::time::current_timestamp;
use rusqlite::{params, params_from_iter, OptionalExtension};
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
    let report_name = trimmed_filter(report_name).unwrap_or_else(|| source_file_name.trim_end_matches(".csv").to_string());
    let note = trimmed_filter(note).unwrap_or_default();
    let imported_at = current_timestamp();
    let imported_by = current_username();
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

pub fn get_import_batch_detail(app_data_dir: &Path, batch_id: i64) -> AppResult<ImportBatchDetail> {
    let connection = open_database(app_data_dir)?;
    let batch = connection
        .query_row(
            "
            SELECT
                id,
                report_name,
                note,
                source_path,
                source_file_name,
                imported_at,
                imported_by,
                target_month_from,
                target_month_to,
                row_count,
                total_minutes
            FROM import_batches
            WHERE id = ?1
            ",
            params![batch_id],
            |row| {
                Ok(ImportBatchDetail {
                    id: row.get(0)?,
                    report_name: row.get(1)?,
                    note: row.get(2)?,
                    source_path: row.get(3)?,
                    source_file_name: row.get(4)?,
                    imported_at: row.get(5)?,
                    imported_by: row.get(6)?,
                    target_month_from: row.get(7)?,
                    target_month_to: row.get(8)?,
                    row_count: row.get(9)?,
                    total_minutes: row.get(10)?,
                    preview_rows: Vec::new(),
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::new(format!("Import report #{batch_id} was not found.")))?;

    let mut statement = connection.prepare(
        "
        SELECT
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
            late_night_overtime_minutes
        FROM monthly_report_rows
        WHERE batch_id = ?1
        ORDER BY id
        ",
    )?;
    let rows = statement.query_map(params![batch_id], |row| {
        Ok(WorkRecord {
            date: row.get(0)?,
            project_code: row.get(1)?,
            project_name: row.get(2)?,
            process_code: row.get(3)?,
            process_name: row.get(4)?,
            phase_name: row.get(5)?,
            work_content: row.get(6)?,
            totals: crate::domain::import_csv::models::MinuteTotals {
                regular_minutes: row.get(7)?,
                normal_overtime_minutes: row.get(8)?,
                legal_holiday_overtime_minutes: row.get(9)?,
                legal_public_holiday_overtime_minutes: row.get(10)?,
                late_night_overtime_minutes: row.get(11)?,
            },
        })
    })?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }

    Ok(ImportBatchDetail {
        preview_rows: build_preview_rows(&records),
        ..batch
    })
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
