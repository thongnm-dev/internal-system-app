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
#[cfg(target_os = "macos")]
use std::process::Command;

use chrono::{Local, TimeZone};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::database::ai_account_store::StoredAccount;
use crate::models::ai_usage::DetectedLogin;

/// Keychain service cho config dir mặc định (macOS).
const DEFAULT_KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

/// Phần `oauthAccount` trong `.claude.json`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OauthAccount {
    #[serde(default)]
    email_address: String,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    organization_type: String,
    #[serde(default)]
    billing_type: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeJson {
    #[serde(rename = "oauthAccount")]
    oauth_account: Option<OauthAccount>,
}

/// Phần `claudeAiOauth` trong blob Keychain.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeychainClaudeOauth {
    #[serde(default)]
    subscription_type: Option<String>,
    #[serde(default)]
    expires_at: Option<i64>,
    #[serde(default)]
    access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeychainBlob {
    claude_ai_oauth: Option<KeychainClaudeOauth>,
}

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

    let already_added = existing.iter().any(|a| {
        (!a.email.is_empty() && a.email.eq_ignore_ascii_case(&email))
            || (!a.config_dir.is_empty() && paths_equal(&a.config_dir, config_dir_label))
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

/// Đọc `subscriptionType` + `expiresAt` từ Keychain cho một `config_dir` (macOS).
/// Thử các service khả dĩ, lấy entry đầu tiên đọc được. Best-effort.
fn read_keychain_meta(config_dir: &str) -> (Option<String>, Option<i64>) {
    keychain_services_for(config_dir)
        .into_iter()
        .find_map(|service| read_keychain_oauth(&service))
        .map(|oauth| (oauth.subscription_type, oauth.expires_at))
        .unwrap_or((None, None))
}

/// Đọc blob `claudeAiOauth` từ một Keychain service (macOS). Best-effort.
fn read_keychain_oauth(service: &str) -> Option<KeychainClaudeOauth> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("security")
            .args(["find-generic-password", "-s", service, "-w"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8(output.stdout).ok()?;
        let blob: KeychainBlob = serde_json::from_str(text.trim()).ok()?;
        blob.claude_ai_oauth
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = service;
        None
    }
}

/// Các tên Keychain service khả dĩ ứng với một `config_dir`, xếp theo độ ưu tiên.
///
/// Khớp thuật toán của Claude Code (hàm `Ky` trong bundle):
/// - `CLAUDE_CONFIG_DIR` **không set** (login mặc định) → `Claude Code-credentials`.
/// - `CLAUDE_CONFIG_DIR=X` → `Claude Code-credentials-<hash>`, với
///   `hash` = 8 hex đầu của `SHA-256(X)` — **X là chuỗi env thô** (đã `.normalize("NFC")`),
///   KHÔNG phải path đã resolve.
///
/// Vì app không biết user set `CLAUDE_CONFIG_DIR` dưới dạng nào (đường dẫn tuyệt đối,
/// có `~`, hay có `/` cuối), ta thử vài biến thể của chuỗi đã lưu để tăng khả năng khớp.
pub fn keychain_services_for(config_dir: &str) -> Vec<String> {
    let dir = config_dir.trim();
    if is_default_config_dir(dir) {
        return vec![DEFAULT_KEYCHAIN_SERVICE.to_string()];
    }

    // Sinh các biến thể chuỗi ứng viên để hash.
    let expanded = expand_tilde(dir).to_string_lossy().to_string();
    let mut inputs: Vec<String> = Vec::new();
    for base in [dir.to_string(), expanded] {
        let trimmed = base.trim_end_matches('/').to_string();
        for candidate in [base, trimmed] {
            if !candidate.is_empty() && !inputs.contains(&candidate) {
                inputs.push(candidate);
            }
        }
    }

    inputs
        .iter()
        .map(|input| format!("{DEFAULT_KEYCHAIN_SERVICE}-{}", sha256_hex8(input)))
        .collect()
}

/// 8 ký tự hex đầu của SHA-256 chuỗi (giống `createHash("sha256").digest("hex").substring(0,8)`).
fn sha256_hex8(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().take(4).map(|b| format!("{b:02x}")).collect()
}

/// `true` nếu `config_dir` là login mặc định (rỗng hoặc `~/.claude`).
fn is_default_config_dir(config_dir: &str) -> bool {
    let dir = config_dir.trim();
    if dir.is_empty() {
        return true;
    }
    match home_dir() {
        Some(home) => expand_tilde(dir) == home.join(".claude"),
        None => false,
    }
}

/// Đọc OAuth access token của một account subscription từ Keychain (macOS).
/// Thử lần lượt các service khả dĩ; trả token đầu tiên đọc được. `None` nếu không có.
pub fn read_oauth_token(config_dir: &str) -> Option<String> {
    keychain_services_for(config_dir)
        .into_iter()
        .find_map(|service| read_keychain_oauth(&service).and_then(|oauth| oauth.access_token))
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
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

/// Thư mục home của user (`$HOME`).
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Mở rộng `~` đầu đường dẫn thành thư mục home.
fn expand_tilde(path: &str) -> PathBuf {
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
