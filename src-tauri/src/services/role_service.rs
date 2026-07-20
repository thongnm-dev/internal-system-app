use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::role_store;
use crate::models::role::{CreateRoleRequest, RoleSummary, UpdateRoleRequest};

pub async fn list_roles() -> AppResult<Vec<RoleSummary>> {
    role_store::list_all().await
}

pub async fn create_role(request: CreateRoleRequest) -> AppResult<RoleSummary> {
    let name = request.name.trim().to_string();
    let description = request.description.unwrap_or_default().trim().to_string();

    if name.is_empty() {
        return Err(AppError::new("Role name is required."));
    }

    if role_store::name_exists(&name, None).await? {
        return Err(AppError::new(format!("Role '{name}' already exists.")));
    }

    role_store::insert_role(&name, &description).await
}

pub async fn update_role(id: i32, request: UpdateRoleRequest) -> AppResult<RoleSummary> {
    let name = request.name.trim().to_string();
    let description = request.description.unwrap_or_default().trim().to_string();

    if name.is_empty() {
        return Err(AppError::new("Role name is required."));
    }

    role_store::find_by_id(id)
        .await?
        .ok_or_else(|| AppError::new(format!("Role '{id}' not found.")))?;

    if role_store::name_exists(&name, Some(id)).await? {
        return Err(AppError::new(format!("Role '{name}' already exists.")));
    }

    role_store::update_role(id, &name, &description).await
}

pub async fn delete_role(id: i32) -> AppResult<()> {
    let role = role_store::find_by_id(id)
        .await?
        .ok_or_else(|| AppError::new(format!("Role '{id}' not found.")))?;

    if role.user_count > 0 {
        return Err(AppError::new(format!(
            "Cannot delete role '{}' — it is still assigned to {} user(s).",
            role.name, role.user_count
        )));
    }

    if !role_store::delete_by_id(id).await? {
        return Err(AppError::new(format!("Role '{id}' not found.")));
    }

    Ok(())
}
