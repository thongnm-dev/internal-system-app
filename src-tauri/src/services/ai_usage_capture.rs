//! Capture login Claude **đang active** trên máy để lưu thành profile riêng.
//!
//! Đọc blob `claudeAiOauth` từ Keychain mặc định (`Claude Code-credentials`) + định
//! danh từ `~/.claude.json`. Blob token được ghi vào app data dir (xem
//! [`crate::database::ai_profile_store`]); phần định danh dùng để tạo account.

use chrono::{Local, TimeZone};
use serde_json::Value;

use crate::models::ai_usage::CapturedLogin;

/// Dữ liệu capture đầy đủ (kèm blob token) — chỉ dùng nội bộ backend.
pub struct Captured {
    pub email: String,
    pub display_name: String,
    pub subscription_type: String,
    pub billing_type: String,
    pub expires_at_ms: Option<i64>,
    /// Blob `claudeAiOauth` nguyên vẹn (accessToken, refreshToken, expiresAt, scopes…).
    pub claude_ai_oauth: Value,
}

/// Capture login Claude đang active. `None` nếu không có login (chưa `claude /login`).
pub fn capture_current() -> Option<Captured> {
    let oauth = read_default_keychain_oauth()?;
    // Phải có access token thì mới coi là login hợp lệ.
    let access_token = oauth.get("accessToken").and_then(Value::as_str).unwrap_or("");
    if access_token.trim().is_empty() {
        return None;
    }

    let subscription_type = oauth
        .get("subscriptionType")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let expires_at_ms = oauth.get("expiresAt").and_then(Value::as_i64);

    let identity = read_default_identity();
    let (email, display_name, billing_type, org_type) = identity.unwrap_or_default();

    Some(Captured {
        email,
        display_name,
        subscription_type: if subscription_type.is_empty() {
            org_type
        } else {
            subscription_type
        },
        billing_type,
        expires_at_ms,
        claude_ai_oauth: oauth,
    })
}

/// Bản xem trước cho frontend (không kèm token thật).
pub fn preview() -> Option<CapturedLogin> {
    capture_current().map(|c| CapturedLogin {
        email: c.email,
        display_name: c.display_name,
        subscription_type: c.subscription_type,
        billing_type: c.billing_type,
        token_expires_at: c.expires_at_ms.map(format_epoch_ms).unwrap_or_default(),
        has_token: true,
    })
}

/// Đọc blob `claudeAiOauth` (dạng JSON Value) từ credential store mặc định.
fn read_default_keychain_oauth() -> Option<Value> {
    let text = crate::services::ai_usage_detect::read_credential_blob("")?;
    let blob: Value = serde_json::from_str(&text).ok()?;
    blob.get("claudeAiOauth").cloned()
}

/// Đọc `oauthAccount` từ `~/.claude.json` → (email, display_name, billing_type, org_type).
fn read_default_identity() -> Option<(String, String, String, String)> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let path = std::path::PathBuf::from(home).join(".claude.json");
    read_identity_file(&path)
}

/// Đọc `oauthAccount` từ một file `.claude.json` → (email, display_name, billing_type, org_type).
fn read_identity_file(path: &std::path::Path) -> Option<(String, String, String, String)> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;
    let account = json.get("oauthAccount")?;
    let get = |key: &str| {
        account
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    Some((
        get("emailAddress"),
        get("displayName"),
        get("billingType"),
        get("organizationType"),
    ))
}

/// Đọc định danh login Claude tại một `CLAUDE_CONFIG_DIR` bất kỳ (đăng ký thủ công
/// account thứ 2 đã `CLAUDE_CONFIG_DIR=<dir> claude /login`). `None` nếu dir chưa có login.
pub fn read_login_at(config_dir: &str) -> Option<CapturedLogin> {
    let config_file = expand_tilde(config_dir).join(".claude.json");
    let (email, display_name, billing_type, org_type) = read_identity_file(&config_file)?;
    if email.trim().is_empty() {
        return None;
    }
    let has_token = crate::services::ai_usage_detect::read_oauth_token(config_dir).is_some();
    Some(CapturedLogin {
        email,
        display_name,
        subscription_type: org_type,
        billing_type,
        token_expires_at: String::new(),
        has_token,
    })
}

/// Mở rộng `~` đầu đường dẫn thành thư mục home.
fn expand_tilde(path: &str) -> std::path::PathBuf {
    let trimmed = path.trim();
    let home = || std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    if trimmed == "~" {
        if let Some(h) = home() {
            return std::path::PathBuf::from(h);
        }
    }
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(h) = home() {
            return std::path::PathBuf::from(h).join(rest);
        }
    }
    std::path::PathBuf::from(trimmed)
}

/// Epoch millis → `YYYY-MM-DD HH:MM:SS` theo timezone local.
fn format_epoch_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let nanos = (ms.rem_euclid(1000) * 1_000_000) as u32;
    match Local.timestamp_opt(secs, nanos).single() {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => String::new(),
    }
}
