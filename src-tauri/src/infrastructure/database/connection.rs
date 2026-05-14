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
            report_name TEXT NOT NULL DEFAULT '',
            note TEXT NOT NULL DEFAULT '',
            target_month_from TEXT NOT NULL DEFAULT '',
            target_month_to TEXT NOT NULL DEFAULT '',
            imported_by TEXT NOT NULL DEFAULT '',
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

    add_column_if_missing(
        connection,
        "import_batches",
        "report_name",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        connection,
        "import_batches",
        "note",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        connection,
        "import_batches",
        "target_month_from",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        connection,
        "import_batches",
        "target_month_to",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        connection,
        "import_batches",
        "imported_by",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    connection.execute_batch(
        "
        UPDATE import_batches
        SET report_name = source_file_name
        WHERE report_name = '';

        UPDATE import_batches
        SET imported_by = 'unknown'
        WHERE imported_by = '';

        UPDATE import_batches
        SET
            target_month_from = COALESCE((
                SELECT MIN(SUBSTR(REPLACE(work_date, '/', '-'), 1, 7))
                FROM monthly_report_rows
                WHERE monthly_report_rows.batch_id = import_batches.id
            ), ''),
            target_month_to = COALESCE((
                SELECT MAX(SUBSTR(REPLACE(work_date, '/', '-'), 1, 7))
                FROM monthly_report_rows
                WHERE monthly_report_rows.batch_id = import_batches.id
            ), '')
        WHERE target_month_from = '' AND target_month_to = '';
        ",
    )?;

    Ok(())
}

fn add_column_if_missing(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> AppResult<()> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;

    for column in columns {
        if column? == column_name {
            return Ok(());
        }
    }

    connection.execute(
        &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}"),
        [],
    )?;
    Ok(())
}
