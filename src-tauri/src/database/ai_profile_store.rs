//! Lưu OAuth credential đã **capture** của account subscription vào app data dir.
//!
//! Mỗi account captured có một "profile" riêng:
//! `<data_dir>/ai_profiles/<id>/.credentials.json` với cấu trúc giống credential
//! file của Claude Code (`{ "claudeAiOauth": { accessToken, refreshToken, ... } }`),
//! để (a) probe usage bằng token này và (b) sau này dùng làm `CLAUDE_CONFIG_DIR` khi switch.

use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

use crate::app::result::AppResult;
use crate::database::ai_account_store;

/// Tên thư mục con chứa các profile token.
const PROFILES_DIR: &str = "ai_profiles";
/// Tên file credential trong mỗi profile (giống Claude Code file-based creds).
const CREDENTIALS_FILE: &str = ".credentials.json";

#[derive(Debug, Deserialize)]
struct CredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OauthBlob>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OauthBlob {
    #[serde(default)]
    access_token: Option<String>,
}

/// Thư mục gốc chứa mọi profile.
pub fn profiles_root() -> PathBuf {
    ai_account_store::data_dir().join(PROFILES_DIR)
}

/// Thư mục profile của một account (theo id).
pub fn profile_dir(id: i64) -> PathBuf {
    profiles_root().join(id.to_string())
}

/// Ghi blob `claudeAiOauth` (dạng JSON đã capture) vào profile của account.
/// Trả về đường dẫn thư mục profile (dùng làm `config_dir` của account).
pub fn save_credentials(id: i64, claude_ai_oauth: &Value) -> AppResult<String> {
    let dir = profile_dir(id);
    std::fs::create_dir_all(&dir)?;
    let payload = serde_json::json!({ "claudeAiOauth": claude_ai_oauth });
    let content = serde_json::to_string_pretty(&payload)?;
    std::fs::write(dir.join(CREDENTIALS_FILE), content)?;
    Ok(dir.to_string_lossy().to_string())
}

/// Đọc access token đã capture của một account (nếu có).
pub fn read_access_token(id: i64) -> Option<String> {
    let path = profile_dir(id).join(CREDENTIALS_FILE);
    let content = std::fs::read_to_string(path).ok()?;
    let parsed: CredentialsFile = serde_json::from_str(&content).ok()?;
    parsed
        .claude_ai_oauth?
        .access_token
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Xoá profile của một account (best-effort, bỏ qua nếu không tồn tại).
pub fn delete(id: i64) {
    let _ = std::fs::remove_dir_all(profile_dir(id));
}
