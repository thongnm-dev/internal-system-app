//! Model thông tin hệ thống của máy đang chạy ứng dụng.

use serde::Serialize;

/// Thông tin hệ thống, hiển thị trên footer hoặc trang Settings.
#[derive(Serialize)]
pub struct SystemInfo {
    /// Tên user đang đăng nhập hệ điều hành.
    pub username: String,
    /// Thời điểm lấy thông tin (format: YYYY-MM-DD HH:MM:SS).
    pub timestamp: String,
    /// Địa chỉ IP local của máy.
    pub ip_address: String,
    /// Phiên bản ứng dụng (từ Cargo.toml).
    pub version: String,
}
