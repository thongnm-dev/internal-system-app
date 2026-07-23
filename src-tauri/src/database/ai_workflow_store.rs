use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::ai_workflow::{AiWorkflow, AiWorkflowStep};
use crate::utils::pgsql_connect;
use serde_json::Value as JsonValue;

pub async fn insert(
    name: &str,
    description: &str,
    created_by: &str,
) -> AppResult<AiWorkflow> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT * FROM sp_ai_workflow_insert($1, $2, $3)",
            &[&name, &description, &created_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to insert workflow: {e}")))?;

    let layout: JsonValue = row.get("layout");
    Ok(AiWorkflow {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        layout: layout.to_string(),
        created_by: row.get("created_by"),
        step_count: 0,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn select_list(created_by: &str) -> AppResult<Vec<AiWorkflow>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_ai_workflow_select_list($1)",
            &[&created_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query workflows: {e}")))?;

    Ok(rows.iter().map(row_to_workflow).collect())
}

pub async fn update(
    id: i32,
    name: &str,
    description: &str,
    created_by: &str,
) -> AppResult<Option<AiWorkflow>> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt(
            "SELECT * FROM sp_ai_workflow_update($1, $2, $3, $4)",
            &[&id, &name, &description, &created_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update workflow: {e}")))?;

    Ok(row.map(|r| {
        let layout: JsonValue = r.get("layout");
        AiWorkflow {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            layout: layout.to_string(),
            created_by: r.get("created_by"),
            step_count: 0,
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }
    }))
}

pub async fn delete(id: i32, created_by: &str) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_ai_workflow_delete($1, $2)",
            &[&id, &created_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to delete workflow: {e}")))?;

    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}

pub async fn insert_step(
    workflow_id: i32,
    name: &str,
    step_type: &str,
    skill_name: &str,
    description: &str,
    icon: &str,
    step_order: i32,
) -> AppResult<AiWorkflowStep> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT * FROM sp_ai_workflow_step_insert($1, $2, $3, $4, $5, $6, $7)",
            &[&workflow_id, &name, &step_type, &skill_name, &description, &icon, &step_order],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to insert step: {e}")))?;

    Ok(row_to_step(&row))
}

pub async fn select_steps(workflow_id: i32) -> AppResult<Vec<AiWorkflowStep>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_ai_workflow_step_select($1)",
            &[&workflow_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query steps: {e}")))?;

    Ok(rows.iter().map(row_to_step).collect())
}

pub async fn update_step(
    id: i32,
    name: &str,
    step_type: &str,
    skill_name: &str,
    description: &str,
    icon: &str,
    step_order: i32,
) -> AppResult<Option<AiWorkflowStep>> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt(
            "SELECT * FROM sp_ai_workflow_step_update($1, $2, $3, $4, $5, $6, $7)",
            &[&id, &name, &step_type, &skill_name, &description, &icon, &step_order],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update step: {e}")))?;

    Ok(row.map(|r| row_to_step(&r)))
}

pub async fn delete_step(id: i32) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_ai_workflow_step_delete($1)",
            &[&id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to delete step: {e}")))?;

    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}

pub async fn reorder_steps(workflow_id: i32, step_ids: &[i32]) -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    client
        .execute(
            "SELECT sp_ai_workflow_step_reorder($1, $2)",
            &[&workflow_id, &step_ids],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to reorder steps: {e}")))?;

    Ok(())
}

fn row_to_workflow(row: &tokio_postgres::Row) -> AiWorkflow {
    let layout: JsonValue = row.get("layout");
    AiWorkflow {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        layout: layout.to_string(),
        created_by: row.get("created_by"),
        step_count: row.get("step_count"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub async fn update_layout(id: i32, layout_json: &str, created_by: &str) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let layout: JsonValue = serde_json::from_str(layout_json)
        .map_err(|e| AppError::new(format!("Invalid layout JSON: {e}")))?;

    let row = client
        .query_one(
            "SELECT sp_ai_workflow_update_layout($1, $2, $3)",
            &[&id, &layout, &created_by],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update layout: {e}")))?;

    let updated: i32 = row.get(0);
    Ok(updated > 0)
}

fn row_to_step(row: &tokio_postgres::Row) -> AiWorkflowStep {
    AiWorkflowStep {
        id: row.get("id"),
        workflow_id: row.get("workflow_id"),
        name: row.get("name"),
        step_type: row.get("step_type"),
        skill_name: row.get("skill_name"),
        description: row.get("description"),
        icon: row.get("icon"),
        step_order: row.get("step_order"),
        created_at: row.get("created_at"),
    }
}
