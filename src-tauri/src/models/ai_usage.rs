//! Model cho module AI Usage (quản lý account AI + theo dõi usage + auto-switch).

use serde::{Deserialize, Serialize};

/// Một account AI được đăng ký — bản trả về frontend (API key đã che).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiAccount {
    pub id: i64,
    pub name: String,
    /// API key đã che bớt (chỉ hiện vài ký tự cuối) — trả về frontend để hiển thị.
    pub api_key_masked: String,
    /// Loại tài khoản: `api` | `admin` | `oauth` | `subscription` | `unknown`.
    /// Với account API key thì tự detect từ prefix; `subscription` do frontend chỉ định.
    pub account_type: String,
    /// Nhà cung cấp: `claude` | `codex`. Auto-switch chỉ diễn ra trong cùng provider.
    pub provider: String,
    /// Thư mục `CLAUDE_CONFIG_DIR` nơi account subscription đã đăng nhập (rỗng với account API key).
    pub config_dir: String,
    /// Email tài khoản (lấy từ `.claude.json` khi detect login local; rỗng nếu chưa rõ).
    pub email: String,
    /// Loại subscription (`team` | `claude_pro` | `max` …) khi là account subscription.
    pub subscription_type: String,
    /// Nguồn account: `detected` (dò từ login local, token trong Keychain — luôn mới),
    /// `captured` (tool tự capture token → profile trong app data dir), `manual` (API key/nhập tay).
    pub source: String,
    /// Thứ tự ưu tiên — số nhỏ = ưu tiên cao hơn.
    pub priority: i32,
    /// `true` nếu account đang được chọn (active) cho provider của nó.
    pub is_active: bool,
    /// Trạng thái: `healthy` | `low` | `exhausted` | `error` | `unknown`.
    pub status: String,
    /// Nguồn số liệu usage: `billing_api` | `ratelimit_header` | `error_signal` | `manual` | `unknown`.
    pub usage_source: String,
    /// Phần trăm usage còn lại (0–100). Với subscription = min(session, weekly).
    pub usage_percent: f64,
    /// Thời điểm reset usage tiếp theo, format `YYYY-MM-DD HH:MM:SS` (hoặc rỗng nếu chưa rõ).
    pub reset_at: String,
    /// Session hiện tại (cửa sổ 5 giờ) — phần trăm CÒN LẠI (0–100).
    pub session_percent: f64,
    /// Thời điểm reset session (`YYYY-MM-DD HH:MM:SS`, rỗng nếu chưa có số liệu).
    pub session_reset_at: String,
    /// Weekly limit (cửa sổ 7 ngày) — phần trăm CÒN LẠI (0–100).
    pub weekly_percent: f64,
    /// Thời điểm reset weekly (`YYYY-MM-DD HH:MM:SS`, rỗng nếu chưa có số liệu).
    pub weekly_reset_at: String,
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
    /// API key. Bỏ trống với account subscription (đăng nhập qua `claude /login`).
    pub api_key: Option<String>,
    /// `claude` | `codex`. Mặc định `claude` nếu bỏ trống.
    pub provider: Option<String>,
    /// Loại account chỉ định rõ (vd `subscription`); bỏ trống → tự detect từ key.
    pub account_type: Option<String>,
    /// Thư mục `CLAUDE_CONFIG_DIR` cho account subscription.
    pub config_dir: Option<String>,
    /// Email account (điền khi thêm từ detect login local).
    pub email: Option<String>,
    /// Loại subscription (điền khi thêm từ detect login local).
    pub subscription_type: Option<String>,
    /// Nguồn account (`detected` | `captured` | `manual`). Bỏ trống → `manual`.
    pub source: Option<String>,
    /// Ưu tiên; bỏ trống → tự đặt xuống cuối danh sách của provider.
    pub priority: Option<i32>,
}

/// Login Claude hiện đang active trên máy (bản xem trước để capture — KHÔNG kèm token).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapturedLogin {
    pub email: String,
    pub display_name: String,
    pub subscription_type: String,
    pub billing_type: String,
    /// Thời điểm token hết hạn (`YYYY-MM-DD HH:MM:SS`, rỗng nếu không đọc được).
    pub token_expires_at: String,
    /// Có đọc được OAuth token trong Keychain không.
    pub has_token: bool,
}

/// Một login Claude phát hiện được trên máy (đọc từ `.claude.json` + Keychain).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectedLogin {
    /// Thư mục config (data dir) của login này, vd `~/.claude` hoặc `<CLAUDE_CONFIG_DIR>`.
    pub config_dir: String,
    /// Email tài khoản.
    pub email: String,
    /// Tên hiển thị.
    pub display_name: String,
    /// Loại subscription (`team` | `claude_pro` | `max` …).
    pub subscription_type: String,
    /// Kiểu thanh toán (`apple_subscription` | `stripe` …).
    pub billing_type: String,
    /// Thời điểm token hết hạn (`YYYY-MM-DD HH:MM:SS`, rỗng nếu không đọc được Keychain).
    pub token_expires_at: String,
    /// Đã có trong danh sách account chưa (khớp theo email hoặc config_dir).
    pub already_added: bool,
}

/// Request cập nhật thông tin account (không đổi API key).
#[derive(Debug, Deserialize)]
pub struct UpdateAiAccountRequest {
    pub id: i64,
    pub name: Option<String>,
    pub provider: Option<String>,
    pub priority: Option<i32>,
    /// Đổi thư mục `CLAUDE_CONFIG_DIR` của account subscription.
    pub config_dir: Option<String>,
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
            poll_interval_secs: 300,
        }
    }
}
