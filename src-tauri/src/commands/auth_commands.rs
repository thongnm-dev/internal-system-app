use crate::models::auth::{LoginRequest, LoginResponse};
use crate::services::auth_service;

#[tauri::command]
pub async fn login(request: LoginRequest) -> Result<LoginResponse, String> {
    auth_service::login(request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn request_password_reset(username: String) -> Result<String, String> {
    auth_service::request_password_reset(&username)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn verify_password_reset(username: String, code: String) -> Result<String, String> {
    auth_service::verify_password_reset(&username, &code)
        .await
        .map_err(crate::app::error::log_err)
}
