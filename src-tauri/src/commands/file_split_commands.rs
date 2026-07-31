//! Tauri command handler cho công cụ nén ZIP (AES-256) và cắt file `.001`.

use crate::models::file_split::{FileSplitOptions, FileSplitResult};
use crate::services::file_split_service;

/// Nén các file/folder đã chọn thành ZIP (mã hoá AES-256 nếu có mật khẩu) rồi
/// cắt thành các phần `.001/.002/...` nếu vượt giới hạn dung lượng.
#[tauri::command]
pub fn file_split_run(options: FileSplitOptions) -> Result<FileSplitResult, String> {
    file_split_service::compress_and_split(options).map_err(crate::app::error::log_err)
}
