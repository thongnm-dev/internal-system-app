use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::ai_task::AiTask;
use crate::utils::pgsql_connect;

pub async fn insert(
    task_code: &str,
    category: &str,
    description: &str,
    created_by: &str,
) -> AppResult<AiTask> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT * FROM sp_ai_task_insert($1, $2, $3, $4)",
            &[&task_code, &category, &description, &created_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to insert task: {e}")))?;

    Ok(row_to_task(&row))
}

pub async fn select_list(keyword: Option<&str>) -> AppResult<Vec<AiTask>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_ai_task_select_list($1)", &[&keyword])
        .await
        .map_err(|e| AppError::new(format!("Failed to query tasks: {e}")))?;

    Ok(rows.iter().map(row_to_task).collect())
}

fn row_to_task(row: &tokio_postgres::Row) -> AiTask {
    AiTask {
        id: row.get("id"),
        task_code: row.get("task_code"),
        category: row.get("category"),
        description: row.get("description"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    }
}
