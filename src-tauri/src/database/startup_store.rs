//! Khởi tạo database khi ứng dụng khởi động.
//!
//! Tạo bảng và stored procedure nếu chưa tồn tại.
//! Được gọi một lần duy nhất trong Tauri setup hook.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::utils::pgsql_connect;
use tokio_postgres::Client;

/// Khởi tạo database: kết nối, tạo bảng, tạo stored procedure, seed dữ liệu.
pub async fn init() -> AppResult<()> {
    let client = pgsql_connect::connect().await?;
    ensure_tables(&client).await?;
    seed_data(&client).await?;
    Ok(())
}

/// Tạo các bảng cần thiết nếu chưa tồn tại, sau đó tạo/cập nhật stored procedure.
async fn ensure_tables(client: &Client) -> AppResult<()> {
    client
        .batch_execute(include_str!("../../../docs/database/init_tables.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create tables: {e}")))?;

    ensure_stored_procedures(client).await?;

    Ok(())
}

/// Seed dữ liệu mặc định: tạo role admin và user admin nếu chưa tồn tại.
async fn seed_data(client: &Client) -> AppResult<()> {
    client
        .batch_execute(include_str!("../../../docs/database/seed_data.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to seed data: {e}")))?;

    Ok(())
}

/// Tạo hoặc cập nhật toàn bộ stored procedure từ các file SQL
/// trong thư mục `docs/store-procedure/`.
///
/// Sử dụng `include_str!` để nhúng nội dung SQL vào binary tại compile-time,
/// đảm bảo stored procedure luôn đồng bộ với mã nguồn.
async fn ensure_stored_procedures(client: &Client) -> AppResult<()> {
    // === Project stored procedures ===

    client
        .batch_execute(include_str!("../../../docs/store-procedure/sp_project_insert.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_insert: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_select_by_id.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_select_by_id: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_select_list.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_select_list: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_select_code_exists.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_project_select_code_exists: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!("../../../docs/store-procedure/sp_project_update.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_update: {e}")))?;

    client
        .batch_execute(include_str!("../../../docs/store-procedure/sp_project_delete.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_delete: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_member_upsert.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_member_upsert: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_member_delete_by_project.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_project_member_delete_by_project: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_member_select.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_member_select: {e}")))?;

    // === Project Task stored procedures ===

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_task_insert.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_task_insert: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_task_category_sync.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_project_task_category_sync: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_task_select_by_project.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_project_task_select_by_project: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_task_delete.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_task_delete: {e}")))?;

    // === Auth stored procedures ===

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_auth_find_user_by_username.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_auth_find_user_by_username: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_auth_get_user_roles.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_auth_get_user_roles: {e}")))?;

    // === Daily Work Notes stored procedures ===

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_insert.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_daily_note_insert: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_select_by_date.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_note_select_by_date: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_select_by_month.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_note_select_by_month: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_count_by_month.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_note_count_by_month: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_update_content.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_note_update_content: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_update_status.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_note_update_status: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_delete.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_daily_note_delete: {e}")))?;

    // === Daily Report stored procedures ===

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_report_entry_upsert.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_report_entry_upsert: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_report_entry_delete.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_report_entry_delete: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_report_entry_select_by_month.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_report_entry_select_by_month: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_report_task_insert.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_report_task_insert: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_report_task_select.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_report_task_select: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_report_task_delete.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_report_task_delete: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_report_project_select.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_report_project_select: {e}"
            ))
        })?;

    // === User management stored procedures ===

    client
        .batch_execute(include_str!("../../../docs/store-procedure/sp_user_insert.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_user_insert: {e}")))?;

    client
        .batch_execute(include_str!("../../../docs/store-procedure/sp_user_update.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_user_update: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_user_select_by_id.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_user_select_by_id: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_user_select_list.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_user_select_list: {e}")))?;

    client
        .batch_execute(include_str!("../../../docs/store-procedure/sp_user_delete.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_user_delete: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_user_change_password.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_user_change_password: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_user_username_exists.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_user_username_exists: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_user_role_sync.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_user_role_sync: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_role_select_list.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_role_select_list: {e}")))?;

    Ok(())
}
