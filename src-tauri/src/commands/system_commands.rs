//! Tauri command handlers cho thông tin hệ thống và kiểm tra mạng.

use crate::models::system::SystemInfo;
use crate::services::network_service;
use crate::services::system_service;

/// Lấy thông tin hệ thống: username, timestamp, IP local, phiên bản app.
#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    system_service::get_system_info()
}

/// Kiểm tra máy có kết nối internet hay không.
/// Trả về `true` nếu ít nhất một probe endpoint phản hồi.
#[tauri::command]
pub async fn check_internet_connection() -> bool {
    network_service::check_connection().await
}
