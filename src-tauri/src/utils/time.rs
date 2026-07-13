//! Tiện ích xử lý thời gian.

use chrono::Local;

/// Lấy timestamp hiện tại theo timezone local, format: `YYYY-MM-DD HH:MM:SS`.
pub fn current_timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
