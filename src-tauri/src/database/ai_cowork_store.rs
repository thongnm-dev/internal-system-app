//! Tầng lưu trữ cục bộ cho state màn hình AI Cowork.
//!
//! Lưu lại project directory làm việc gần nhất trong file JSON
//! `ai_cowork.json`, nằm trong thư mục `data` cùng cấp với thư mục
//! `config` ([`app_config::local_data_dir`]) — để khi mở lại màn hình, tự động
//! load lại thư mục project đã dùng lần trước.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::app::result::AppResult;
use crate::utils::app_config;

/// Tên file dữ liệu cục bộ.
const DATA_FILE: &str = "ai_cowork.json";

/// State làm việc gần nhất của màn AI Cowork.
///
/// Ngoài project directory, còn lưu lại danh sách task đang hiển thị (kèm trạng
/// thái tick chọn) và workflow đang chọn / đang áp dụng — để khi mở lại màn hình
/// khôi phục nguyên vẹn tiến trình đang làm dở. Danh sách task lưu dạng JSON
/// nguyên bản (opaque) để không phụ thuộc cấu trúc task của tầng frontend.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AiCoworkState {
    #[serde(default)]
    pub project_dir: String,
    #[serde(default)]
    pub selected_tasks: Vec<Value>,
    #[serde(default)]
    pub confirmed_task_ids: Vec<i64>,
    #[serde(default)]
    pub selected_workflow_id: Option<i64>,
    #[serde(default)]
    pub applied_workflow_id: Option<i64>,
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
