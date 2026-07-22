//! Tauri command handlers cho cấu hình ứng dụng (config.ini) và quản lý Store Procedure.

use crate::models::app_config::{AppConfigData, SaveAppConfigRequest, SpExecutionSummary, SpInfo};
use crate::services::{app_config_service, sp_management_service};

#[tauri::command]
pub fn get_app_config() -> Result<AppConfigData, String> {
    app_config_service::get_app_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_app_config(request: SaveAppConfigRequest) -> Result<AppConfigData, String> {
    app_config_service::save_app_config(request).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_stored_procedures() -> Vec<SpInfo> {
    sp_management_service::list_procedures()
}

#[tauri::command]
pub async fn execute_stored_procedures() -> Result<SpExecutionSummary, String> {
    sp_management_service::execute_all_procedures()
        .await
        .map_err(|e| e.to_string())
}
