//! Tauri command handlers cho màn hình SQL Editor.

use crate::models::sql_editor::{
    DbTable, QueryResult, SaveSqlConnectionRequest, SqlConnection,
};
use crate::services::sql_editor_service;

/// Liệt kê tất cả kết nối đã lưu.
#[tauri::command]
pub fn sql_list_connections() -> Result<Vec<SqlConnection>, String> {
    sql_editor_service::list_connections().map_err(crate::app::error::log_err)
}

/// Thêm mới hoặc cập nhật một kết nối.
#[tauri::command]
pub fn sql_save_connection(
    request: SaveSqlConnectionRequest,
) -> Result<SqlConnection, String> {
    sql_editor_service::save_connection(request).map_err(crate::app::error::log_err)
}

/// Xoá một kết nối theo id.
#[tauri::command]
pub fn sql_delete_connection(id: i64) -> Result<(), String> {
    sql_editor_service::delete_connection(id).map_err(crate::app::error::log_err)
}

/// Thử kết nối với cấu hình cho trước (không lưu).
#[tauri::command]
pub async fn sql_test_connection(
    request: SaveSqlConnectionRequest,
) -> Result<(), String> {
    sql_editor_service::test_connection(request)
        .await
        .map_err(crate::app::error::log_err)
}

/// Đọc schema (bảng + cột) của kết nối để phục vụ autocomplete.
#[tauri::command]
pub async fn sql_get_schema(connection_id: i64) -> Result<Vec<DbTable>, String> {
    sql_editor_service::get_schema(connection_id)
        .await
        .map_err(crate::app::error::log_err)
}

/// Chạy native query trên kết nối được chọn.
#[tauri::command]
pub async fn sql_run_query(
    connection_id: i64,
    sql: String,
) -> Result<QueryResult, String> {
    sql_editor_service::run_query(connection_id, sql)
        .await
        .map_err(crate::app::error::log_err)
}
