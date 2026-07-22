use crate::models::sync::{SyncDailyReportRequest, SyncResult};
use crate::services::sync_service;

#[tauri::command]
pub async fn sync_daily_report(
    app: tauri::AppHandle,
    request: SyncDailyReportRequest,
) -> Result<SyncResult, String> {
    sync_service::sync_daily_report(app, request)
        .await
        .map_err(crate::app::error::log_err)
}
