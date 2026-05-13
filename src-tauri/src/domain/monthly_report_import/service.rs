use crate::app::result::AppResult;
use crate::domain::monthly_report_import::models::{
    ImportBatchSummary, ImportCsvPreviewResult, ImportCsvResult, ImportPreviewRow,
};
use crate::domain::statistics::service::read_work_records;
use crate::infrastructure::database::connection::open_database;
use crate::infrastructure::file_system::display_path;
use crate::utils::time::current_timestamp;
use rusqlite::params;
use std::path::Path;

pub fn preview_csv(path: &str) -> AppResult<ImportCsvPreviewResult> {
    let (input_path, records) = read_work_records(path)?;
    let source_path = display_path(&input_path);
    let source_file_name = input_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| source_path.clone());
    let total_minutes: i64 = records.iter().map(|record| record.totals.total_minutes()).sum();

    Ok(ImportCsvPreviewResult {
        source_path,
        source_file_name,
        row_count: records.len(),
        total_minutes,
        preview_rows: build_preview_rows(&records),
    })
}

pub fn import_csv_to_database(path: &str, app_data_dir: &Path) -> AppResult<ImportCsvResult> {
    let (input_path, records) = read_work_records(path)?;
    let source_path = display_path(&input_path);
    let source_file_name = input_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| source_path.clone());
    let imported_at = current_timestamp();
    let total_minutes: i64 = records.iter().map(|record| record.totals.total_minutes()).sum();

    let mut connection = open_database(app_data_dir)?;
    let transaction = connection.transaction()?;
    transaction.execute(
        "
        INSERT INTO import_batches(source_path, source_file_name, imported_at, row_count, total_minutes)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ",
        params![
            source_path,
            source_file_name,
            imported_at,
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
    })
}

pub fn list_import_batches(app_data_dir: &Path) -> AppResult<Vec<ImportBatchSummary>> {
    let connection = open_database(app_data_dir)?;
    let mut statement = connection.prepare(
        "
        SELECT id, source_file_name, imported_at, row_count, total_minutes
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
            row_count: row.get(3)?,
            total_minutes: row.get(4)?,
        })
    })?;

    let mut batches = Vec::new();
    for row in rows {
        batches.push(row?);
    }

    Ok(batches)
}

fn build_preview_rows(records: &[crate::domain::statistics::models::WorkRecord]) -> Vec<ImportPreviewRow> {
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
