//! Business logic cho màn hình daily report.
//!
//! Xử lý validation dữ liệu đầu vào và delegate sang `daily_report_store`.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::daily_report_store;
use crate::models::daily_report::{
    CreateDailyReportTaskRequest, DailyReportEntry, DailyReportPhase, DailyReportProject,
    DailyReportTaskHours, DailyReportUserTask, SaveDailyReportEntryRequest,
};

/// Số giờ tối đa cho phép trong một ô nhập (một ngày).
const MAX_HOUR: f64 = 24.0;

// ============================================================================
// Entries
// ============================================================================

/// Lưu (upsert) một ô nhập giờ công.
///
/// Validate:
/// - `task_id`, `project_id`, `entry_date` không được rỗng.
/// - `hour` phải > 0 và <= 24.
/// - Các trường OT được clamp về [0, 24]; nếu `is_ot = false` thì bỏ qua (đặt 0).
pub async fn save_entry(
    username: &str,
    request: SaveDailyReportEntryRequest,
) -> AppResult<DailyReportEntry> {
    let task_id = request.task_id.trim();
    if task_id.is_empty() {
        return Err(AppError::new("Task id is required."));
    }

    let project_id = request.project_id.trim();
    if project_id.is_empty() {
        return Err(AppError::new("Project id is required."));
    }

    if request.entry_date.trim().is_empty() {
        return Err(AppError::new("Entry date is required."));
    }

    if !request.hour.is_finite() || request.hour <= 0.0 {
        return Err(AppError::new("Hour must be greater than 0."));
    }
    let hour = request.hour.min(MAX_HOUR);

    let is_ot = request.is_ot;
    let regular_ot = if is_ot { clamp_hour(request.regular_ot) } else { 0.0 };
    let midnight_ot = if is_ot { clamp_hour(request.midnight_ot) } else { 0.0 };

    daily_report_store::upsert_entry(
        username,
        task_id,
        project_id,
        request.entry_date.trim(),
        request.comment.trim(),
        hour,
        is_ot,
        regular_ot,
        midnight_ot,
        request.phase.trim(),
    )
    .await
}

/// Xóa một ô nhập giờ công (khi người dùng bấm "Clear").
/// Không báo lỗi nếu ô chưa từng được lưu (idempotent).
pub async fn clear_entry(username: &str, task_id: &str, entry_date: &str) -> AppResult<()> {
    if task_id.trim().is_empty() {
        return Err(AppError::new("Task id is required."));
    }
    if entry_date.trim().is_empty() {
        return Err(AppError::new("Entry date is required."));
    }
    daily_report_store::delete_entry(username, task_id.trim(), entry_date.trim()).await?;
    Ok(())
}

/// Lấy toàn bộ ô nhập giờ công của một user trong một tháng.
/// Validate: `month` phải từ 1 đến 12.
pub async fn get_entries_by_month(
    username: &str,
    year: i32,
    month: i32,
) -> AppResult<Vec<DailyReportEntry>> {
    if !(1..=12).contains(&month) {
        return Err(AppError::new("Month must be between 1 and 12."));
    }
    daily_report_store::select_entries_by_month(username, year, month).await
}

// ============================================================================
// User tasks
// ============================================================================

/// Tạo (hoặc cập nhật theo `task_id`) một task người dùng tự thêm.
///
/// Validate:
/// - `task_id`, `project_id`, `name` không được rỗng.
/// - `code` mặc định là category đầu tiên, hoặc "TASK" nếu không có.
pub async fn create_task(
    username: &str,
    request: CreateDailyReportTaskRequest,
) -> AppResult<DailyReportUserTask> {
    let task_id = request.task_id.trim();
    if task_id.is_empty() {
        return Err(AppError::new("Task id is required."));
    }

    let project_id = request.project_id.trim();
    if project_id.is_empty() {
        return Err(AppError::new("Project id is required."));
    }

    let name = request.name.trim();
    if name.is_empty() {
        return Err(AppError::new("Task name is required."));
    }

    let categories: Vec<String> = request
        .categories
        .iter()
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect();

    let code = {
        let trimmed = request.code.trim();
        if !trimmed.is_empty() {
            trimmed.to_string()
        } else {
            categories.first().cloned().unwrap_or_else(|| "TASK".to_string())
        }
    };

    let assignee = {
        let trimmed = request.assignee.trim();
        if trimmed.is_empty() { username } else { trimmed }
    };

    daily_report_store::insert_task(
        username,
        task_id,
        project_id,
        &code,
        name,
        request.description.trim(),
        &categories,
        assignee,
        request.estimate_hour.trim(),
        request.due_date.trim(),
        request.issue_key.trim(),
    )
    .await
}

/// Lấy task người dùng tự thêm của một user theo tháng đang xem.
/// Validate: `month` phải từ 1 đến 12.
pub async fn get_tasks(
    username: &str,
    year: i32,
    month: i32,
) -> AppResult<Vec<DailyReportUserTask>> {
    if !(1..=12).contains(&month) {
        return Err(AppError::new("Month must be between 1 and 12."));
    }
    daily_report_store::select_tasks(username, year, month).await
}

/// Tổng số giờ tích luỹ (mọi tháng) gộp theo task, cho một user.
pub async fn get_task_hours_total(username: &str) -> AppResult<Vec<DailyReportTaskHours>> {
    daily_report_store::select_task_hours_total(username).await
}

/// Đánh dấu một task hoàn thành / bỏ hoàn thành. Trả về lỗi nếu không tìm thấy.
pub async fn set_task_completed(
    username: &str,
    task_id: &str,
    is_completed: bool,
) -> AppResult<DailyReportUserTask> {
    let task_id = task_id.trim();
    if task_id.is_empty() {
        return Err(AppError::new("Task id is required."));
    }
    daily_report_store::set_task_completed(username, task_id, is_completed)
        .await?
        .ok_or_else(|| AppError::new(format!("Daily report task '{}' not found.", task_id)))
}

/// Đánh dấu một project_task delivery / bỏ delivery. Trả về lỗi nếu không tìm thấy.
pub async fn set_project_task_completed(task_id: &str, is_completed: bool) -> AppResult<bool> {
    let task_id = task_id.trim();
    if task_id.is_empty() {
        return Err(AppError::new("Task id is required."));
    }
    daily_report_store::set_project_task_completed(task_id, is_completed)
        .await?
        .ok_or_else(|| AppError::new(format!("Project task '{}' not found.", task_id)))
}

/// Danh sách công đoạn (process/phase) cho dropdown phase khi nhập giờ.
pub async fn get_phases() -> AppResult<Vec<DailyReportPhase>> {
    daily_report_store::select_categories().await
}

/// Xóa một task người dùng tự thêm. Trả về lỗi nếu không tìm thấy.
pub async fn delete_task(username: &str, task_id: &str) -> AppResult<()> {
    if task_id.trim().is_empty() {
        return Err(AppError::new("Task id is required."));
    }
    if !daily_report_store::delete_task(username, task_id.trim()).await? {
        return Err(AppError::new(format!("Daily report task '{}' not found.", task_id)));
    }
    Ok(())
}

// ============================================================================
// Projects
// ============================================================================

/// Lấy danh sách project active kèm cờ is_member cho daily report.
pub async fn get_projects(username: &str) -> AppResult<Vec<DailyReportProject>> {
    daily_report_store::select_projects(username).await
}

/// Giới hạn số giờ về khoảng hợp lệ [0, 24].
fn clamp_hour(value: f64) -> f64 {
    if !value.is_finite() || value < 0.0 {
        0.0
    } else {
        value.min(MAX_HOUR)
    }
}
