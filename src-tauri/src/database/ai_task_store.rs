use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::ai_task::{AiTask, AiTaskWfProc, AiTaskWfProcStep};
use crate::utils::pgsql_connect;

// ── ai_tasks ────────────────────────────────────────────────────────────

pub async fn insert(
    task_cd: &str,
    task_name: &str,
    category: &str,
    created_by: &str,
) -> AppResult<AiTask> {
    let client = pgsql_connect::connect().await?;
    let row = client
        .query_one(
            "SELECT * FROM sp_ai_task_insert($1, $2, $3, $4)",
            &[&task_cd, &task_name, &category, &created_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to insert task: {e}")))?;
    Ok(row_to_task(&row))
}

pub async fn select_list(keyword: Option<&str>, is_complete: Option<bool>) -> AppResult<Vec<AiTask>> {
    let client = pgsql_connect::connect().await?;
    let rows = client
        .query("SELECT * FROM sp_ai_task_select_list($1, $2)", &[&keyword, &is_complete])
        .await
        .map_err(|e| AppError::new(format!("Failed to query tasks: {e}")))?;
    Ok(rows.iter().map(row_to_task).collect())
}

pub async fn update(
    id: i32,
    task_cd: &str,
    task_name: &str,
    category: &str,
    is_complete: bool,
    updated_by: &str,
) -> AppResult<AiTask> {
    let client = pgsql_connect::connect().await?;
    let row = client
        .query_one(
            "SELECT * FROM sp_ai_task_update($1, $2, $3, $4, $5, $6)",
            &[&id, &task_cd, &task_name, &category, &is_complete, &updated_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update task: {e}")))?;
    Ok(row_to_task(&row))
}

fn row_to_task(row: &tokio_postgres::Row) -> AiTask {
    AiTask {
        id: row.get("id"),
        task_cd: row.get("task_cd"),
        task_name: row.get("task_name"),
        category: row.get("category"),
        is_complete: row.get("is_complete"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
        created_by: row.get("created_by"),
        updated_at: row.get("updated_at"),
        updated_by: row.get("updated_by"),
    }
}

// ── ai_task_wf_proc ─────────────────────────────────────────────────────

pub async fn wf_proc_insert(
    task_id: i32,
    wf_id: i32,
    created_by: &str,
) -> AppResult<AiTaskWfProc> {
    let client = pgsql_connect::connect().await?;
    let row = client
        .query_one(
            "SELECT * FROM sp_ai_task_wf_proc_insert($1, $2, $3)",
            &[&task_id, &wf_id, &created_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to insert wf_proc: {e}")))?;
    Ok(row_to_wf_proc(&row))
}

pub async fn wf_proc_select_by_task(task_id: i32) -> AppResult<Vec<AiTaskWfProc>> {
    let client = pgsql_connect::connect().await?;
    let rows = client
        .query(
            "SELECT * FROM sp_ai_task_wf_proc_select_by_task($1)",
            &[&task_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query wf_proc: {e}")))?;
    Ok(rows.iter().map(row_to_wf_proc).collect())
}

pub async fn wf_proc_update(
    id: i32,
    latest_step_id: i32,
    updated_by: &str,
) -> AppResult<AiTaskWfProc> {
    let client = pgsql_connect::connect().await?;
    let row = client
        .query_one(
            "SELECT * FROM sp_ai_task_wf_proc_update($1, $2, $3)",
            &[&id, &latest_step_id, &updated_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update wf_proc: {e}")))?;
    Ok(row_to_wf_proc(&row))
}

fn row_to_wf_proc(row: &tokio_postgres::Row) -> AiTaskWfProc {
    AiTaskWfProc {
        id: row.get("id"),
        task_id: row.get("task_id"),
        wf_id: row.get("wf_id"),
        latest_step_id: row.get("latest_step_id"),
        created_at: row.get("created_at"),
        created_by: row.get("created_by"),
        updated_at: row.get("updated_at"),
        updated_by: row.get("updated_by"),
    }
}

// ── ai_task_wf_proc_step ────────────────────────────────────────────────

pub async fn wf_proc_step_insert(
    wf_proc_id: i32,
    wf_step_id: i32,
    status: &str,
    created_by: &str,
) -> AppResult<AiTaskWfProcStep> {
    let client = pgsql_connect::connect().await?;
    let row = client
        .query_one(
            "SELECT * FROM sp_ai_task_wf_proc_step_insert($1, $2, $3, $4)",
            &[&wf_proc_id, &wf_step_id, &status, &created_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to insert wf_proc_step: {e}")))?;
    Ok(row_to_wf_proc_step(&row))
}

pub async fn wf_proc_step_select_by_proc(wf_proc_id: i32) -> AppResult<Vec<AiTaskWfProcStep>> {
    let client = pgsql_connect::connect().await?;
    let rows = client
        .query(
            "SELECT * FROM sp_ai_task_wf_proc_step_select_by_proc($1)",
            &[&wf_proc_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query wf_proc_step: {e}")))?;
    Ok(rows.iter().map(row_to_wf_proc_step).collect())
}

pub async fn wf_proc_step_update(
    id: i32,
    status: &str,
    updated_by: &str,
) -> AppResult<AiTaskWfProcStep> {
    let client = pgsql_connect::connect().await?;
    let row = client
        .query_one(
            "SELECT * FROM sp_ai_task_wf_proc_step_update($1, $2, $3)",
            &[&id, &status, &updated_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update wf_proc_step: {e}")))?;
    Ok(row_to_wf_proc_step(&row))
}

fn row_to_wf_proc_step(row: &tokio_postgres::Row) -> AiTaskWfProcStep {
    AiTaskWfProcStep {
        id: row.get("id"),
        wf_proc_id: row.get("wf_proc_id"),
        wf_step_id: row.get("wf_step_id"),
        status: row.get("status"),
        created_at: row.get("created_at"),
        created_by: row.get("created_by"),
        updated_at: row.get("updated_at"),
        updated_by: row.get("updated_by"),
    }
}
