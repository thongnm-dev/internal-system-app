//! Model cho module Terminal (PTY nhúng trong app).
//!
//! Các payload này được serialize và bắn về frontend qua Tauri event khi
//! tiến trình shell trong pseudo-terminal có output mới hoặc kết thúc.

use serde::Serialize;

/// Payload event `terminal-output` — một chunk dữ liệu đọc từ PTY.
///
/// `data` là chuỗi base64 của các byte thô đọc được (base64 để tránh làm hỏng
/// các ký tự UTF-8 nhiều byte bị cắt ngang giữa 2 lần đọc, và để mã hoá được cả
/// các byte không phải UTF-8 như escape sequence của terminal).
#[derive(Clone, Serialize)]
pub struct TerminalOutput {
    /// Định danh phiên terminal (khớp với id trả về từ `terminal_spawn`).
    pub id: String,
    /// Dữ liệu đọc từ PTY, đã mã hoá base64.
    pub data: String,
}

/// Payload event `terminal-exit` — tiến trình shell của một phiên đã kết thúc.
#[derive(Clone, Serialize)]
pub struct TerminalExit {
    /// Định danh phiên terminal đã kết thúc.
    pub id: String,
    /// Mã thoát của tiến trình (nếu lấy được).
    pub code: Option<i32>,
}
