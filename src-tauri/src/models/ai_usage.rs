//! Model cho module AI Usage (quản lý account AI tạm thời).

use serde::{Deserialize, Serialize};

/// Một account AI được đăng ký (lưu tạm trong bộ nhớ).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiAccount {
    pub id: i64,
    pub name: String,
    /// API key đã che bớt (chỉ hiện vài ký tự cuối) — trả về frontend để hiển thị.
    pub api_key_masked: String,
    /// Loại tài khoản, tự detect từ prefix của API key (api / admin / oauth / unknown).
    pub account_type: String,
    /// Phần trăm usage còn lại (0–100).
    pub usage_percent: f64,
    /// Thời điểm reset usage tiếp theo, format `YYYY-MM-DD HH:MM:SS`.
    pub reset_at: String,
    /// Số session đã mở với account này.
    pub session_count: i32,
    pub created_at: String,
}

/// Request thêm account AI từ frontend.
#[derive(Debug, Deserialize)]
pub struct AddAiAccountRequest {
    pub name: String,
    pub api_key: String,
}
