//! Tầng lưu trữ cục bộ cho state màn hình AI Translate Cowork.
//!
//! Lưu lại project directory làm việc gần nhất trong file JSON
//! `ai_translate_cowork.json`, nằm trong thư mục `data` cùng cấp với thư mục
//! `config` ([`app_config::local_data_dir`]) — để khi mở lại màn hình, tự động
//! load lại thư mục project đã dùng lần trước.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::app::result::AppResult;
use crate::utils::app_config;

/// Tên file dữ liệu cục bộ.
const DATA_FILE: &str = "ai_translate_cowork.json";

/// State làm việc gần nhất của màn AI Translate Cowork.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AiTranslateCoworkState {
    #[serde(default)]
    pub project_dir: String,
}

fn data_path() -> PathBuf {
    app_config::local_data_dir().join(DATA_FILE)
}

/// Đọc state từ file. File chưa tồn tại → trả về mặc định (rỗng).
pub fn load() -> AppResult<AiTranslateCoworkState> {
    let path = data_path();
    if !path.exists() {
        return Ok(AiTranslateCoworkState::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

/// Ghi state xuống file (pretty JSON, ghi đè).
pub fn save(data: &AiTranslateCoworkState) -> AppResult<()> {
    let path = data_path();
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, content)?;
    Ok(())
}
