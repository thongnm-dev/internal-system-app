//! Đường dẫn dữ liệu và cấu hình ứng dụng.
//!
//! - `data_dir()` → `%LOCALAPPDATA%\management-systems` (AppData cục bộ).
//! - `config_path()` → `config/config.ini` kế bên .exe (production) hoặc
//!   `src-tauri/config/config.ini` (development).

use std::path::PathBuf;

const APP_DIR_NAME: &str = "management-systems";

/// Template mặc định cho `config.ini`, được embed vào binary lúc compile.
/// Chỉ dùng để tạo file khi cài đặt mới (file chưa tồn tại).
const DEFAULT_CONFIG_TEMPLATE: &str = include_str!("../../config/config.ini");

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

/// Xác định đường dẫn tới file `config.ini`.
///
/// Production: `exe_dir/config/config.ini`.
/// Development: fallback về `CARGO_MANIFEST_DIR/config/config.ini`.
///
/// Nếu file chưa tồn tại (cài đặt mới), tự tạo từ template embedded.
pub fn config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    if let Some(dir) = &exe_dir {
        let config_dir = dir.join("config");
        let candidate = config_dir.join("config.ini");

        if candidate.exists() {
            return candidate;
        }

        // Cài đặt mới: tạo config.ini từ template embedded
        if let Ok(()) = std::fs::create_dir_all(&config_dir) {
            let _ = std::fs::write(&candidate, DEFAULT_CONFIG_TEMPLATE);
        }
        return candidate;
    }

    // Fallback: cạnh .exe trực tiếp (tương thích cũ)
    if let Some(dir) = &exe_dir {
        let candidate = dir.join("config.ini");
        if candidate.exists() {
            return candidate;
        }
    }

    // Development: config.ini nằm trong thư mục src-tauri/config/
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("config").join("config.ini")
}

/// Thư mục `data` cùng cấp với thư mục `config` (kế bên .exe ở production, hoặc
/// `src-tauri/data` ở development khi không lấy được exe path).
///
/// Khác với [`data_dir`] (AppData `%LOCALAPPDATA%`), thư mục này dùng cho các file
/// lịch sử làm việc gắn với từng bản cài đặt (ví dụ: state của màn AI Translate Cowork).
///
/// Tự tạo thư mục nếu chưa tồn tại.
pub fn local_data_dir() -> PathBuf {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("data")))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"));
    let _ = std::fs::create_dir_all(&dir);
    dir
}
