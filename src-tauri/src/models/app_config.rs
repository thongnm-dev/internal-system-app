//! Các kiểu dữ liệu cho cấu hình ứng dụng (config.ini) và quản lý Store Procedure.

use serde::{Deserialize, Serialize};

/// Một key-value pair trong một section của config.ini.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

/// Một section (nhóm) trong config.ini.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSection {
    pub name: String,
    pub entries: Vec<ConfigEntry>,
}

/// Toàn bộ nội dung config.ini, chia theo section.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfigData {
    pub sections: Vec<ConfigSection>,
    pub config_path: String,
}

/// Request từ frontend khi lưu config.ini.
#[derive(Debug, Deserialize)]
pub struct SaveAppConfigRequest {
    pub sections: Vec<ConfigSection>,
}

/// Kết quả thực thi một stored procedure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpExecutionResult {
    pub name: String,
    pub success: bool,
    pub message: String,
}

/// Danh sách stored procedure có sẵn.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpInfo {
    pub name: String,
    pub file_name: String,
}

/// Kết quả thực thi toàn bộ stored procedure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpExecutionSummary {
    pub total: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub results: Vec<SpExecutionResult>,
}
