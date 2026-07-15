//! Model cho module quản lý menu (governance).

use serde::{Deserialize, Serialize};

/// Một mục menu trong hệ thống.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuConfig {
    pub key: String,
    pub title: String,
    pub path: String,
    pub icon: String,
    pub group: String,
    pub visible: bool,
    pub order: i32,
}

/// Request tạo hoặc cập nhật menu item từ frontend.
#[derive(Debug, Deserialize)]
pub struct SaveMenuConfigRequest {
    pub key: String,
    pub title: String,
    pub path: String,
    pub icon: String,
    pub group: String,
    pub visible: bool,
    pub order: i32,
}

/// Request lưu toàn bộ danh sách menu (bulk save).
#[derive(Debug, Deserialize)]
pub struct SaveAllMenuConfigsRequest {
    pub items: Vec<SaveMenuConfigRequest>,
}
