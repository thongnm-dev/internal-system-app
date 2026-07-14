//! Business logic cho module ghi chú công việc hằng ngày.
//!
//! Xử lý validation dữ liệu đầu vào và delegate sang `daily_note_store`.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::daily_note_store;
use crate::models::daily_note::{CreateDailyNoteRequest, DailyNoteDateCount, DailyWorkNote};

/// Danh sách trạng thái hợp lệ cho ghi chú công việc.
const VALID_STATUSES: [&str; 3] = ["completed", "incomplete", "reserved"];

/// Tạo ghi chú công việc mới.
///
/// Validate:
/// - `content` không được rỗng.
/// - `status` phải nằm trong danh sách hợp lệ.
/// - `note_date` không được rỗng (format YYYY-MM-DD).
pub async fn create_note(
    username: &str,
    request: CreateDailyNoteRequest,
) -> AppResult<DailyWorkNote> {
    let content = request.content.trim().to_string();
    if content.is_empty() {
        return Err(AppError::new("Content is required."));
    }

    let status = request.status.trim().to_lowercase();
    if !VALID_STATUSES.contains(&status.as_str()) {
        return Err(AppError::new(format!(
            "Invalid status '{}'. Must be one of: completed, incomplete, reserved.",
            status
        )));
    }

    if request.note_date.trim().is_empty() {
        return Err(AppError::new("Note date is required."));
    }

    daily_note_store::insert(username, &content, request.note_date.trim(), &status).await
}

/// Lấy danh sách ghi chú của một user trong một ngày cụ thể.
pub async fn get_notes_by_date(
    username: &str,
    note_date: &str,
) -> AppResult<Vec<DailyWorkNote>> {
    daily_note_store::select_by_date(username, note_date).await
}

/// Lấy danh sách ghi chú của một user trong một tháng.
/// Validate: `month` phải từ 1 đến 12.
pub async fn get_notes_by_month(
    username: &str,
    year: i32,
    month: i32,
) -> AppResult<Vec<DailyWorkNote>> {
    if !(1..=12).contains(&month) {
        return Err(AppError::new("Month must be between 1 and 12."));
    }
    daily_note_store::select_by_month(username, year, month).await
}

/// Đếm số ghi chú theo ngày trong tháng (dùng cho badge trên lịch).
/// Validate: `month` phải từ 1 đến 12.
pub async fn get_date_counts(
    username: &str,
    year: i32,
    month: i32,
) -> AppResult<Vec<DailyNoteDateCount>> {
    if !(1..=12).contains(&month) {
        return Err(AppError::new("Month must be between 1 and 12."));
    }
    daily_note_store::count_by_month(username, year, month).await
}

pub async fn update_note_content(
    id: i32,
    username: &str,
    content: &str,
) -> AppResult<DailyWorkNote> {
    let content = content.trim().to_string();
    if content.is_empty() {
        return Err(AppError::new("Content is required."));
    }
    daily_note_store::update_content(id, username, &content).await
}

/// Cập nhật trạng thái của ghi chú công việc.
///
/// Validate `status` phải hợp lệ. Stored procedure kiểm tra cả `username`
/// để đảm bảo user chỉ sửa note của chính mình.
pub async fn update_note_status(
    id: i32,
    username: &str,
    status: &str,
) -> AppResult<DailyWorkNote> {
    let status = status.trim().to_lowercase();
    if !VALID_STATUSES.contains(&status.as_str()) {
        return Err(AppError::new(format!(
            "Invalid status '{}'. Must be one of: completed, incomplete, reserved.",
            status
        )));
    }

    daily_note_store::update_status(id, username, &status).await
}

/// Xóa ghi chú theo ID. Kiểm tra `username` để đảm bảo quyền sở hữu.
/// Trả về lỗi nếu không tìm thấy ghi chú.
pub async fn delete_note(id: i32, username: &str) -> AppResult<()> {
    if !daily_note_store::delete(id, username).await? {
        return Err(AppError::new(format!("Daily note '{}' not found.", id)));
    }
    Ok(())
}
