//! Tầng lưu trữ cục bộ cho danh sách kết nối của màn hình SQL Editor.
//!
//! Danh sách kết nối (kèm mật khẩu gốc) được lưu trong file JSON
//! `sql_connections.json` trong thư mục AppData (`%LOCALAPPDATA%\management-systems`)
//! — KHÔNG đẩy mật khẩu lên database dùng chung.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::app::result::AppResult;
use crate::models::sql_editor::SqlConnection;
use crate::utils::app_config;

/// Tên file dữ liệu cục bộ.
const DATA_FILE: &str = "sql_connections.json";

/// Toàn bộ dữ liệu kết nối được serialize xuống file.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SqlConnectionData {
    #[serde(default)]
    pub connections: Vec<SqlConnection>,
    /// Bộ đếm id tự tăng (bắt đầu từ 1).
    #[serde(default)]
    pub next_id: i64,
}

fn data_path() -> PathBuf {
    app_config::data_dir().join(DATA_FILE)
}

/// Đọc dữ liệu từ file. File chưa tồn tại → trả về mặc định (rỗng).
pub fn load() -> AppResult<SqlConnectionData> {
    let path = data_path();
    if !path.exists() {
        return Ok(SqlConnectionData::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

/// Ghi dữ liệu xuống file (pretty JSON, ghi đè).
pub fn save(data: &SqlConnectionData) -> AppResult<()> {
    let path = data_path();
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, content)?;
    Ok(())
}
