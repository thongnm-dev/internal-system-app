//! Tầng truy cập dữ liệu (data access layer) cho module dự án.
//!
//! Tất cả các thao tác đều gọi stored procedure qua PostgreSQL,
//! không viết SQL trực tiếp trong code Rust.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::project::{ProjectDetail, ProjectMember, ProjectSummary, ProjectTask};
use crate::utils::pgsql_connect;

/// Chuyển một chuỗi rỗng (hoặc chỉ chứa khoảng trắng) thành `None`.
///
/// Dùng cho các cột numeric/optional: PostgreSQL không ép được `""` sang
/// numeric, nên phải gửi `NULL` thay vì chuỗi rỗng.
fn empty_to_none(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Đọc một cột text có thể NULL, trả về chuỗi rỗng nếu là NULL.
///
/// Các trường Backlog là tùy chọn nên có thể NULL trong database; đọc thẳng
/// vào `String` sẽ panic, vì vậy đọc qua `Option<String>` rồi mặc định "".
fn get_opt_text(row: &tokio_postgres::Row, col: &str) -> String {
    row.get::<_, Option<String>>(col).unwrap_or_default()
}

/// Thêm dự án mới vào database.
///
/// Gọi `sp_project_insert` để tạo bản ghi project, sau đó gọi
/// `sp_project_member_upsert` cho từng thành viên.
/// Toàn bộ thao tác chạy trong một transaction.
pub async fn insert_project(project: &ProjectDetail) -> AppResult<ProjectDetail> {
    let client = pgsql_connect::connect().await?;

    // Thông tin Backlog là tùy chọn (có thể không thiết lập). Chuỗi rỗng được
    // gửi thành NULL — riêng `backlog_code` là numeric nên bắt buộc phải NULL.
    let backlog_key = empty_to_none(&project.backlog_key);
    let backlog_code = empty_to_none(&project.backlog_code);
    let backlog_name = empty_to_none(&project.backlog_name);

    pgsql_connect::with_transaction(&client, || async {
        let row = client
            .query_one(
                "SELECT * FROM sp_project_insert($1, $2, $3, $4, $5, $6)",
                &[
                    &project.code,
                    &project.name,
                    &project.client,
                    &backlog_key,
                    &backlog_code,
                    &backlog_name,
                ],
            )
            .await
            .map_err(|e| {
                AppError::new(format!(
                    "Failed to insert project: {}",
                    pgsql_connect::pg_error_detail(&e)
                ))
            })?;

        let id: i32 = row.get("id");

        for member in &project.members {
            client
                .execute(
                    "SELECT sp_project_member_upsert($1, $2, $3)",
                    &[&id, &member.username, &member.name],
                )
                .await
                .map_err(|e| AppError::new(format!("Failed to insert member: {e}")))?;
        }

        Ok(ProjectDetail {
            id,
            code: row.get("code"),
            name: row.get("name"),
            client: row.get("client"),
            backlog_key: get_opt_text(&row, "backlog_key"),
            backlog_code: get_opt_text(&row, "backlog_code"),
            backlog_name: get_opt_text(&row, "backlog_name"),
            is_active: row.get("is_active"),
            members: project.members.clone(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    })
    .await
}

/// Cập nhật thông tin dự án theo ID.
///
/// Gọi `sp_project_update` để cập nhật bản ghi project,
/// xóa toàn bộ member cũ rồi thêm lại danh sách mới.
/// Toàn bộ thao tác chạy trong một transaction.
pub async fn update_project(project_id: i32, project: &ProjectDetail) -> AppResult<ProjectDetail> {
    let client = pgsql_connect::connect().await?;

    let backlog_key = empty_to_none(&project.backlog_key);
    let backlog_code = empty_to_none(&project.backlog_code);
    let backlog_name = empty_to_none(&project.backlog_name);

    pgsql_connect::with_transaction(&client, || async {
        let row = client
            .query_opt(
                "SELECT * FROM sp_project_update($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &project_id,
                    &project.code,
                    &project.name,
                    &project.client,
                    &backlog_key,
                    &backlog_code,
                    &backlog_name,
                ],
            )
            .await
            .map_err(|e| {
                AppError::new(format!(
                    "Failed to update project: {}",
                    pgsql_connect::pg_error_detail(&e)
                ))
            })?
            .ok_or_else(|| AppError::new(format!("Project '{}' not found.", project_id)))?;

        client
            .execute(
                "SELECT sp_project_member_delete_by_project($1)",
                &[&project_id],
            )
            .await
            .map_err(|e| AppError::new(format!("Failed to clear members: {e}")))?;

        for member in &project.members {
            client
                .execute(
                    "SELECT sp_project_member_upsert($1, $2, $3)",
                    &[&project_id, &member.username, &member.name],
                )
                .await
                .map_err(|e| AppError::new(format!("Failed to insert member: {e}")))?;
        }

        Ok(ProjectDetail {
            id: row.get("id"),
            code: row.get("code"),
            name: row.get("name"),
            client: row.get("client"),
            backlog_key: get_opt_text(&row, "backlog_key"),
            backlog_code: get_opt_text(&row, "backlog_code"),
            backlog_name: get_opt_text(&row, "backlog_name"),
            is_active: row.get("is_active"),
            members: project.members.clone(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    })
    .await
}

/// Tìm dự án theo ID, kèm danh sách thành viên.
/// Trả về `None` nếu không tồn tại.
pub async fn find_by_id(project_id: i32) -> AppResult<Option<ProjectDetail>> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_opt(
            "SELECT * FROM sp_project_select_by_id($1)",
            &[&project_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to query project: {e}")))?;

    match row {
        None => Ok(None),
        Some(row) => {
            let id: i32 = row.get("id");
            // Load danh sách thành viên riêng
            let members = load_members(&client, id).await?;
            Ok(Some(ProjectDetail {
                id,
                code: row.get("code"),
                name: row.get("name"),
                client: row.get("client"),
                backlog_key: get_opt_text(&row, "backlog_key"),
                backlog_code: get_opt_text(&row, "backlog_code"),
                backlog_name: get_opt_text(&row, "backlog_name"),
                is_active: row.get("is_active"),
                members,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        }
    }
}

/// Lấy danh sách tóm tắt tất cả dự án, bao gồm số lượng thành viên.
/// Kết quả sắp xếp theo `project_id` tăng dần.
pub async fn list_all() -> AppResult<Vec<ProjectSummary>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query("SELECT * FROM sp_project_select_list()", &[])
        .await
        .map_err(|e| AppError::new(format!("Failed to list projects: {e}")))?;

    let summaries = rows
        .iter()
        .map(|row| ProjectSummary {
            id: row.get("id"),
            code: row.get("code"),
            name: row.get("name"),
            client: row.get("client"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            member_count: row.get("member_count"),
            backlog_key: row.get("backlog_key"),
        })
        .collect();

    Ok(summaries)
}

/// Xóa dự án theo ID. Thành viên sẽ bị xóa tự động nhờ ON DELETE CASCADE.
/// Trả về `true` nếu xóa thành công, `false` nếu không tìm thấy.
pub async fn delete_by_id(project_id: i32) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one("SELECT sp_project_delete($1)", &[&project_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to delete project: {e}")))?;

    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}

/// Kiểm tra mã dự án đã tồn tại hay chưa (case-insensitive).
///
/// `exclude_id`: nếu có, loại trừ dự án có ID này khỏi phép kiểm tra
/// (dùng khi cập nhật để không conflict với chính nó).
pub async fn code_exists(project_code: &str, exclude_id: Option<i32>) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one(
            "SELECT sp_project_select_code_exists($1, $2)",
            &[&project_code, &exclude_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to check project code: {e}")))?;

    Ok(row.get(0))
}

/// Thêm task mới cho dự án, kèm danh sách categories.
/// Chạy trong transaction: insert task → sync categories.
pub async fn insert_task(task: &ProjectTask) -> AppResult<ProjectTask> {
    let client = pgsql_connect::connect().await?;

    let due_date: Option<chrono::NaiveDate> = if task.due_date.is_empty() {
        None
    } else {
        chrono::NaiveDate::parse_from_str(&task.due_date, "%Y-%m-%d").ok()
    };

    pgsql_connect::with_transaction(&client, || async {
        let row = client
            .query_one(
                "SELECT * FROM sp_project_task_insert($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    &task.id,
                    &task.project_id,
                    &task.short_name,
                    &task.description,
                    &task.assignee,
                    &task.estimate_hour,
                    &due_date,
                    &task.issue_key,
                    &task.is_user_added,
                ],
            )
            .await
            .map_err(|e| {
                AppError::new(format!(
                    "Failed to insert project task: {}",
                    pgsql_connect::pg_error_detail(&e)
                ))
            })?;

        if !task.categories.is_empty() {
            client
                .execute(
                    "SELECT sp_project_task_category_sync($1, $2)",
                    &[&task.id, &task.categories],
                )
                .await
                .map_err(|e| AppError::new(format!("Failed to sync task categories: {e}")))?;
        }

        Ok(ProjectTask {
            id: row.get("id"),
            project_id: row.get("project_id"),
            short_name: row.get("short_name"),
            description: row.get("description"),
            assignee: row.get("assignee"),
            estimate_hour: row.get("estimate_hour"),
            due_date: row.get("due_date"),
            issue_key: row.get("issue_key"),
            is_user_added: row.get("is_user_added"),
            categories: task.categories.clone(),
            created_at: row.get("created_at"),
        })
    })
    .await
}

/// Cập nhật task đã có và đồng bộ lại danh sách categories.
pub async fn update_task(task: &ProjectTask) -> AppResult<ProjectTask> {
    let client = pgsql_connect::connect().await?;

    let due_date: Option<chrono::NaiveDate> = if task.due_date.is_empty() {
        None
    } else {
        chrono::NaiveDate::parse_from_str(&task.due_date, "%Y-%m-%d").ok()
    };

    pgsql_connect::with_transaction(&client, || async {
        let row = client
            .query_opt(
                "SELECT * FROM sp_project_task_update($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &task.id,
                    &task.short_name,
                    &task.description,
                    &task.assignee,
                    &task.estimate_hour,
                    &due_date,
                    &task.issue_key,
                ],
            )
            .await
            .map_err(|e| AppError::new(format!("Failed to update project task: {e}")))?;

        let row = row.ok_or_else(|| AppError::new(format!("Task '{}' not found.", task.id)))?;

        client
            .execute(
                "SELECT sp_project_task_category_sync($1, $2)",
                &[&task.id, &task.categories],
            )
            .await
            .map_err(|e| AppError::new(format!("Failed to sync task categories: {e}")))?;

        Ok(ProjectTask {
            id: row.get("id"),
            project_id: row.get("project_id"),
            short_name: row.get("short_name"),
            description: row.get("description"),
            assignee: row.get("assignee"),
            estimate_hour: row.get("estimate_hour"),
            due_date: row.get("due_date"),
            issue_key: row.get("issue_key"),
            is_user_added: row.get("is_user_added"),
            categories: task.categories.clone(),
            created_at: row.get("created_at"),
        })
    })
    .await
}

/// Lấy danh sách task của dự án, kèm categories đã được aggregate thành array.
pub async fn list_tasks_by_project(project_id: i32) -> AppResult<Vec<ProjectTask>> {
    let client = pgsql_connect::connect().await?;

    let rows = client
        .query(
            "SELECT * FROM sp_project_task_select_by_project($1)",
            &[&project_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to list project tasks: {e}")))?;

    Ok(rows
        .iter()
        .map(|row| ProjectTask {
            id: row.get("id"),
            project_id: row.get("project_id"),
            short_name: row.get("short_name"),
            description: row.get("description"),
            assignee: row.get("assignee"),
            estimate_hour: row.get("estimate_hour"),
            due_date: row.get("due_date"),
            issue_key: row.get("issue_key"),
            is_user_added: row.get("is_user_added"),
            categories: row.get("categories"),
            created_at: row.get("created_at"),
        })
        .collect())
}

/// Xóa task theo ID. Categories bị xóa tự động nhờ ON DELETE CASCADE.
pub async fn delete_task(task_id: &str) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;

    let row = client
        .query_one("SELECT sp_project_task_delete($1)", &[&task_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to delete project task: {e}")))?;

    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}

/// Load danh sách thành viên của một dự án, sắp xếp theo username.
async fn load_members(
    client: &tokio_postgres::Client,
    project_id: i32,
) -> AppResult<Vec<ProjectMember>> {
    let rows = client
        .query("SELECT * FROM sp_project_member_select($1)", &[&project_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to load members: {e}")))?;

    Ok(rows
        .iter()
        .map(|row| ProjectMember {
            username: row.get("username"),
            name: row.get("name"),
        })
        .collect())
}
