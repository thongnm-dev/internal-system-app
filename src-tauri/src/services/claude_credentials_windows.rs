//! Đọc credential store cho Windows: file `.credentials.json` trong config dir.
//! Dùng chung cơ chế đọc file với Linux (không có Keychain), nên struct này không bị
//! `cfg` giới hạn riêng target_os `windows`. Xem [`crate::services::claude_credentials_macos`]
//! cho phía macOS.

use crate::services::claude_detected::{self, CredentialPlatform};

/// Đọc credential store trên Windows (và Linux) — file `.credentials.json` trong config dir.
pub(crate) struct WindowsCredentials;

impl CredentialPlatform for WindowsCredentials {
    fn read_blob(config_dir: &str) -> Option<String> {
        let dir = if claude_detected::is_default_config_dir(config_dir) {
            claude_detected::home_dir()?.join(".claude")
        } else {
            claude_detected::expand_tilde(config_dir)
        };
        std::fs::read_to_string(dir.join(".credentials.json")).ok()
    }
}
