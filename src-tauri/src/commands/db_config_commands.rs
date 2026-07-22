//! Tauri command handlers cho cấu hình kết nối database.

use crate::models::db_config::{DatabaseConfig, DatabaseStatus, SaveDatabaseConfigRequest};
use crate::services::db_config_service;

/// Kiểm tra trạng thái cấu hình database (đã cấu hình chưa, kết nối được không).
/// Được gọi ở màn hình khởi động, sau bước kiểm tra kết nối internet.
#[tauri::command]
pub async fn check_database_status() -> DatabaseStatus {
    db_config_service::check_status().await
}

/// Đọc cấu hình database hiện tại để prefill form. Trả về `null` nếu chưa cấu hình.
#[tauri::command]
pub fn get_database_config() -> Option<DatabaseConfig> {
    db_config_service::get_config()
}

/// Chỉ thử kết nối với cấu hình cho trước, không ghi file (nút "Kiểm tra").
#[tauri::command]
pub async fn test_database_config(
    request: SaveDatabaseConfigRequest,
) -> Result<(), String> {
    db_config_service::test_config(request)
        .await
        .map_err(crate::app::error::log_err)
}

/// Lưu cấu hình database. Thử kết nối trước, ghi `config.ini`, rồi khởi tạo bảng.
#[tauri::command]
pub async fn save_database_config(
    request: SaveDatabaseConfigRequest,
) -> Result<DatabaseStatus, String> {
    db_config_service::save_config(request)
        .await
        .map_err(crate::app::error::log_err)
}
