//! Tầng lưu trữ cục bộ cho state màn hình AI Cowork.
//!
//! Lưu lại project directory làm việc gần nhất trong file JSON
//! `ai_cowork.json`, nằm trong thư mục `data` cùng cấp với thư mục
//! `config` ([`app_config::local_data_dir`]) — để khi mở lại màn hình, tự động
//! load lại thư mục project đã dùng lần trước.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::app::result::AppResult;
use crate::utils::app_config;

/// Tên file dữ liệu cục bộ.
const DATA_FILE: &str = "ai_cowork.json";

/// State làm việc gần nhất của màn AI Cowork.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AiCoworkState {
    #[serde(default)]
    pub project_dir: String,
}

fn data_path() -> PathBuf {
    app_config::local_data_dir().join(DATA_FILE)
}

/// Đọc state từ file. File chưa tồn tại → trả về mặc định (rỗng).
pub fn load() -> AppResult<AiCoworkState> {
    let path = data_path();
    if !path.exists() {
        return Ok(AiCoworkState::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

/// Ghi state xuống file (pretty JSON, ghi đè).
pub fn save(data: &AiCoworkState) -> AppResult<()> {
    let path = data_path();
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, content)?;
    Ok(())
}
