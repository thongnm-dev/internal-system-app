//! Model cho module xác thực người dùng.

use serde::{Deserialize, Serialize};

/// Request đăng nhập từ frontend.
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Thông tin người dùng trả về frontend sau khi đăng nhập thành công.
#[derive(Serialize)]
pub struct LoginResponse {
    pub user_id: i32,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub roles: Vec<String>,
}
