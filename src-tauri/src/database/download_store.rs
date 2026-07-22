use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::s3::{DownloadHistoryItem, DownloadHistorySearchItem, DownloadHistorySearchParams};
use crate::utils::pgsql_connect;

pub struct DownloadDetail {
    pub bug_no: String,
    pub sync_path: String,
}

pub async fn insert_download(
    aws_cd: &str,
    date_ymd: &str,
    time_hms: &str,
    user_id: &str,
    sync_path: &str,
    details: &[DownloadDetail],
) -> AppResult<i32> {
    let client = pgsql_connect::connect().await?;

    client
        .batch_execute("BEGIN")
        .await
        .map_err(|e| AppError::new(format!("Failed to begin transaction: {e}")))?;

    let count = details.len() as i32;
    let row = match client
        .query_one(
            "SELECT * FROM sp_download_hdr_insert($1, $2, $3, $4, $5, $6)",
            &[&date_ymd, &time_hms, &aws_cd, &sync_path, &count, &user_id],
        )
        .await
    {
        Ok(row) => row,
        Err(e) => {
            let _ = client.batch_execute("ROLLBACK").await;
            return Err(AppError::new(format!("Failed to insert download_hdr: {e}")));
        }
    };

    let download_id: i32 = row.get("id");

    for dtl in details {
        if let Err(e) = client
            .execute(
                "SELECT sp_download_dtl_insert($1, $2, $3)",
                &[&download_id, &dtl.bug_no, &dtl.sync_path],
            )
            .await
        {
            let _ = client.batch_execute("ROLLBACK").await;
            return Err(AppError::new(format!("Failed to insert download_dtl: {e}")));
        }
    }

    client
        .batch_execute("COMMIT")
        .await
        .map_err(|e| AppError::new(format!("Failed to commit: {e}")))?;

    Ok(download_id)
}

pub async fn search_download_history(
    params: &DownloadHistorySearchParams,
) -> AppResult<Vec<DownloadHistorySearchItem>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_download_history_search($1, $2, $3, $4, $5, $6)",
            &[
                &params.from_date,
                &params.to_date,
                &params.aws_cd,
                &params.bug_no,
                &params.is_moved_at_local,
                &params.is_moved_at_s3,
            ],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to search download history: {e}")))?;

    Ok(rows
        .iter()
        .map(|row| DownloadHistorySearchItem {
            id: row.get("id"),
            download_ymd: row.get("download_ymd"),
            aws_cd: row.get("aws_cd"),
            aws_name: row.get("aws_name"),
            is_moved_at_local: row
                .get::<_, Option<bool>>("is_moved_at_local")
                .unwrap_or(false),
            bug_no: row.get("bug_no"),
        })
        .collect())
}

pub async fn get_download_history(user_id: &str) -> AppResult<Vec<DownloadHistoryItem>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_download_history_select($1)",
            &[&user_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to get download history: {e}")))?;

    Ok(rows
        .iter()
        .map(|row| DownloadHistoryItem {
            id: row.get("id"),
            download_ymd: row.get("download_ymd"),
            download_hms: row.get("download_hms"),
            sync_path: row.get("sync_path"),
            download_count: row.get("download_count"),
            is_moved_at_local: row
                .get::<_, Option<bool>>("is_moved_at_local")
                .unwrap_or(false),
            aws_name: row.get("aws_name"),
            aws_name_alias: row
                .get::<_, Option<String>>("aws_name_alias")
                .unwrap_or_default(),
        })
        .collect())
}

pub async fn update_moved_at_local(id: i32, path_copied: &str) -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    client
        .execute(
            "SELECT sp_download_hdr_update_moved_local($1, $2)",
            &[&id, &path_copied],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update is_moved_at_local: {e}")))?;

    Ok(())
}

pub async fn update_moved_at_s3(aws_cd: &str, bug_nos: &[String]) -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    client
        .execute(
            "SELECT sp_download_dtl_update_moved_s3($1, $2)",
            &[&aws_cd, &bug_nos],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update is_moved_at_s3: {e}")))?;

    Ok(())
}
