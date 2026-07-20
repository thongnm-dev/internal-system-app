//! Đường dẫn thư mục dữ liệu cục bộ của ứng dụng (`%LOCALAPPDATA%\management-systems`).
//!
//! Production: `dirs::data_local_dir()` → `%LOCALAPPDATA%\management-systems`.
//! Dev / fallback: `CARGO_MANIFEST_DIR`.

use std::path::PathBuf;

const APP_DIR_NAME: &str = "management-systems";

/// Thư mục AppData dùng chung cho mọi file dữ liệu cục bộ (JSON, profile, v.v.).
///
/// Tự tạo thư mục nếu chưa tồn tại.
pub fn data_dir() -> PathBuf {
    if let Some(local_data) = dirs::data_local_dir() {
        let dir = local_data.join(APP_DIR_NAME);
        if std::fs::create_dir_all(&dir).is_ok() {
            return dir;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.to_path_buf();
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
