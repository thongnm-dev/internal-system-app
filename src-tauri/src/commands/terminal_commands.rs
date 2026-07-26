//! Tauri IPC commands cho module Terminal nhúng.
//!
//! Tầng mỏng — chỉ nhận request từ frontend và uỷ quyền xuống
//! `services::terminal_service`.

use tauri::AppHandle;

use crate::services::terminal_service;

/// Mở một phiên terminal mới, trả về id phiên để frontend dùng cho các lệnh sau.
#[tauri::command]
pub fn terminal_spawn(
    app: AppHandle,
    cwd: Option<String>,
    shell: Option<String>,
    rows: u16,
    cols: u16,
) -> Result<String, String> {
    terminal_service::spawn(app, cwd, shell, rows, cols)
}

/// Gửi dữ liệu người dùng gõ vào shell của phiên `id`.
///
/// Cố tình để đồng bộ (không `async`): các lệnh write chạy tuần tự theo đúng thứ
/// tự IPC nhận được, giữ nguyên thứ tự byte gõ vào (async runtime có thể đảo thứ
/// tự 2 phím gõ nhanh). Mỗi lần chỉ ghi vài byte nên không chặn đáng kể.
#[tauri::command]
pub fn terminal_write(id: String, data: String) -> Result<(), String> {
    terminal_service::write(&id, &data)
}

/// Đổi kích thước (số hàng/cột) của phiên `id`.
#[tauri::command]
pub fn terminal_resize(id: String, rows: u16, cols: u16) -> Result<(), String> {
    terminal_service::resize(&id, rows, cols)
}

/// Kết thúc phiên `id`.
#[tauri::command]
pub fn terminal_kill(id: String) -> Result<(), String> {
    terminal_service::kill(&id)
}
