use crate::models::system::SystemInfo;
use crate::services::system_service;

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    system_service::get_system_info()
}
