use crate::domain::system::models::SystemInfo;
use crate::domain::system::service;

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    service::get_system_info()
}
