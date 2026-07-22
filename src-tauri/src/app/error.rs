//! Kiểu lỗi chung (AppError) cho toàn bộ ứng dụng.
//!
//! Hỗ trợ chuyển đổi tự động từ các lỗi thư viện phổ biến
//! (`io::Error`, `csv::Error`, `serde_json::Error`) thông qua `From` trait.

use std::fmt::{Debug, Display, Formatter};

/// Kiểu lỗi thống nhất cho toàn bộ tầng business logic và data access.
///
/// Chứa một `message` dạng chuỗi, có thể hiển thị trực tiếp cho người dùng
/// hoặc trả về frontend qua IPC.
#[derive(Debug)]
pub struct AppError {
    /// Nội dung mô tả lỗi, có thể hiển thị trực tiếp cho người dùng.
    message: String,
}

impl AppError {
    /// Tạo lỗi mới từ bất kỳ giá trị nào có thể chuyển thành `String`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Hiển thị nội dung lỗi dạng text (dùng cho `.to_string()` và logging).
impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

/// Chuyển đổi tự động từ `std::io::Error` (lỗi đọc/ghi file, network I/O).
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

/// Chuyển đổi tự động từ `csv::Error` (lỗi parse file CSV).
impl From<csv::Error> for AppError {
    fn from(error: csv::Error) -> Self {
        Self::new(error.to_string())
    }
}

/// Chuyển đổi tự động từ `serde_json::Error` (lỗi serialize/deserialize JSON).
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}

/// Ghi log lỗi (dạng Debug chi tiết) rồi trả về thông báo sạch (dạng Display)
/// cho frontend — dùng cho `.map_err(log_err)` ở tầng command.
///
/// Ghi log giữ nguyên `{e:?}` (ví dụ `AppError { message: "..." }`) để tiện chẩn đoán,
/// nhưng chuỗi trả về frontend chỉ là nội dung message, không kèm wrapper.
pub fn log_err(e: impl Debug + Display) -> String {
    log::error!("{e:?}");
    e.to_string()
}
