//! Tauri command handlers cho module ghi chú công việc hằng ngày.
//!
//! Mỗi hàm là một `#[tauri::command]` async, nhận tham số từ frontend
//! qua IPC invoke, gọi service tương ứng và trả kết quả về frontend.

use crate::models::daily_note::{CreateDailyNoteRequest, DailyNoteDateCount, DailyWorkNote};
use crate::services::daily_note_service;

/// Tạo ghi chú công việc mới cho user.
/// Frontend gửi `username` và `CreateDailyNoteRequest` (content, note_date, status).
#[tauri::command]
pub async fn create_daily_note(
    username: String,
    request: CreateDailyNoteRequest,
) -> Result<DailyWorkNote, String> {
    daily_note_service::create_note(&username, request)
        .await
        .map_err(crate::app::error::log_err)
}

/// Lấy danh sách ghi chú của user trong một ngày cụ thể.
/// `note_date` có định dạng YYYY-MM-DD.
#[tauri::command]
pub async fn get_daily_notes_by_date(
    username: String,
    note_date: String,
) -> Result<Vec<DailyWorkNote>, String> {
    daily_note_service::get_notes_by_date(&username, &note_date)
        .await
        .map_err(crate::app::error::log_err)
}

/// Lấy danh sách ghi chú của user trong một tháng.
/// `year` và `month` (1-12) xác định khoảng thời gian.
#[tauri::command]
pub async fn get_daily_notes_by_month(
    username: String,
    year: i32,
    month: i32,
) -> Result<Vec<DailyWorkNote>, String> {
    daily_note_service::get_notes_by_month(&username, year, month)
        .await
        .map_err(crate::app::error::log_err)
}

/// Đếm số ghi chú theo từng ngày trong tháng.
/// Dùng để hiển thị badge (số lượng công việc) trên ô lịch ở frontend.
#[tauri::command]
pub async fn get_daily_note_counts(
    username: String,
    year: i32,
    month: i32,
) -> Result<Vec<DailyNoteDateCount>, String> {
    daily_note_service::get_date_counts(&username, year, month)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn update_daily_note_content(
    id: i32,
    username: String,
    content: String,
) -> Result<DailyWorkNote, String> {
    daily_note_service::update_note_content(id, &username, &content)
        .await
        .map_err(crate::app::error::log_err)
}

/// Cập nhật trạng thái ghi chú (completed / incomplete / reserved).
/// Kiểm tra `username` để đảm bảo user chỉ sửa note của mình.
#[tauri::command]
pub async fn update_daily_note_status(
    id: i32,
    username: String,
    status: String,
) -> Result<DailyWorkNote, String> {
    daily_note_service::update_note_status(id, &username, &status)
        .await
        .map_err(crate::app::error::log_err)
}

/// Xóa ghi chú theo `id`. Kiểm tra `username` để đảm bảo quyền sở hữu.
#[tauri::command]
pub async fn delete_daily_note(
    id: i32,
    username: String,
) -> Result<(), String> {
    daily_note_service::delete_note(id, &username)
        .await
        .map_err(crate::app::error::log_err)
}
