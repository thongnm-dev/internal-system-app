//! Đọc credential store trên macOS — qua Keychain (lệnh `security`).
//! Xem [`crate::services::claude_credentials_windows`] cho phía Windows/Linux.

use std::process::Command;

use sha2::{Digest, Sha256};

use crate::services::claude_detected::{self, CredentialPlatform};

/// Keychain service cho config dir mặc định.
const DEFAULT_KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

/// Đọc credential store trên macOS — qua Keychain (lệnh `security`).
pub(crate) struct MacosCredentials;

impl CredentialPlatform for MacosCredentials {
    fn read_blob(config_dir: &str) -> Option<String> {
        keychain_services_for(config_dir)
            .into_iter()
            .find_map(|service| {
                let output = Command::new("security")
                    .args(["find-generic-password", "-s", &service, "-w"])
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            })
    }
}

/// Các tên Keychain service khả dĩ ứng với một `config_dir`, xếp theo độ ưu tiên.
fn keychain_services_for(config_dir: &str) -> Vec<String> {
    let dir = config_dir.trim();
    if claude_detected::is_default_config_dir(dir) {
        return vec![DEFAULT_KEYCHAIN_SERVICE.to_string()];
    }

    // Sinh các biến thể chuỗi ứng viên để hash.
    let expanded = claude_detected::expand_tilde(dir).to_string_lossy().to_string();
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

/// 8 ký tự hex đầu của SHA-256 chuỗi (Keychain hash).
fn sha256_hex8(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().take(4).map(|b| format!("{b:02x}")).collect()
}
