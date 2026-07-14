//! Business logic cho module cài đặt ứng dụng.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::settings_store;
use crate::models::settings::{AppSettings, SaveSettingsRequest};
use std::path::Path;

/// Đọc cài đặt hiện tại.
pub fn get_settings(app_data_dir: &Path) -> AppResult<AppSettings> {
    settings_store::load(app_data_dir)
}

/// Validate và lưu cài đặt.
pub fn save_settings(
    app_data_dir: &Path,
    request: SaveSettingsRequest,
) -> AppResult<AppSettings> {
    let theme = request.theme.trim().to_string();
    if theme != "light" && theme != "dark" {
        return Err(AppError::new("Theme không hợp lệ (chỉ chấp nhận light hoặc dark)."));
    }

    let language = request.language.trim().to_string();
    if language != "vi" && language != "en" && language != "ja" {
        return Err(AppError::new("Ngôn ngữ không hợp lệ (chỉ chấp nhận vi, en hoặc ja)."));
    }

    let settings = AppSettings {
        user: request.user,
        theme,
        language,
        api_keys: request.api_keys,
    };

    settings_store::save(app_data_dir, &settings)?;
    Ok(settings)
}
