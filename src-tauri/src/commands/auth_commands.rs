//! Tauri command handlers cho module xác thực.

use crate::models::auth::{LoginRequest, LoginResponse};
use crate::services::auth_service;

#[tauri::command]
pub async fn login(request: LoginRequest) -> Result<LoginResponse, String> {
    auth_service::login(request)
        .await
        .map_err(|e| e.to_string())
}
