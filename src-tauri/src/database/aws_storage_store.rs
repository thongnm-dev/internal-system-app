//! Tầng truy cập dữ liệu cho bảng `aws_storage` và `aws_work_folder`.
//!
//! Tất cả các thao tác đều gọi stored procedure qua PostgreSQL,
//! không viết SQL trực tiếp trong code Rust.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::s3::AwsStorage;
use crate::utils::pgsql_connect;

pub async fn list_by_upload() -> AppResult<Vec<AwsStorage>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_aws_storage_select_by_upload()", &[])
        .await
        .map_err(|e| AppError::new(format!("Failed to list upload storages: {e}")))?;

    Ok(rows.iter().map(row_to_storage).collect())
}

pub async fn list_by_download() -> AppResult<Vec<AwsStorage>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_aws_storage_select_by_download()", &[])
        .await
        .map_err(|e| AppError::new(format!("Failed to list download storages: {e}")))?;

    Ok(rows.iter().map(row_to_storage).collect())
}

pub async fn list_by_codes(codes: &[String]) -> AppResult<Vec<AwsStorage>> {
    if codes.is_empty() {
        return Ok(Vec::new());
    }
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_aws_storage_select_by_code_list($1)",
            &[&codes],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to list storages by codes: {e}")))?;

    Ok(rows.iter().map(row_to_storage).collect())
}

pub async fn list_delete_options(destination_code: &str) -> AppResult<Vec<AwsStorage>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_aws_storage_select_by_codes($1)", &[&destination_code])
        .await
        .map_err(|e| AppError::new(format!("Failed to list delete options: {e}")))?;

    Ok(rows.iter().map(row_to_storage).collect())
}

pub async fn get_work_folder_name(folder_key: &str) -> AppResult<String> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt(
            "SELECT * FROM sp_aws_work_folder_get_name($1)",
            &[&folder_key],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query aws_work_folder: {e}")))?
        .ok_or_else(|| {
            AppError::new(format!(
                "Work folder not found for key '{folder_key}'"
            ))
        })?;

    Ok(row.get("name"))
}

pub async fn list_browser_allowed_prefixes(folder_key: &str) -> AppResult<Vec<String>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_aws_storage_select_browser_allowed($1)",
            &[&folder_key],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to list browser-allowed prefixes: {e}")))?;

    Ok(rows.iter().map(|r| r.get("allowed_prefix")).collect())
}

fn row_to_storage(row: &tokio_postgres::Row) -> AwsStorage {
    AwsStorage {
        id: row.get("id"),
        code: row.get("code"),
        name: row.get("name"),
        name_alias: row.get("name_alias"),
        subscribe: row.get("subscribe"),
        is_upload: row.get::<_, Option<bool>>("is_upload").unwrap_or(false),
        is_download: row.get::<_, Option<bool>>("is_download").unwrap_or(false),
        file_only: row.get::<_, Option<bool>>("file_only").unwrap_or(false),
        link_available: row.get::<_, Option<Vec<String>>>("link_available").unwrap_or_default(),
        exclude_subscribe: row.get::<_, Option<Vec<String>>>("exclude_subscribe").unwrap_or_default(),
    }
}
