//! Tầng truy cập dữ liệu (data access layer) cho module dự án.
//!
//! Tất cả các thao tác đều gọi stored procedure qua PostgreSQL,
//! không viết SQL trực tiếp trong code Rust.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::project::{ProjectDetail, ProjectMember, ProjectSummary};
use crate::utils::pgsql_connect;

/// Khởi tạo database: tạo bảng và stored procedure nếu chưa tồn tại.
/// Được gọi một lần khi ứng dụng khởi động.
pub async fn init() -> AppResult<()> {
    let client = pgsql_connect::connect().await?;
    pgsql_connect::ensure_tables(&client).await?;
    Ok(())
}

/// Thêm dự án mới vào database.
///
/// Gọi `sp_project_insert` để tạo bản ghi project, sau đó gọi
/// `sp_project_member_upsert` cho từng thành viên.
/// Trả về `ProjectDetail` với ID và timestamp được tạo bởi database.
pub async fn insert_project(project: &ProjectDetail) -> AppResult<ProjectDetail> {
    let client = pgsql_connect::connect().await?;

    // Gọi stored procedure thêm dự án
    let row = client
        .query_one(
            "SELECT * FROM sp_project_insert($1, $2, $3, $4, $5, $6)",
            &[
                &project.code,
                &project.name,
                &project.client,
                &project.backlog_key,
                &project.backlog_url,
                &project.backlog_space,
            ],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to insert project: {e}")))?;

    let id: i32 = row.get("id");

    // Thêm từng thành viên vào bảng project_members
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
        backlog_key: row.get("backlog_key"),
        backlog_url: row.get("backlog_url"),
        backlog_space: row.get("backlog_space"),
        is_active: row.get("is_active"),
        members: project.members.clone(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

/// Cập nhật thông tin dự án theo ID.
///
/// Gọi `sp_project_update` để cập nhật bản ghi project,
/// xóa toàn bộ member cũ rồi thêm lại danh sách mới.
pub async fn update_project(project_id: i32, project: &ProjectDetail) -> AppResult<ProjectDetail> {
    let client = pgsql_connect::connect().await?;

    // Cập nhật thông tin cơ bản của dự án
    let row = client
        .query_opt(
            "SELECT * FROM sp_project_update($1, $2, $3, $4, $5, $6, $7)",
            &[
                &project_id,
                &project.code,
                &project.name,
                &project.client,
                &project.backlog_key,
                &project.backlog_url,
                &project.backlog_space,
            ],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update project: {e}")))?
        .ok_or_else(|| AppError::new(format!("Project '{}' not found.", project_id)))?;

    // Xóa toàn bộ thành viên cũ
    client
        .execute(
            "SELECT sp_project_member_delete_by_project($1)",
            &[&project_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to clear members: {e}")))?;

    // Thêm lại danh sách thành viên mới
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
        backlog_key: row.get("backlog_key"),
        backlog_url: row.get("backlog_url"),
        backlog_space: row.get("backlog_space"),
        is_active: row.get("is_active"),
        members: project.members.clone(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
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
                backlog_key: row.get("backlog_key"),
                backlog_url: row.get("backlog_url"),
                backlog_space: row.get("backlog_space"),
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
