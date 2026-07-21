//! Model cho module cài đặt người dùng.

use serde::{Deserialize, Serialize};

/// Thông tin cá nhân người dùng.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserProfile {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub full_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub position: String,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            full_name: String::new(),
            email: String::new(),
            phone: String::new(),
            address: String::new(),
            position: String::new(),
        }
    }
}

/// Toàn bộ cài đặt ứng dụng.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub user: UserProfile,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_theme() -> String {
    "light".to_string()
}

fn default_language() -> String {
    "vi".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            user: UserProfile::default(),
            theme: default_theme(),
            language: default_language(),
        }
    }
}

/// Request lưu cài đặt từ frontend.
#[derive(Debug, Deserialize)]
pub struct SaveSettingsRequest {
    pub user_id: i32,
    pub user: UserProfile,
    pub theme: String,
    pub language: String,
}
