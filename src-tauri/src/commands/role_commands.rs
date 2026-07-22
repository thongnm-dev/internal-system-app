use crate::models::role::{CreateRoleRequest, RoleSummary, UpdateRoleRequest};
use crate::services::role_service;

#[tauri::command]
pub async fn list_role_details() -> Result<Vec<RoleSummary>, String> {
    role_service::list_roles().await.map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn create_role(request: CreateRoleRequest) -> Result<RoleSummary, String> {
    role_service::create_role(request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn update_role(role_id: i32, request: UpdateRoleRequest) -> Result<RoleSummary, String> {
    role_service::update_role(role_id, request)
        .await
        .map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn delete_role(role_id: i32) -> Result<(), String> {
    role_service::delete_role(role_id)
        .await
        .map_err(crate::app::error::log_err)
}
