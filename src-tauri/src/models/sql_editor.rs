//! Model cho màn hình SQL Editor.
//!
//! Bao gồm cấu hình kết nối (nhiều kết nối, nhiều loại DB) và kết quả chạy query.
//! Danh sách kết nối được lưu cục bộ dạng JSON — mật khẩu không đẩy lên DB dùng chung.

use serde::{Deserialize, Serialize};

/// Một cấu hình kết nối database do người dùng lưu lại.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqlConnection {
    pub id: i64,
    /// Tên hiển thị của kết nối (ví dụ: "Prod PG", "Local dev").
    pub name: String,
    /// Loại DB: `postgres` (mysql/sqlite/... để mở rộng sau).
    pub db_type: String,
    pub host: String,
    pub port: u16,
    /// Tên database.
    pub database: String,
    pub username: String,
    pub password: String,
    /// Thời điểm tạo (`YYYY-MM-DD HH:MM:SS`).
    pub created_at: String,
}

/// Request thêm mới / cập nhật một kết nối từ frontend.
///
/// `id = 0` (hoặc bỏ trống) → thêm mới; `id > 0` → cập nhật kết nối đã có.
#[derive(Debug, Deserialize)]
pub struct SaveSqlConnectionRequest {
    #[serde(default)]
    pub id: i64,
    pub name: String,
    #[serde(default = "default_db_type")]
    pub db_type: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

fn default_db_type() -> String {
    "postgres".to_string()
}

/// Một bảng trong database (kèm danh sách cột) — dùng cho autocomplete.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbTable {
    pub schema: String,
    pub name: String,
    pub columns: Vec<String>,
}

/// Một tập kết quả trả về sau khi chạy query.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult {
    /// Tên các cột (rỗng nếu câu lệnh không trả về bảng, ví dụ INSERT/UPDATE).
    pub columns: Vec<String>,
    /// Dữ liệu từng dòng — mỗi ô là chuỗi hoặc `null`.
    pub rows: Vec<Vec<Option<String>>>,
    /// Số dòng trả về (với SELECT).
    pub row_count: i64,
    /// Số dòng bị ảnh hưởng (với INSERT/UPDATE/DELETE).
    pub affected: i64,
    /// Thời gian thực thi (mili-giây).
    pub execution_ms: i64,
    /// `true` nếu câu lệnh có trả về bảng kết quả.
    pub has_result_set: bool,
}
