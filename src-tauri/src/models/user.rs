use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetail {
    pub id: i32,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub position: String,
    pub is_active: bool,
    pub roles: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: i32,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub is_active: bool,
    pub roles: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub position: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub position: Option<String>,
    pub is_active: bool,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub new_password: String,
}
