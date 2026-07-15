//! Tầng truy cập dữ liệu cho bảng `api_keys`.

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
            "SELECT id, name, key_label, api_key FROM api_keys WHERE user_id = $1 ORDER BY created_at",
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
        .execute("DELETE FROM api_keys WHERE user_id = $1", &[&user_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to delete old api_keys: {e}")))?;

    for key in keys {
        client
            .execute(
                "INSERT INTO api_keys (id, user_id, name, key_label, api_key) VALUES ($1, $2, $3, $4, $5)",
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
            "SELECT api_key FROM api_keys WHERE name = $1 AND key_label = $2 LIMIT 1",
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
