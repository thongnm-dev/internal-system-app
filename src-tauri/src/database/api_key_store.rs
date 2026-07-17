//! Tầng truy cập dữ liệu cho bảng `api_keys`.
//!
//! Tất cả các thao tác đều gọi stored procedure qua PostgreSQL,
//! không viết SQL trực tiếp trong code Rust.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::utils::pgsql_connect;

pub struct ApiKeyRow {
    pub id: String,
    pub name: String,
    pub key_label: String,
    pub api_key: String,
}

pub async fn list_by_user_id(user_id: i32) -> AppResult<Vec<ApiKeyRow>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_api_key_select_by_user($1)",
            &[&user_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query api_keys: {e}")))?;

    Ok(rows
        .iter()
        .map(|r| ApiKeyRow {
            id: r.get("id"),
            name: r.get("name"),
            key_label: r.get("key_label"),
            api_key: r.get("api_key"),
        })
        .collect())
}

pub async fn sync_for_user(user_id: i32, keys: &[ApiKeyRow]) -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    client
        .execute("SELECT sp_api_key_delete_by_user($1)", &[&user_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to delete old api_keys: {e}")))?;

    for key in keys {
        client
            .execute(
                "SELECT sp_api_key_insert($1, $2, $3, $4, $5)",
                &[&key.id, &user_id, &key.name, &key.key_label, &key.api_key],
            )
            .await
            .map_err(|e| AppError::new(format!("Failed to insert api_key '{}': {e}", key.name)))?;
    }

    Ok(())
}

pub async fn get_value(name: &str, key_label: &str) -> AppResult<String> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt(
            "SELECT * FROM sp_api_key_get_value($1, $2)",
            &[&name, &key_label],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query api_keys: {e}")))?;

    row.map(|r| r.get::<_, String>("api_key"))
        .ok_or_else(|| {
            AppError::new(format!(
                "Không tìm thấy cấu hình '{key_label}' (name='{name}') trong bảng api_keys."
            ))
        })
}
