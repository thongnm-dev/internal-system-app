//! Xử lý AI Usage riêng cho macOS: đọc credential qua Keychain (lệnh `security`) +
//! mở terminal bằng Terminal.app.
//!
//! Phần xử lý chung (parse blob JSON, ghi script tạm rồi launch) nằm ở default method
//! của [`crate::services::claude_detected::CredentialPlatform`] /
//! [`crate::services::claude_terminal::TerminalPlatform`] — struct ở đây chỉ override
//! phần khác nhau theo OS. Xem [`crate::services::claude_usage_windows`] cho phía Windows.

use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::services::claude_detected::{self, CredentialPlatform};
use crate::services::claude_terminal::TerminalPlatform;

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

/// Mở terminal macOS — ghi ra file `.command` tạm rồi mở bằng Terminal.app.
pub(crate) struct MacosTerminal;

impl TerminalPlatform for MacosTerminal {
    fn script_extension() -> &'static str {
        "command"
    }

    fn script_content(expanded_wd: &str, is_default: bool, expanded_dir: &str, command: Option<&str>) -> String {
        // Ghi câu lệnh ra 1 file `.command` tạm rồi mở bằng Terminal, thay vì nhồi cả câu lệnh
        // (có dấu `"` bọc quanh prompt) vào chuỗi AppleScript `do script "..."` — dấu `"` đó cắt
        // sớm chuỗi literal của AppleScript và gây lỗi cú pháp (-2740). Trong file script, dấu `"`
        // chỉ là quoting shell bình thường.
        let mut script = format!("#!/bin/bash\ncd '{expanded_wd}'\n");
        if !is_default {
            script.push_str(&format!("export CLAUDE_CONFIG_DIR='{expanded_dir}'\n"));
        }
        if let Some(cmd) = command {
            script.push_str(&format!("{cmd}\n"));
        }
        script.push_str("exec bash\n");
        script
    }

    fn launch(script_path: &Path) -> AppResult<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(script_path)
            .map_err(|e| AppError::new(&format!("Không thể đọc quyền file terminal: {e}")))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(script_path, perms)
            .map_err(|e| AppError::new(&format!("Không thể set quyền file terminal: {e}")))?;
        std::process::Command::new("open")
            .args(["-a", "Terminal"])
            .arg(script_path)
            .spawn()
            .map_err(|e| AppError::new(&format!("Không thể mở Terminal: {e}")))?;
        Ok(())
    }
}
