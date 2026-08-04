//! Dò các login Claude đã tồn tại trên máy.
//!
//! Không có API usage % chính thức cho subscription, nên module này chỉ trích xuất
//! **định danh** (email, tên hiển thị, loại subscription) từ file `.claude.json` của
//! từng config dir, và **thời điểm token hết hạn** từ Keychain macOS — đủ để hiển thị
//! và auto-thêm account vào màn hình AI Usage.
//!
//! Bố cục file của Claude Code:
//! - Mặc định (không set `CLAUDE_CONFIG_DIR`): config = `~/.claude.json`, data = `~/.claude/`,
//!   credential Keychain service = `Claude Code-credentials`.
//! - Custom `CLAUDE_CONFIG_DIR=X`: config = `X/.claude.json`, credential service có hậu tố hash.

use std::path::{Path, PathBuf};

use chrono::{Local, TimeZone};

use crate::database::ai_account_store::StoredAccount;
use crate::models::ai_usage::{ClaudeJson, DetectedLogin, KeychainBlob, OauthAccount};

/// Dò tất cả login Claude phát hiện được: login mặc định (`~/.claude.json`) +
/// các config dir custom đã đăng ký trong danh sách account.
pub fn scan(existing: &[StoredAccount]) -> Vec<DetectedLogin> {
    let mut out: Vec<DetectedLogin> = Vec::new();
    let mut seen: Vec<PathBuf> = Vec::new();

    // 1) Login mặc định.
    if let Some(home) = home_dir() {
        let config_file = home.join(".claude.json");
        let data_dir = home.join(".claude");
        collect(
            &config_file,
            &data_dir.to_string_lossy(),
            existing,
            &mut out,
            &mut seen,
        );
    }

    // 2) Các config dir custom đã đăng ký (nếu có login trong đó).
    for account in existing {
        let dir = account.config_dir.trim();
        if dir.is_empty() {
            continue;
        }
        let config_file = expand_tilde(dir).join(".claude.json");
        collect(&config_file, dir, existing, &mut out, &mut seen);
    }

    out
}

/// Đọc 1 config dir → thêm vào `out` nếu có login hợp lệ (email không rỗng).
fn collect(
    config_file: &Path,
    config_dir_label: &str,
    existing: &[StoredAccount],
    out: &mut Vec<DetectedLogin>,
    seen: &mut Vec<PathBuf>,
) {
    let key = config_file.to_path_buf();
    if seen.contains(&key) {
        return;
    }
    seen.push(key);

    let Some(account) = read_claude_json(config_file) else {
        return;
    };
    let email = account.email_address.trim().to_string();
    if email.is_empty() {
        return;
    }

    let (kc_subscription, expires_at) = read_keychain_meta(config_dir_label);

    let subscription_type = kc_subscription
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| account.organization_type.clone());
    let token_expires_at = expires_at.map(format_epoch_ms).unwrap_or_default();

    // Chỉ coi là "đã thêm" khi đã có bản DETECTED (ưu tiên detected). Bản captured
    // cùng email vẫn để trạng thái "mới" để lần import sẽ nâng cấp thành detected.
    let already_added = existing.iter().any(|a| {
        a.source == "detected"
            && ((!a.email.is_empty() && a.email.eq_ignore_ascii_case(&email))
                || (!a.config_dir.is_empty() && paths_equal(&a.config_dir, config_dir_label)))
    });

    out.push(DetectedLogin {
        config_dir: config_dir_label.to_string(),
        email,
        display_name: account.display_name,
        subscription_type,
        billing_type: account.billing_type,
        token_expires_at,
        already_added,
    });
}

/// Đọc `oauthAccount` từ file `.claude.json`.
fn read_claude_json(path: &Path) -> Option<OauthAccount> {
    let content = std::fs::read_to_string(path).ok()?;
    let parsed: ClaudeJson = serde_json::from_str(&content).ok()?;
    parsed.oauth_account
}

/// Đọc credential store theo từng platform.
///
/// `read_blob` là phần khác nhau giữa các OS (bắt buộc override — xem struct implement ở
/// [`crate::services::claude_credentials_windows::WindowsCredentials`] /
/// [`crate::services::claude_credentials_macos::MacosCredentials`]); các method còn lại
/// (`read_keychain_meta`, `read_oauth_token`) xử lý chung trên blob JSON trả về, viết
/// một lần rồi cả 2 platform kế thừa gọi lại — không phải cài đặt lại ở từng struct.
pub(crate) trait CredentialPlatform {
    /// Đọc blob credential thô (JSON string chứa `claudeAiOauth`) cho một `config_dir`.
    fn read_blob(config_dir: &str) -> Option<String>;

    /// Đọc `subscriptionType` + `expiresAt` từ blob credential. Dùng chung mọi platform.
    fn read_keychain_meta(config_dir: &str) -> (Option<String>, Option<i64>) {
        Self::read_blob(config_dir)
            .and_then(|text| serde_json::from_str::<KeychainBlob>(&text).ok())
            .and_then(|blob| blob.claude_ai_oauth)
            .map(|oauth| (oauth.subscription_type, oauth.expires_at))
            .unwrap_or((None, None))
    }

    /// Đọc OAuth access token từ blob credential. Dùng chung mọi platform.
    fn read_oauth_token(config_dir: &str) -> Option<String> {
        Self::read_blob(config_dir)
            .and_then(|text| serde_json::from_str::<KeychainBlob>(&text).ok())
            .and_then(|blob| blob.claude_ai_oauth)
            .and_then(|oauth| oauth.access_token)
            .map(|token| token.trim().to_string())
            .filter(|token| !token.is_empty())
    }
}

#[cfg(target_os = "macos")]
type CurrentCredentials = crate::services::claude_credentials_macos::MacosCredentials;
#[cfg(not(target_os = "macos"))]
type CurrentCredentials = crate::services::claude_credentials_windows::WindowsCredentials;

/// Đọc `subscriptionType` + `expiresAt` từ credential store cho một `config_dir`.
/// Best-effort: macOS dùng Keychain, Windows/Linux đọc file `.credentials.json`.
fn read_keychain_meta(config_dir: &str) -> (Option<String>, Option<i64>) {
    CurrentCredentials::read_keychain_meta(config_dir)
}

/// Đọc blob credential (dạng String) cho một `config_dir`.
/// macOS: Keychain service. Windows/Linux: đọc file `.credentials.json` trong config dir.
pub(crate) fn read_credential_blob(config_dir: &str) -> Option<String> {
    CurrentCredentials::read_blob(config_dir)
}

/// `true` nếu `config_dir` là login mặc định (rỗng hoặc `~/.claude`).
pub fn is_default_config_dir(config_dir: &str) -> bool {
    let dir = config_dir.trim();
    if dir.is_empty() {
        return true;
    }
    match home_dir() {
        Some(home) => expand_tilde(dir) == home.join(".claude"),
        None => false,
    }
}

/// Đọc OAuth access token của một account subscription từ credential store.
/// macOS: Keychain, Windows/Linux: `.credentials.json`. `None` nếu không có.
pub fn read_oauth_token(config_dir: &str) -> Option<String> {
    CurrentCredentials::read_oauth_token(config_dir)
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

/// Thư mục home của user (`$HOME` trên macOS/Linux, `USERPROFILE` trên Windows).
pub(crate) fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Mở rộng `~` đầu đường dẫn thành thư mục home.
pub(crate) fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

/// So sánh 2 đường dẫn sau khi mở rộng `~`.
fn paths_equal(a: &str, b: &str) -> bool {
    expand_tilde(a.trim()) == expand_tilde(b.trim())
}
