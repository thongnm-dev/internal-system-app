//! Business logic cho module phân quyền menu (governance).

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::{menu_permission_store, role_store, user_store};
use crate::models::menu_permission::{
    EffectiveMenuPermission, SaveRoleMenuPermissionsRequest, SaveUserMenuPermissionsRequest,
    UserMenuPermission,
};

pub async fn list_role_menu_permissions(role_id: i32) -> AppResult<Vec<String>> {
    menu_permission_store::list_role_menu_keys(role_id).await
}

pub async fn save_role_menu_permissions(
    request: SaveRoleMenuPermissionsRequest,
) -> AppResult<Vec<String>> {
    role_store::find_by_id(request.role_id)
        .await?
        .ok_or_else(|| AppError::new(format!("Role '{}' not found.", request.role_id)))?;

    menu_permission_store::sync_role_menu_keys(request.role_id, &request.menu_keys).await?;
    menu_permission_store::list_role_menu_keys(request.role_id).await
}

pub async fn list_user_menu_permissions(user_id: i32) -> AppResult<Vec<UserMenuPermission>> {
    menu_permission_store::list_user_overrides(user_id).await
}

pub async fn save_user_menu_permissions(
    request: SaveUserMenuPermissionsRequest,
) -> AppResult<Vec<UserMenuPermission>> {
    user_store::find_by_id(request.user_id)
        .await?
        .ok_or_else(|| AppError::new(format!("User '{}' not found.", request.user_id)))?;

    // Một menu không thể vừa được cấp vừa bị thu hồi.
    if let Some(key) = request
        .allow_keys
        .iter()
        .find(|k| request.deny_keys.contains(k))
    {
        return Err(AppError::new(format!(
            "Menu '{key}' vừa được cấp vừa bị thu hồi — chỉ chọn một."
        )));
    }

    menu_permission_store::sync_user_overrides(
        request.user_id,
        &request.allow_keys,
        &request.deny_keys,
    )
    .await?;
    menu_permission_store::list_user_overrides(request.user_id).await
}

pub async fn list_effective_menu_permissions(
    user_id: i32,
) -> AppResult<Vec<EffectiveMenuPermission>> {
    menu_permission_store::list_effective(user_id).await
}
