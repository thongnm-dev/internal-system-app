//! Model cho module AI Usage (quản lý account AI + theo dõi usage + auto-switch).

use serde::{Deserialize, Serialize};

/// Một account AI được đăng ký — bản trả về frontend (API key đã che).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiAccount {
    pub id: i64,
    pub name: String,
    /// API key đã che bớt (chỉ hiện vài ký tự cuối) — trả về frontend để hiển thị.
    pub api_key_masked: String,
    /// Loại tài khoản, tự detect từ prefix của API key (api / admin / oauth / unknown).
    pub account_type: String,
    /// Nhà cung cấp: `claude` | `codex`. Auto-switch chỉ diễn ra trong cùng provider.
    pub provider: String,
    /// Thứ tự ưu tiên — số nhỏ = ưu tiên cao hơn.
    pub priority: i32,
    /// `true` nếu account đang được chọn (active) cho provider của nó.
    pub is_active: bool,
    /// Trạng thái: `healthy` | `low` | `exhausted` | `error` | `unknown`.
    pub status: String,
    /// Nguồn số liệu usage: `billing_api` | `ratelimit_header` | `error_signal` | `manual` | `unknown`.
    pub usage_source: String,
    /// Phần trăm usage còn lại (0–100).
    pub usage_percent: f64,
    /// Thời điểm reset usage tiếp theo, format `YYYY-MM-DD HH:MM:SS` (hoặc rỗng nếu chưa rõ).
    pub reset_at: String,
    /// Số session đã mở với account này.
    pub session_count: i32,
    /// Lần probe usage gần nhất (`YYYY-MM-DD HH:MM:SS`, rỗng nếu chưa probe).
    pub last_checked_at: String,
    pub created_at: String,
}

/// Request thêm account AI từ frontend.
#[derive(Debug, Deserialize)]
pub struct AddAiAccountRequest {
    pub name: String,
    pub api_key: String,
    /// `claude` | `codex`. Mặc định `claude` nếu bỏ trống.
    pub provider: Option<String>,
    /// Ưu tiên; bỏ trống → tự đặt xuống cuối danh sách của provider.
    pub priority: Option<i32>,
}

/// Request cập nhật thông tin account (không đổi API key).
#[derive(Debug, Deserialize)]
pub struct UpdateAiAccountRequest {
    pub id: i64,
    pub name: Option<String>,
    pub provider: Option<String>,
    pub priority: Option<i32>,
}

/// Tín hiệu usage do skill/automation báo về (ví dụ: dính "usage limit reached").
#[derive(Debug, Deserialize)]
pub struct ReportUsageSignalRequest {
    pub id: i64,
    /// `true` = account vừa hết usage (đánh dấu `exhausted` + auto-switch).
    pub exhausted: bool,
    /// Thời điểm reset (tuỳ chọn), format `YYYY-MM-DD HH:MM:SS`.
    pub reset_at: Option<String>,
}

/// Cấu hình cho auto-switch và poll nền.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiUsageSettings {
    /// Ngưỡng phần trăm còn lại để coi là "low" và ưu tiên switch sang account khác.
    pub switch_threshold_percent: f64,
    /// Chu kỳ poll usage nền (giây).
    pub poll_interval_secs: u64,
}

impl Default for AiUsageSettings {
    fn default() -> Self {
        Self {
            switch_threshold_percent: 10.0,
            poll_interval_secs: 60,
        }
    }
}
