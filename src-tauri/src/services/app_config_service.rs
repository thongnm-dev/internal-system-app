//! Business logic cho đọc/ghi toàn bộ config.ini.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::app_config::{
    AppConfigData, ConfigEntry, ConfigSection, SaveAppConfigRequest,
};
use crate::utils::app_config::config_path;
use ini::Ini;

/// Đọc toàn bộ config.ini, trả về danh sách sections kèm đường dẫn file.
pub fn get_app_config() -> AppResult<AppConfigData> {
    let path = config_path();
    let path_str = path.display().to_string();

    if !path.exists() {
        return Ok(AppConfigData {
            sections: Vec::new(),
            config_path: path_str,
        });
    }

    let ini = Ini::load_from_file(&path).map_err(|e| {
        AppError::new(format!("Failed to load config.ini: {e}"))
    })?;

    let mut sections = Vec::new();

    for (section_name, props) in ini.iter() {
        let name = section_name.unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }

        let entries: Vec<ConfigEntry> = props
            .iter()
            .map(|(k, v)| ConfigEntry {
                key: k.to_string(),
                value: v.to_string(),
            })
            .collect();

        sections.push(ConfigSection { name, entries });
    }

    Ok(AppConfigData {
        sections,
        config_path: path_str,
    })
}

/// Ghi toàn bộ config.ini từ dữ liệu frontend gửi lên.
pub fn save_app_config(request: SaveAppConfigRequest) -> AppResult<AppConfigData> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::new(format!(
                "Failed to create config directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    let mut ini = Ini::new();

    for section in &request.sections {
        for entry in &section.entries {
            ini.with_section(Some(&section.name))
                .set(&entry.key, &entry.value);
        }
    }

    ini.write_to_file(&path).map_err(|e| {
        AppError::new(format!("Failed to write config.ini: {e}"))
    })?;

    get_app_config()
}
