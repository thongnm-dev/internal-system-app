//! Mở terminal Windows — ghi ra file `.bat` tạm rồi mở bằng `cmd /k`.
//! Xem [`crate::services::claude_terminal`] cho phần logic dùng chung mọi platform,
//! [`crate::services::claude_terminal_macos`] cho phía macOS.

use std::path::Path;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::services::claude_terminal::TerminalPlatform;

pub(crate) struct WindowsTerminal;

impl TerminalPlatform for WindowsTerminal {
    fn script_extension() -> &'static str {
        "bat"
    }

    fn script_content(expanded_wd: &str, is_default: bool, expanded_dir: &str, command: Option<&str>) -> String {
        // Ghi ra file .bat tạm thay vì nhồi cả câu lệnh (có thể chứa dấu ngoặc kép của prompt)
        // vào một argument của `cmd /k`: cmd.exe không tự bóc tách quote lồng nhau theo kiểu
        // CommandLineToArgvW mà Rust dùng để escape arguments, nên quote bị lệch/dư khi nhồi trực tiếp.
        // `chcp 65001` bắt buộc phải đứng trước mọi dòng chứa ký tự non-ASCII (vd. tên file tiếng
        // Nhật trong prompt): file .bat được ghi bằng UTF-8, nhưng cmd.exe mặc định đọc theo code
        // page ANSI/OEM của hệ thống — nếu không chuyển sang UTF-8 trước, các byte UTF-8 đó bị
        // hiểu sai thành ký tự khác (mojibake) trước khi được truyền tiếp cho `claude`.
        let mut script = format!("@echo off\r\nchcp 65001 > nul\r\ncd /d \"{expanded_wd}\"\r\n");
        if !is_default {
            script.push_str(&format!("set CLAUDE_CONFIG_DIR={expanded_dir}\r\n"));
        }
        if let Some(cmd) = command {
            script.push_str(&format!("{cmd}\r\n"));
        }
        script
    }

    fn launch(script_path: &Path) -> AppResult<()> {
        std::process::Command::new("cmd")
            .args(["/c", "start", "cmd", "/k"])
            .arg(script_path)
            .spawn()
            .map_err(|e| AppError::new(&format!("Không thể mở terminal: {e}")))?;
        Ok(())
    }
}
