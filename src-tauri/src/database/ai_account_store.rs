//! Tầng lưu trữ cục bộ cho module AI Usage.
//!
//! Danh sách account AI (kèm token gốc) và cấu hình được lưu trong một file JSON
//! `ai_accounts.json` đặt cạnh file thực thi (production) hoặc trong thư mục
//! `CARGO_MANIFEST_DIR` (development) — KHÔNG đẩy token lên database dùng chung.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::app::result::AppResult;
use crate::models::ai_usage::AiUsageSettings;

/// Tên file dữ liệu cục bộ.
const DATA_FILE: &str = "ai_accounts.json";

/// Account đầy đủ (giữ API key gốc) — chỉ tồn tại ở tầng backend, không trả ra frontend.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredAccount {
    pub id: i64,
    pub name: String,
    pub api_key: String,
    pub account_type: String,
    pub provider: String,
    /// Thư mục `CLAUDE_CONFIG_DIR` cho account subscription (rỗng với account API key).
    #[serde(default)]
    pub config_dir: String,
    /// Email account (từ detect login local).
    #[serde(default)]
    pub email: String,
    /// Loại subscription (từ detect login local).
    #[serde(default)]
    pub subscription_type: String,
    /// Nguồn account: `detected` | `captured` | `manual`.
    #[serde(default)]
    pub source: String,
    pub priority: i32,
    pub is_active: bool,
    pub status: String,
    pub usage_source: String,
    pub usage_percent: f64,
    pub reset_at: String,
    /// Session hiện tại (cửa sổ 5 giờ) — phần trăm CÒN LẠI (0–100).
    #[serde(default)]
    pub session_percent: f64,
    /// Thời điểm reset session (`YYYY-MM-DD HH:MM:SS`, rỗng nếu chưa có số liệu).
    #[serde(default)]
    pub session_reset_at: String,
    /// Weekly limit (cửa sổ 7 ngày) — phần trăm CÒN LẠI (0–100).
    #[serde(default)]
    pub weekly_percent: f64,
    /// Thời điểm reset weekly (`YYYY-MM-DD HH:MM:SS`, rỗng nếu chưa có số liệu).
    #[serde(default)]
    pub weekly_reset_at: String,
    pub session_count: i32,
    pub last_checked_at: String,
    pub created_at: String,
}

/// Toàn bộ dữ liệu AI Usage được serialize xuống file.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AiAccountData {
    #[serde(default)]
    pub accounts: Vec<StoredAccount>,
    /// Bộ đếm id tự tăng (bắt đầu từ 1).
    #[serde(default)]
    pub next_id: i64,
    #[serde(default)]
    pub settings: AiUsageSettings,
}

/// Xác định đường dẫn tới file `ai_accounts.json`.
///
/// Ưu tiên thư mục chứa file thực thi (production, file có thể ghi cạnh binary);
/// fallback về `CARGO_MANIFEST_DIR` khi không lấy được đường dẫn exe.
fn data_path() -> PathBuf {
    data_dir().join(DATA_FILE)
}

/// Thư mục chứa dữ liệu cục bộ của module AI Usage (cạnh binary ở production,
/// `CARGO_MANIFEST_DIR` khi dev). Dùng chung cho profile token đã capture.
pub fn data_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.to_path_buf();
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Đọc dữ liệu từ file. File chưa tồn tại → trả về mặc định (rỗng).
pub fn load() -> AppResult<AiAccountData> {
    let path = data_path();
    if !path.exists() {
        return Ok(AiAccountData::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

/// Ghi dữ liệu xuống file (pretty JSON, ghi đè).
pub fn save(data: &AiAccountData) -> AppResult<()> {
    let path = data_path();
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, content)?;
    Ok(())
}
