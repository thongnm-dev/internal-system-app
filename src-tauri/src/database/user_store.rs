use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::user::{UserDetail, UserSummary};
use crate::utils::pgsql_connect;

pub async fn insert_user(
    username: &str,
    password_hash: &str,
    full_name: &str,
    email: &str,
    phone: &str,
    address: &str,
    position: &str,
    roles: &[String],
) -> AppResult<UserDetail> {
    let client = pgsql_connect::connect().await?;

    pgsql_connect::with_transaction(&client, || async {
        let row = client
            .query_one(
                "SELECT * FROM sp_user_insert($1, $2, $3, $4, $5, $6, $7)",
                &[&username, &password_hash, &full_name, &email, &phone, &address, &position],
            )
            .await
            .map_err(|e| AppError::new(format!("Failed to insert user: {e}")))?;

        let id: i32 = row.get("id");

        client
            .execute("SELECT sp_user_role_sync($1, $2)", &[&id, &roles])
            .await
            .map_err(|e| AppError::new(format!("Failed to sync roles: {e}")))?;

        let user_roles = load_roles(&client, id).await?;

        Ok(UserDetail {
            id,
            username: row.get("username"),
            full_name: row.get("full_name"),
            email: row.get("email"),
            phone: row.get("phone"),
            address: row.get("address"),
            position: row.get("position"),
            is_active: row.get("is_active"),
            roles: user_roles,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    })
    .await
}

pub async fn update_user(
    user_id: i32,
    full_name: &str,
    email: &str,
    phone: &str,
    address: &str,
    position: &str,
    is_active: bool,
    roles: &[String],
) -> AppResult<UserDetail> {
    let client = pgsql_connect::connect().await?;

    pgsql_connect::with_transaction(&client, || async {
        let row = client
            .query_opt(
                "SELECT * FROM sp_user_update($1, $2, $3, $4, $5, $6, $7)",
                &[&user_id, &full_name, &email, &phone, &address, &position, &is_active],
            )
            .await
            .map_err(|e| AppError::new(format!("Failed to update user: {e}")))?
            .ok_or_else(|| AppError::new(format!("User '{user_id}' not found.")))?;

        client
            .execute("SELECT sp_user_role_sync($1, $2)", &[&user_id, &roles])
            .await
            .map_err(|e| AppError::new(format!("Failed to sync roles: {e}")))?;

        let user_roles = load_roles(&client, user_id).await?;

        Ok(UserDetail {
            id: row.get("id"),
            username: row.get("username"),
            full_name: row.get("full_name"),
            email: row.get("email"),
            phone: row.get("phone"),
            address: row.get("address"),
            position: row.get("position"),
            is_active: row.get("is_active"),
            roles: user_roles,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    })
    .await
}

pub async fn find_by_id(user_id: i32) -> AppResult<Option<UserDetail>> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt("SELECT * FROM sp_user_select_by_id($1)", &[&user_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to query user: {e}")))?;

    match row {
        None => Ok(None),
        Some(row) => {
            let id: i32 = row.get("id");
            let roles = load_roles(&client, id).await?;
            Ok(Some(UserDetail {
                id,
                username: row.get("username"),
                full_name: row.get("full_name"),
                email: row.get("email"),
                phone: row.get("phone"),
                address: row.get("address"),
                position: row.get("position"),
                is_active: row.get("is_active"),
                roles,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        }
    }
}

pub async fn list_all() -> AppResult<Vec<UserSummary>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_user_select_list()", &[])
        .await
        .map_err(|e| AppError::new(format!("Failed to list users: {e}")))?;

    let users = rows
        .iter()
        .map(|row| {
            let roles_csv: String = row.get("roles");
            let roles: Vec<String> = if roles_csv.is_empty() {
                vec![]
            } else {
                roles_csv.split(',').map(|s| s.to_string()).collect()
            };
            UserSummary {
                id: row.get("id"),
                username: row.get("username"),
                full_name: row.get("full_name"),
                email: row.get("email"),
                phone: row.get("phone"),
                position: row.get("position"),
                is_active: row.get("is_active"),
                roles,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect();

    Ok(users)
}

pub async fn delete_by_id(user_id: i32) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one("SELECT sp_user_delete($1)", &[&user_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to delete user: {e}")))?;

    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}

pub async fn change_password(user_id: i32, password_hash: &str) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_user_change_password($1, $2)",
            &[&user_id, &password_hash],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to change password: {e}")))?;

    let updated: i32 = row.get(0);
    Ok(updated > 0)
}

pub async fn username_exists(username: &str, exclude_id: Option<i32>) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_user_username_exists($1, $2)",
            &[&username, &exclude_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to check username: {e}")))?;

    Ok(row.get(0))
}

pub async fn list_roles() -> AppResult<Vec<String>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_role_select_list()", &[])
        .await
        .map_err(|e| AppError::new(format!("Failed to list roles: {e}")))?;

    Ok(rows.iter().map(|r| r.get("name")).collect())
}

pub async fn get_staff_no(username: &str) -> AppResult<String> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt(
            "SELECT staff_no FROM users WHERE username = $1",
            &[&username],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query staff_no: {e}")))?
        .ok_or_else(|| AppError::new(format!("User '{username}' not found.")))?;

    let staff_no: Option<String> = row.get("staff_no");
    staff_no.filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::new(format!("Staff No is not set for user '{username}'.")))
}

async fn load_roles(
    client: &tokio_postgres::Client,
    user_id: i32,
) -> AppResult<Vec<String>> {
    let rows = client
        .query("SELECT * FROM sp_auth_get_user_roles($1)", &[&user_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to load user roles: {e}")))?;

    Ok(rows.iter().map(|r| r.get("name")).collect())
}
