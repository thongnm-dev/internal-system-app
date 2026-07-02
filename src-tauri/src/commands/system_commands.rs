use crate::services::system::models::SystemInfo;
use crate::services::system::service;

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    service::get_system_info()
}
