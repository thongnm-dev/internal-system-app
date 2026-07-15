//! Data access cho bảng `menu_configs` (PostgreSQL).

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::menu_config::{MenuConfig, SaveMenuConfigRequest};
use crate::utils::pgsql_connect;

pub async fn list_all() -> AppResult<Vec<MenuConfig>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_menu_config_select_list()", &[])
        .await
        .map_err(|e| AppError::new(format!("Failed to list menu configs: {e}")))?;

    let items = rows
        .iter()
        .map(|row| MenuConfig {
            key: row.get("key"),
            title: row.get("title"),
            path: row.get("path"),
            icon: row.get("icon"),
            group: row.get("menu_group"),
            visible: row.get("is_visible"),
            order: row.get("display_order"),
        })
        .collect();

    Ok(items)
}

pub async fn upsert(item: &SaveMenuConfigRequest) -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    client
        .execute(
            "SELECT sp_menu_config_upsert($1, $2, $3, $4, $5, $6, $7)",
            &[
                &item.key,
                &item.title,
                &item.path,
                &item.icon,
                &item.group,
                &item.visible,
                &item.order,
            ],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to upsert menu config: {e}")))?;

    Ok(())
}

pub async fn save_all(items: &[SaveMenuConfigRequest]) -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    pgsql_connect::with_transaction(&client, || async {
        client
            .execute("SELECT sp_menu_config_delete_all()", &[])
            .await
            .map_err(|e| AppError::new(format!("Failed to clear menu configs: {e}")))?;

        for item in items {
            client
                .execute(
                    "SELECT sp_menu_config_upsert($1, $2, $3, $4, $5, $6, $7)",
                    &[
                        &item.key,
                        &item.title,
                        &item.path,
                        &item.icon,
                        &item.group,
                        &item.visible,
                        &item.order,
                    ],
                )
                .await
                .map_err(|e| AppError::new(format!("Failed to upsert menu config '{}': {e}", item.key)))?;
        }

        Ok(())
    })
    .await
}
