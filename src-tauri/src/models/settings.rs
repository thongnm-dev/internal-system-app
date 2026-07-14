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

/// Một API key được cấu hình.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiKeySetting {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub api_key: String,
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
    #[serde(default = "default_api_keys")]
    pub api_keys: Vec<ApiKeySetting>,
}

fn default_theme() -> String {
    "light".to_string()
}

fn default_language() -> String {
    "vi".to_string()
}

fn default_api_keys() -> Vec<ApiKeySetting> {
    vec![ApiKeySetting {
        id: "default".to_string(),
        name: String::new(),
        key: String::new(),
        api_key: String::new(),
    }]
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            user: UserProfile::default(),
            theme: default_theme(),
            language: default_language(),
            api_keys: default_api_keys(),
        }
    }
}

/// Request lưu cài đặt từ frontend.
#[derive(Debug, Deserialize)]
pub struct SaveSettingsRequest {
    pub user: UserProfile,
    pub theme: String,
    pub language: String,
    pub api_keys: Vec<ApiKeySetting>,
}
