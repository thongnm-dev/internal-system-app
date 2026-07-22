use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::utils::pgsql_connect;

pub struct UserRow {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub full_name: String,
    pub email: String,
    pub is_active: bool,
}

pub async fn find_user_by_username(username: &str) -> AppResult<Option<UserRow>> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt(
            "SELECT * FROM sp_auth_find_user_by_username($1)",
            &[&username],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query user: {e}")))?;

    Ok(row.map(|r| UserRow {
        id: r.get("id"),
        username: r.get("username"),
        password_hash: r.get("password_hash"),
        full_name: r.get("full_name"),
        email: r.get("email"),
        is_active: r.get("is_active"),
    }))
}

pub async fn get_user_roles(user_id: i32) -> AppResult<Vec<String>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_auth_get_user_roles($1)",
            &[&user_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query user roles: {e}")))?;

    Ok(rows.iter().map(|r| r.get("name")).collect())
}

pub async fn save_reset_code(user_id: i32, code: &str, expires_at: chrono::DateTime<chrono::Utc>) -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    client
        .execute(
            "SELECT sp_auth_reset_code_save($1, $2, $3)",
            &[&user_id, &code, &expires_at],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to save reset code: {e}")))?;

    Ok(())
}

pub async fn verify_reset_code(user_id: i32, code: &str) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_auth_reset_code_verify($1, $2)",
            &[&user_id, &code],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to verify reset code: {e}")))?;

    let matched_id: i32 = row.get(0);
    Ok(matched_id > 0)
}

pub async fn has_unexpired_code(user_id: i32) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_auth_reset_code_has_valid($1)",
            &[&user_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to check reset code: {e}")))?;

    Ok(row.get(0))
}

pub async fn reset_password(user_id: i32, password_hash: &str) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_auth_reset_password($1, $2)",
            &[&user_id, &password_hash],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to reset password: {e}")))?;

    let updated: i32 = row.get(0);
    Ok(updated > 0)
}
