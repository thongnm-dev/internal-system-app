//! Tầng lưu trữ cục bộ cho danh sách repository của màn hình Git Desktop.
//!
//! Danh sách repo được lưu trong file JSON `git_repos.json` trong thư mục AppData
//! (`%LOCALAPPDATA%\management-systems`) — mỗi máy có danh sách riêng, không đẩy
//! lên database dùng chung.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::app::result::AppResult;
use crate::models::git::GitRepo;
use crate::utils::app_config;

/// Tên file dữ liệu cục bộ.
const DATA_FILE: &str = "git_repos.json";

/// Toàn bộ dữ liệu danh sách repo được serialize xuống file.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GitRepoData {
    #[serde(default)]
    pub repos: Vec<GitRepo>,
    /// Bộ đếm id tự tăng (bắt đầu từ 1).
    #[serde(default)]
    pub next_id: i64,
}

fn data_path() -> PathBuf {
    app_config::data_dir().join(DATA_FILE)
}

/// Đọc dữ liệu từ file. File chưa tồn tại → trả về mặc định (rỗng).
pub fn load() -> AppResult<GitRepoData> {
    let path = data_path();
    if !path.exists() {
        return Ok(GitRepoData::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

/// Ghi dữ liệu xuống file (pretty JSON, ghi đè).
pub fn save(data: &GitRepoData) -> AppResult<()> {
    let path = data_path();
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, content)?;
    Ok(())
}
