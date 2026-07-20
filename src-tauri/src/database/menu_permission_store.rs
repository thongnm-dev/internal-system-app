//! Data access cho bảng `role_menu_permissions` và `user_menu_permissions` (PostgreSQL).

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::menu_permission::{EffectiveMenuPermission, UserMenuPermission};
use crate::utils::pgsql_connect;

pub async fn list_role_menu_keys(role_id: i32) -> AppResult<Vec<String>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_menu_permission_role_select($1)", &[&role_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to list role menu permissions: {e}")))?;

    Ok(rows.iter().map(|row| row.get("menu_key")).collect())
}

pub async fn sync_role_menu_keys(role_id: i32, menu_keys: &[String]) -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    client
        .execute(
            "SELECT sp_menu_permission_role_sync($1, $2)",
            &[&role_id, &menu_keys],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to save role menu permissions: {e}")))?;

    Ok(())
}

pub async fn list_user_overrides(user_id: i32) -> AppResult<Vec<UserMenuPermission>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_menu_permission_user_select($1)", &[&user_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to list user menu permissions: {e}")))?;

    Ok(rows
        .iter()
        .map(|row| UserMenuPermission {
            menu_key: row.get("menu_key"),
            is_allowed: row.get("is_allowed"),
        })
        .collect())
}

pub async fn sync_user_overrides(
    user_id: i32,
    allow_keys: &[String],
    deny_keys: &[String],
) -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    client
        .execute(
            "SELECT sp_menu_permission_user_sync($1, $2, $3)",
            &[&user_id, &allow_keys, &deny_keys],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to save user menu permissions: {e}")))?;

    Ok(())
}

pub async fn list_effective(user_id: i32) -> AppResult<Vec<EffectiveMenuPermission>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_menu_permission_effective_select($1)",
            &[&user_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to resolve menu permissions: {e}")))?;

    Ok(rows
        .iter()
        .map(|row| EffectiveMenuPermission {
            menu_key: row.get("menu_key"),
            is_allowed: row.get("is_allowed"),
            role_allowed: row.get("role_allowed"),
            source: row.get("source"),
        })
        .collect())
}
