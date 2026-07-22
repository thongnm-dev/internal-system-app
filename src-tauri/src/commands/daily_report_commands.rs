//! Tauri command handlers cho màn hình daily report.
//!
//! Mỗi hàm là một `#[tauri::command]` async, nhận tham số từ frontend
//! qua IPC invoke, gọi service tương ứng và trả kết quả về frontend.

use crate::models::daily_report::{
    CreateDailyReportTaskRequest, DailyReportEntry, DailyReportPhase, DailyReportProject,
    DailyReportTaskHours, DailyReportUserTask, SaveDailyReportEntryRequest,
};
use crate::services::daily_report_service;

// ============================================================================
// Entries
// ============================================================================

/// Lưu (upsert) một ô nhập giờ công cho user.
/// Frontend gửi `username` và `SaveDailyReportEntryRequest`.
#[tauri::command]
pub async fn save_daily_report_entry(
    username: String,
    request: SaveDailyReportEntryRequest,
) -> Result<DailyReportEntry, String> {
    daily_report_service::save_entry(&username, request)
        .await
        .map_err(crate::app::error::log_err)
}

/// Xóa một ô nhập giờ công (khi bấm "Clear" trên dialog).
/// `entry_date` có định dạng YYYY-MM-DD.
#[tauri::command]
pub async fn clear_daily_report_entry(
    username: String,
    task_id: String,
    entry_date: String,
    category_id: i32,
) -> Result<(), String> {
    daily_report_service::clear_entry(&username, &task_id, &entry_date, category_id)
        .await
        .map_err(crate::app::error::log_err)
}

/// Lấy toàn bộ ô nhập giờ công của user trong một tháng.
/// `year` và `month` (1-12) xác định khoảng thời gian.
#[tauri::command]
pub async fn get_daily_report_entries(
    username: String,
    year: i32,
    month: i32,
) -> Result<Vec<DailyReportEntry>, String> {
    daily_report_service::get_entries_by_month(&username, year, month)
        .await
        .map_err(crate::app::error::log_err)
}

// ============================================================================
// User tasks
// ============================================================================

/// Tạo (hoặc cập nhật theo `task_id`) một task do người dùng tự thêm.
#[tauri::command]
pub async fn create_daily_report_task(
    username: String,
    request: CreateDailyReportTaskRequest,
) -> Result<DailyReportUserTask, String> {
    daily_report_service::create_task(&username, request)
        .await
        .map_err(crate::app::error::log_err)
}

/// Lấy task người dùng tự thêm của user theo tháng đang xem (`year`, `month` 1-12).
#[tauri::command]
pub async fn get_daily_report_tasks(
    username: String,
    year: i32,
    month: i32,
) -> Result<Vec<DailyReportUserTask>, String> {
    daily_report_service::get_tasks(&username, year, month)
        .await
        .map_err(crate::app::error::log_err)
}

/// Tổng số giờ tích luỹ (mọi tháng) gộp theo task, cho user.
#[tauri::command]
pub async fn get_daily_report_task_hours(
    username: String,
) -> Result<Vec<DailyReportTaskHours>, String> {
    daily_report_service::get_task_hours_total(&username)
        .await
        .map_err(crate::app::error::log_err)
}

/// Đánh dấu một task user tự thêm hoàn thành / bỏ hoàn thành.
#[tauri::command]
pub async fn set_daily_report_task_completed(
    username: String,
    task_id: i32,
    is_completed: bool,
) -> Result<DailyReportUserTask, String> {
    daily_report_service::set_task_completed(&username, task_id, is_completed)
        .await
        .map_err(crate::app::error::log_err)
}

/// Đánh dấu một project_task delivery / bỏ delivery (task được giao cho user).
#[tauri::command]
pub async fn set_project_task_completed(
    task_id: String,
    is_completed: bool,
) -> Result<bool, String> {
    daily_report_service::set_project_task_completed(&task_id, is_completed)
        .await
        .map_err(crate::app::error::log_err)
}

/// Danh sách công đoạn (process/phase) cho dropdown phase khi nhập giờ.
#[tauri::command]
pub async fn get_daily_report_phases() -> Result<Vec<DailyReportPhase>, String> {
    daily_report_service::get_phases()
        .await
        .map_err(crate::app::error::log_err)
}

/// Danh sách category dùng cho project tasks (is_task_category = TRUE).
#[tauri::command]
pub async fn get_task_categories() -> Result<Vec<DailyReportPhase>, String> {
    daily_report_service::get_task_categories()
        .await
        .map_err(crate::app::error::log_err)
}

// ============================================================================
// Projects
// ============================================================================

/// Lấy danh sách project active kèm cờ is_member cho daily report.
#[tauri::command]
pub async fn get_daily_report_projects(
    username: String,
) -> Result<Vec<DailyReportProject>, String> {
    daily_report_service::get_projects(&username)
        .await
        .map_err(crate::app::error::log_err)
}

/// Xóa một task người dùng tự thêm. Kiểm tra `username` để đảm bảo quyền sở hữu.
#[tauri::command]
pub async fn delete_daily_report_task(
    username: String,
    task_id: i32,
) -> Result<(), String> {
    daily_report_service::delete_task(&username, task_id)
        .await
        .map_err(crate::app::error::log_err)
}
