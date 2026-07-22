use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::s3::{UploadHistorySearchItem, UploadHistorySearchParams};
use crate::utils::pgsql_connect;

pub struct UploadFileDetail {
    pub bug_no: String,
    pub file_name: String,
    pub file_path: String,
}

pub async fn insert_upload(
    aws_cd: &str,
    date_ymd: &str,
    time_hms: &str,
    user_id: &str,
    is_moved_at_s3: bool,
    details: &[UploadFileDetail],
) -> AppResult<i32> {
    let client = pgsql_connect::connect().await?;

    client
        .batch_execute("BEGIN")
        .await
        .map_err(|e| AppError::new(format!("Failed to begin transaction: {e}")))?;

    let bug_nos: Vec<&str> = details
        .iter()
        .map(|d| d.bug_no.as_str())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let upload_count = bug_nos.len() as i32;

    let row = match client
        .query_one(
            "SELECT * FROM sp_upload_hdr_insert($1, $2, $3, $4, $5, $6)",
            &[&date_ymd, &time_hms, &aws_cd, &upload_count, &user_id, &is_moved_at_s3],
        )
        .await
    {
        Ok(row) => row,
        Err(e) => {
            let _ = client.batch_execute("ROLLBACK").await;
            return Err(AppError::new(format!("Failed to insert upload_hdr: {e}")));
        }
    };

    let upload_id: i32 = row.get("id");

    let mut dtl_ids: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

    for bug_no in &bug_nos {
        let dtl_row = match client
            .query_one(
                "SELECT * FROM sp_upload_dtl_insert($1, $2)",
                &[&upload_id, bug_no],
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let _ = client.batch_execute("ROLLBACK").await;
                return Err(AppError::new(format!("Failed to insert upload_dtl: {e}")));
            }
        };
        dtl_ids.insert(bug_no.to_string(), dtl_row.get("id"));
    }

    for dtl in details {
        let dtl_id = dtl_ids
            .get(&dtl.bug_no)
            .copied()
            .unwrap_or(0);
        if let Err(e) = client
            .execute(
                "SELECT sp_upload_attach_insert($1, $2, $3, $4)",
                &[&upload_id, &dtl_id, &dtl.file_name, &dtl.file_path],
            )
            .await
        {
            let _ = client.batch_execute("ROLLBACK").await;
            return Err(AppError::new(format!("Failed to insert upload_attach: {e}")));
        }
    }

    client
        .batch_execute("COMMIT")
        .await
        .map_err(|e| AppError::new(format!("Failed to commit: {e}")))?;

    Ok(upload_id)
}

pub async fn search_upload_history(
    params: &UploadHistorySearchParams,
) -> AppResult<Vec<UploadHistorySearchItem>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_upload_history_search($1, $2, $3, $4, $5)",
            &[
                &params.from_date,
                &params.to_date,
                &params.aws_cd,
                &params.bug_no,
                &params.is_moved_at_s3,
            ],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to search upload history: {e}")))?;

    Ok(rows
        .iter()
        .map(|row| UploadHistorySearchItem {
            upload_ymd: row.get("upload_ymd"),
            aws_name: row.get("aws_name"),
            is_moved_at_s3: row
                .get::<_, Option<bool>>("is_moved_at_s3")
                .unwrap_or(false),
            bug_no: row.get("bug_no"),
            att_files: row
                .get::<_, Option<String>>("att_files")
                .unwrap_or_default(),
        })
        .collect())
}
