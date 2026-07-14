//! Lưu trữ cài đặt ứng dụng dưới dạng JSON file.
//!
//! File `pjjyuji_settings.json` nằm trong app data directory.

use crate::app::result::AppResult;
use crate::models::settings::AppSettings;
use std::fs;
use std::path::{Path, PathBuf};

const SETTINGS_FILE: &str = "pjjyuji_settings.json";

/// Đọc cài đặt từ file JSON. Trả về giá trị mặc định nếu file chưa tồn tại.
pub fn load(app_data_dir: &Path) -> AppResult<AppSettings> {
    let path = file_path(app_data_dir)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content = fs::read_to_string(path)?;
    let settings: AppSettings = serde_json::from_str(&content)?;
    Ok(settings)
}

/// Ghi cài đặt ra file JSON (pretty-printed).
pub fn save(app_data_dir: &Path, settings: &AppSettings) -> AppResult<()> {
    let path = file_path(app_data_dir)?;
    let content = serde_json::to_string_pretty(settings)?;
    fs::write(path, content)?;
    Ok(())
}

fn file_path(app_data_dir: &Path) -> AppResult<PathBuf> {
    fs::create_dir_all(app_data_dir)?;
    Ok(app_data_dir.join(SETTINGS_FILE))
}
