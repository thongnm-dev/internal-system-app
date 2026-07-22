//! Tauri command handlers cho cấu hình ứng dụng (config.ini) và quản lý Store Procedure.

use crate::models::app_config::{AppConfigData, SaveAppConfigRequest, SpExecutionResult, SpExecutionSummary, SpInfo};
use crate::services::{app_config_service, sp_management_service};

#[tauri::command]
pub fn get_app_config() -> Result<AppConfigData, String> {
    app_config_service::get_app_config().map_err(crate::app::error::log_err)
}

#[tauri::command]
pub fn save_app_config(request: SaveAppConfigRequest) -> Result<AppConfigData, String> {
    app_config_service::save_app_config(request).map_err(crate::app::error::log_err)
}

#[tauri::command]
pub fn get_stored_procedure_content(name: String) -> Result<String, String> {
    sp_management_service::get_procedure_content(&name)
        .ok_or_else(|| format!("Stored procedure '{}' not found", name))
}

#[tauri::command]
pub fn list_stored_procedures() -> Vec<SpInfo> {
    sp_management_service::list_procedures()
}

#[tauri::command]
pub async fn execute_single_stored_procedure(name: String) -> Result<SpExecutionResult, String> {
    sp_management_service::execute_single_procedure(&name)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn execute_stored_procedures() -> Result<SpExecutionSummary, String> {
    sp_management_service::execute_all_procedures()
        .await
        .map_err(crate::app::error::log_err)
}
