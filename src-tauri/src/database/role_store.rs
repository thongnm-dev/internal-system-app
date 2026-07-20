use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::role::RoleSummary;
use crate::utils::pgsql_connect;

fn map_row(row: &tokio_postgres::Row) -> RoleSummary {
    RoleSummary {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        user_count: row.get("user_count"),
        created_at: row.get("created_at"),
    }
}

pub async fn list_all() -> AppResult<Vec<RoleSummary>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_role_select_detail_list()", &[])
        .await
        .map_err(|e| AppError::new(format!("Failed to list roles: {e}")))?;

    Ok(rows.iter().map(map_row).collect())
}

pub async fn insert_role(name: &str, description: &str) -> AppResult<RoleSummary> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one("SELECT * FROM sp_role_insert($1, $2)", &[&name, &description])
        .await
        .map_err(|e| AppError::new(format!("Failed to insert role: {e}")))?;

    Ok(map_row(&row))
}

pub async fn update_role(id: i32, name: &str, description: &str) -> AppResult<RoleSummary> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt(
            "SELECT * FROM sp_role_update($1, $2, $3)",
            &[&id, &name, &description],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update role: {e}")))?
        .ok_or_else(|| AppError::new(format!("Role '{id}' not found.")))?;

    Ok(map_row(&row))
}

pub async fn delete_by_id(id: i32) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one("SELECT sp_role_delete($1)", &[&id])
        .await
        .map_err(|e| AppError::new(format!("Failed to delete role: {e}")))?;

    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}

pub async fn name_exists(name: &str, exclude_id: Option<i32>) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one("SELECT sp_role_name_exists($1, $2)", &[&name, &exclude_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to check role name: {e}")))?;

    Ok(row.get(0))
}

pub async fn find_by_id(id: i32) -> AppResult<Option<RoleSummary>> {
    Ok(list_all().await?.into_iter().find(|r| r.id == id))
}
