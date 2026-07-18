//! Tầng lưu trữ cục bộ cho danh sách kết nối của màn hình SQL Editor.
//!
//! Danh sách kết nối (kèm mật khẩu gốc) được lưu trong file JSON
//! `sql_connections.json` đặt cạnh file thực thi (production) hoặc trong thư mục
//! `CARGO_MANIFEST_DIR` (development) — KHÔNG đẩy mật khẩu lên database dùng chung.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::app::result::AppResult;
use crate::models::sql_editor::SqlConnection;

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

/// Xác định đường dẫn tới file `sql_connections.json`.
///
/// Ưu tiên thư mục chứa file thực thi (production, ghi được cạnh binary);
/// fallback về `CARGO_MANIFEST_DIR` khi không lấy được đường dẫn exe.
fn data_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join(DATA_FILE);
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DATA_FILE)
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
