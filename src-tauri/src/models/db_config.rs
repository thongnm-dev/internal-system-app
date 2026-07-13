//! Các kiểu dữ liệu (model) cho cấu hình kết nối database.

use serde::{Deserialize, Serialize};

/// Trạng thái cấu hình database, dùng cho màn hình khởi động.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseStatus {
    /// `true` nếu `config.ini` đã có section `[database]` với `dbname`.
    pub configured: bool,
    /// `true` nếu kết nối thử tới database thành công.
    pub connected: bool,
    /// Thông báo lỗi (nếu có) khi kiểm tra/kết nối thất bại.
    pub message: String,
}

/// Thông tin cấu hình database trả về frontend (không kèm mật khẩu rỗng-check).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: String,
}

/// Dữ liệu request từ frontend khi lưu cấu hình database.
#[derive(Debug, Deserialize)]
pub struct SaveDatabaseConfigRequest {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
}
