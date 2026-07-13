//! Tầng truy cập dữ liệu (data access layer) cho module ghi chú công việc hằng ngày.
//!
//! Tất cả các thao tác đều gọi stored procedure qua PostgreSQL.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::daily_note::{DailyNoteDateCount, DailyWorkNote};
use crate::utils::pgsql_connect;

/// Thêm ghi chú công việc mới.
///
/// Gọi `sp_daily_note_insert` và trả về bản ghi vừa tạo
/// (bao gồm `id` và `created_at` được sinh bởi database).
pub async fn insert(
    username: &str,
    content: &str,
    note_date: &str,
    status: &str,
) -> AppResult<DailyWorkNote> {
    let client = pgsql_connect::connect().await?;
    let date = parse_date(note_date)?;

    let row = client
        .query_one(
            "SELECT * FROM sp_daily_note_insert($1, $2, $3, $4)",
            &[&username, &content, &date, &status],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to insert daily note: {e}")))?;

    Ok(row_to_note(&row))
}

/// Lấy tất cả ghi chú của một user trong một ngày cụ thể.
/// Kết quả sắp xếp theo `created_at` giảm dần (mới nhất trước).
pub async fn select_by_date(username: &str, note_date: &str) -> AppResult<Vec<DailyWorkNote>> {
    let client = pgsql_connect::connect().await?;
    let date = parse_date(note_date)?;

    let rows = client
        .query(
            "SELECT * FROM sp_daily_note_select_by_date($1, $2)",
            &[&username, &date],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query daily notes: {e}")))?;

    Ok(rows.iter().map(row_to_note).collect())
}

/// Lấy tất cả ghi chú của một user trong một tháng.
/// Dùng khi cần hiển thị danh sách chi tiết toàn bộ tháng.
pub async fn select_by_month(
    username: &str,
    year: i32,
    month: i32,
) -> AppResult<Vec<DailyWorkNote>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_daily_note_select_by_month($1, $2, $3)",
            &[&username, &year, &month],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query daily notes: {e}")))?;

    Ok(rows.iter().map(row_to_note).collect())
}

/// Đếm số ghi chú theo từng ngày trong tháng.
/// Dùng để hiển thị badge (số lượng) trên các ô lịch.
pub async fn count_by_month(
    username: &str,
    year: i32,
    month: i32,
) -> AppResult<Vec<DailyNoteDateCount>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_daily_note_count_by_month($1, $2, $3)",
            &[&username, &year, &month],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to count daily notes: {e}")))?;

    Ok(rows
        .iter()
        .map(|row| DailyNoteDateCount {
            note_date: row.get("note_date"),
            note_count: row.get("note_count"),
        })
        .collect())
}

/// Cập nhật trạng thái của một ghi chú (completed / incomplete / reserved).
///
/// Kiểm tra cả `id` và `username` để đảm bảo user chỉ sửa note của mình.
/// Trả về bản ghi sau khi cập nhật, hoặc lỗi nếu không tìm thấy.
pub async fn update_status(
    id: i32,
    username: &str,
    status: &str,
) -> AppResult<DailyWorkNote> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt(
            "SELECT * FROM sp_daily_note_update_status($1, $2, $3)",
            &[&id, &username, &status],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update daily note status: {e}")))?
        .ok_or_else(|| AppError::new(format!("Daily note '{}' not found.", id)))?;

    Ok(row_to_note(&row))
}

/// Xóa một ghi chú theo ID và username.
/// Trả về `true` nếu xóa thành công, `false` nếu không tìm thấy.
pub async fn delete(id: i32, username: &str) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_daily_note_delete($1, $2)",
            &[&id, &username],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to delete daily note: {e}")))?;

    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}

/// Chuyển đổi một row từ kết quả stored procedure thành `DailyWorkNote`.
fn row_to_note(row: &tokio_postgres::Row) -> DailyWorkNote {
    DailyWorkNote {
        id: row.get("id"),
        username: row.get("username"),
        content: row.get("content"),
        note_date: row.get("note_date"),
        status: row.get("status"),
        created_at: row.get("created_at"),
    }
}

/// Parse chuỗi ngày (YYYY-MM-DD) thành `NaiveDate` để truyền vào PostgreSQL.
fn parse_date(value: &str) -> AppResult<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|e| AppError::new(format!("Invalid date '{}': {e}", value)))
}
