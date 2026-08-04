//! Mở terminal macOS — ghi ra file `.command` tạm rồi mở bằng Terminal.app.
//! Xem [`crate::services::claude_terminal`] cho phần logic dùng chung mọi platform,
//! [`crate::services::claude_terminal_windows`] cho phía Windows.

use std::path::Path;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::services::claude_terminal::TerminalPlatform;

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
