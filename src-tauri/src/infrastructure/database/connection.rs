use crate::app::result::AppResult;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

const DATABASE_FILE: &str = "pjjyuji_statistics.db";

pub fn open_database(app_data_dir: &Path) -> AppResult<Connection> {
    let path = database_path(app_data_dir)?;
    let connection = Connection::open(path)?;
    migrate(&connection)?;
    Ok(connection)
}

pub fn database_path(app_data_dir: &Path) -> AppResult<PathBuf> {
    fs::create_dir_all(app_data_dir)?;
    Ok(app_data_dir.join(DATABASE_FILE))
}

fn migrate(connection: &Connection) -> AppResult<()> {
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS import_batches (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_path TEXT NOT NULL,
            source_file_name TEXT NOT NULL,
            imported_at TEXT NOT NULL,
            row_count INTEGER NOT NULL,
            total_minutes INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS monthly_report_rows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            batch_id INTEGER NOT NULL,
            work_date TEXT NOT NULL,
            project_code TEXT NOT NULL,
            project_name TEXT NOT NULL,
            process_code TEXT NOT NULL,
            process_name TEXT NOT NULL,
            phase_name TEXT NOT NULL,
            work_content TEXT NOT NULL,
            regular_minutes INTEGER NOT NULL,
            normal_overtime_minutes INTEGER NOT NULL,
            legal_holiday_overtime_minutes INTEGER NOT NULL,
            legal_public_holiday_overtime_minutes INTEGER NOT NULL,
            late_night_overtime_minutes INTEGER NOT NULL,
            total_minutes INTEGER NOT NULL,
            FOREIGN KEY(batch_id) REFERENCES import_batches(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_monthly_report_rows_batch_id
            ON monthly_report_rows(batch_id);
        CREATE INDEX IF NOT EXISTS idx_monthly_report_rows_match_key
            ON monthly_report_rows(work_date, project_code, process_code);
        ",
    )?;

    Ok(())
}
