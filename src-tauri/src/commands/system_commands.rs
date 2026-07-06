use crate::models::system::SystemInfo;
use crate::services::network_service;
use crate::services::system_service;

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    system_service::get_system_info()
}

#[tauri::command]
pub async fn check_internet_connection() -> bool {
    network_service::check_connection().await
}
