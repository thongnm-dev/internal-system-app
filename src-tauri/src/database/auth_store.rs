//! Tầng truy cập dữ liệu cho module xác thực.
//!
//! Tất cả các thao tác đều gọi stored procedure qua PostgreSQL,
//! không viết SQL trực tiếp trong code Rust.

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
