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
    pub priority: i32,
    pub is_active: bool,
    pub status: String,
    pub usage_source: String,
    pub usage_percent: f64,
    pub reset_at: String,
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
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join(DATA_FILE);
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DATA_FILE)
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
