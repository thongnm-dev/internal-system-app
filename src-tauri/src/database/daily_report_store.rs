//! Tầng truy cập dữ liệu (data access layer) cho màn hình daily report.
//!
//! Tất cả các thao tác đều gọi stored procedure qua PostgreSQL,
//! không viết SQL trực tiếp trong code Rust.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::daily_report::{DailyReportEntry, DailyReportUserTask};
use crate::utils::pgsql_connect;

// ============================================================================
// Entries (giờ công theo task / ngày)
// ============================================================================

/// Lưu (upsert) một ô nhập giờ công theo (username, task_id, entry_date).
/// Trả về bản ghi sau khi lưu.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_entry(
    username: &str,
    task_id: &str,
    project_id: &str,
    entry_date: &str,
    comment: &str,
    hour: f64,
    is_ot: bool,
    regular_ot: f64,
    midnight_ot: f64,
    phase: &str,
) -> AppResult<DailyReportEntry> {
    let client = pgsql_connect::connect().await?;
    let date = parse_date(entry_date)?;

    let row = client
        .query_one(
            "SELECT * FROM sp_daily_report_entry_upsert($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[
                &username,
                &task_id,
                &project_id,
                &date,
                &comment,
                &hour,
                &is_ot,
                &regular_ot,
                &midnight_ot,
                &phase,
            ],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to save daily report entry: {e}")))?;

    Ok(row_to_entry(&row))
}

/// Xóa một ô nhập giờ công theo (username, task_id, entry_date).
/// Trả về `true` nếu có bản ghi bị xóa.
pub async fn delete_entry(username: &str, task_id: &str, entry_date: &str) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;
    let date = parse_date(entry_date)?;

    let row = client
        .query_one(
            "SELECT sp_daily_report_entry_delete($1, $2, $3)",
            &[&username, &task_id, &date],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to delete daily report entry: {e}")))?;

    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}

/// Lấy toàn bộ ô nhập giờ công của một user trong một tháng.
pub async fn select_entries_by_month(
    username: &str,
    year: i32,
    month: i32,
) -> AppResult<Vec<DailyReportEntry>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_daily_report_entry_select_by_month($1, $2, $3)",
            &[&username, &year, &month],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query daily report entries: {e}")))?;

    Ok(rows.iter().map(row_to_entry).collect())
}

// ============================================================================
// User tasks (task do người dùng tự thêm)
// ============================================================================

/// Lưu (upsert) một task người dùng tự thêm.
/// Trả về bản ghi sau khi lưu.
#[allow(clippy::too_many_arguments)]
pub async fn insert_task(
    username: &str,
    task_id: &str,
    project_id: &str,
    code: &str,
    name: &str,
    description: &str,
    categories: &[String],
    assignee: &str,
    estimate_hour: &str,
    due_date: &str,
    issue_key: &str,
) -> AppResult<DailyReportUserTask> {
    let client = pgsql_connect::connect().await?;

    let categories_vec = categories.to_vec();

    let row = client
        .query_one(
            "SELECT * FROM sp_daily_report_task_insert($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            &[
                &username,
                &task_id,
                &project_id,
                &code,
                &name,
                &description,
                &categories_vec,
                &assignee,
                &estimate_hour,
                &due_date,
                &issue_key,
            ],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to save daily report task: {e}")))?;

    Ok(row_to_task(&row))
}

/// Lấy toàn bộ task người dùng tự thêm của một user.
pub async fn select_tasks(username: &str) -> AppResult<Vec<DailyReportUserTask>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_daily_report_task_select($1)", &[&username])
        .await
        .map_err(|e| AppError::new(format!("Failed to query daily report tasks: {e}")))?;

    Ok(rows.iter().map(row_to_task).collect())
}

/// Xóa một task người dùng tự thêm theo (username, task_id).
/// Trả về `true` nếu có bản ghi bị xóa.
pub async fn delete_task(username: &str, task_id: &str) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_daily_report_task_delete($1, $2)",
            &[&username, &task_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to delete daily report task: {e}")))?;

    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}

// ============================================================================
// Helpers
// ============================================================================

/// Chuyển đổi một row thành `DailyReportEntry`.
fn row_to_entry(row: &tokio_postgres::Row) -> DailyReportEntry {
    DailyReportEntry {
        id: row.get("id"),
        username: row.get("username"),
        task_id: row.get("task_id"),
        project_id: row.get("project_id"),
        entry_date: row.get("entry_date"),
        comment: row.get("comment"),
        hour: row.get("hour"),
        is_ot: row.get("is_ot"),
        regular_ot: row.get("regular_ot"),
        midnight_ot: row.get("midnight_ot"),
        phase: row.get("phase"),
        updated_at: row.get("updated_at"),
    }
}

/// Chuyển đổi một row thành `DailyReportUserTask`.
fn row_to_task(row: &tokio_postgres::Row) -> DailyReportUserTask {
    DailyReportUserTask {
        id: row.get("id"),
        username: row.get("username"),
        task_id: row.get("task_id"),
        project_id: row.get("project_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        categories: row.get("categories"),
        assignee: row.get("assignee"),
        estimate_hour: row.get("estimate_hour"),
        due_date: row.get("due_date"),
        issue_key: row.get("issue_key"),
        created_at: row.get("created_at"),
    }
}

/// Parse chuỗi ngày (YYYY-MM-DD) thành `NaiveDate` để truyền vào PostgreSQL.
fn parse_date(value: &str) -> AppResult<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|e| AppError::new(format!("Invalid date '{}': {e}", value)))
}
