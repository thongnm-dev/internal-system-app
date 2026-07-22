use crate::models::schedule::ScheduleResult;
use crate::services::schedule_service;

#[tauri::command]
pub fn read_schedule_excel(
    file_path: String,
    target_year: i32,
    target_month: u32,
    user_filter: Option<String>,
) -> Result<ScheduleResult, String> {
    schedule_service::read_schedule(&file_path, target_year, target_month, &user_filter.unwrap_or_default())
        .map_err(crate::app::error::log_err)
}
