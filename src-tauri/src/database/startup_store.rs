//! Khởi tạo database khi ứng dụng khởi động.
//!
//! Tạo bảng và stored procedure nếu chưa tồn tại.
//! Được gọi một lần duy nhất trong Tauri setup hook.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::utils::pgsql_connect;
use tokio_postgres::Client;

/// Khởi tạo database: kết nối, tạo bảng, tạo stored procedure, seed dữ liệu.
///
/// Tạo bảng trước (fail-fast nếu lỗi), sau đó chạy stored procedures và seed data
/// độc lập — nếu một số SP fail thì các SP còn lại và seed data vẫn được thực thi.
pub async fn init() -> AppResult<()> {
    let client = pgsql_connect::connect().await?;

    client
        .batch_execute(include_str!("../../../docs/database/schema.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create tables: {e}")))?;

    let mut errors = Vec::new();

    if let Err(e) = ensure_stored_procedures(&client).await {
        errors.push(e.to_string());
    }

    if let Err(e) = seed_data(&client).await {
        errors.push(e.to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::new(errors.join("\n")))
    }
}

/// Seed dữ liệu mặc định: tạo role admin và user admin nếu chưa tồn tại.
async fn seed_data(client: &Client) -> AppResult<()> {
    client
        .batch_execute(include_str!("../../../docs/database/seed_data.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to seed data: {e}")))?;

    Ok(())
}

/// Tạo hoặc cập nhật toàn bộ stored procedure từ các file SQL.
///
/// Thực thi tất cả SP và thu thập lỗi thay vì dừng lại ở SP đầu tiên bị lỗi.
async fn ensure_stored_procedures(client: &Client) -> AppResult<()> {
    let procedures: &[(&str, &str)] = &[
        // === Project ===
        ("sp_project_insert", include_str!("../../../docs/store-procedure/sp_project_insert.sql")),
        ("sp_project_select_by_id", include_str!("../../../docs/store-procedure/sp_project_select_by_id.sql")),
        ("sp_project_select_list", include_str!("../../../docs/store-procedure/sp_project_select_list.sql")),
        ("sp_project_select_code_exists", include_str!("../../../docs/store-procedure/sp_project_select_code_exists.sql")),
        ("sp_project_update", include_str!("../../../docs/store-procedure/sp_project_update.sql")),
        ("sp_project_delete", include_str!("../../../docs/store-procedure/sp_project_delete.sql")),
        ("sp_project_member_upsert", include_str!("../../../docs/store-procedure/sp_project_member_upsert.sql")),
        ("sp_project_member_delete_by_project", include_str!("../../../docs/store-procedure/sp_project_member_delete_by_project.sql")),
        ("sp_project_member_select", include_str!("../../../docs/store-procedure/sp_project_member_select.sql")),
        // === Project Task ===
        ("sp_project_task_insert", include_str!("../../../docs/store-procedure/sp_project_task_insert.sql")),
        ("sp_project_task_category_sync", include_str!("../../../docs/store-procedure/sp_project_task_category_sync.sql")),
        ("sp_project_task_select_by_project", include_str!("../../../docs/store-procedure/sp_project_task_select_by_project.sql")),
        ("sp_project_task_update", include_str!("../../../docs/store-procedure/sp_project_task_update.sql")),
        ("sp_project_task_delete", include_str!("../../../docs/store-procedure/sp_project_task_delete.sql")),
        ("sp_project_task_set_completed", include_str!("../../../docs/store-procedure/sp_project_task_set_completed.sql")),
        // === Auth ===
        ("sp_auth_find_user_by_username", include_str!("../../../docs/store-procedure/sp_auth_find_user_by_username.sql")),
        ("sp_auth_get_user_roles", include_str!("../../../docs/store-procedure/sp_auth_get_user_roles.sql")),
        // === Daily Work Notes ===
        ("sp_daily_note_insert", include_str!("../../../docs/store-procedure/sp_daily_note_insert.sql")),
        ("sp_daily_note_select_by_date", include_str!("../../../docs/store-procedure/sp_daily_note_select_by_date.sql")),
        ("sp_daily_note_select_by_month", include_str!("../../../docs/store-procedure/sp_daily_note_select_by_month.sql")),
        ("sp_daily_note_count_by_month", include_str!("../../../docs/store-procedure/sp_daily_note_count_by_month.sql")),
        ("sp_daily_note_update_content", include_str!("../../../docs/store-procedure/sp_daily_note_update_content.sql")),
        ("sp_daily_note_update_status", include_str!("../../../docs/store-procedure/sp_daily_note_update_status.sql")),
        ("sp_daily_note_delete", include_str!("../../../docs/store-procedure/sp_daily_note_delete.sql")),
        // === Daily Report ===
        ("sp_daily_report_entry_upsert", include_str!("../../../docs/store-procedure/sp_daily_report_entry_upsert.sql")),
        ("sp_daily_report_entry_delete", include_str!("../../../docs/store-procedure/sp_daily_report_entry_delete.sql")),
        ("sp_daily_report_entry_select_by_month", include_str!("../../../docs/store-procedure/sp_daily_report_entry_select_by_month.sql")),
        ("sp_daily_report_task_insert", include_str!("../../../docs/store-procedure/sp_daily_report_task_insert.sql")),
        ("sp_daily_report_task_select", include_str!("../../../docs/store-procedure/sp_daily_report_task_select.sql")),
        ("sp_daily_report_task_delete", include_str!("../../../docs/store-procedure/sp_daily_report_task_delete.sql")),
        ("sp_daily_report_task_set_completed", include_str!("../../../docs/store-procedure/sp_daily_report_task_set_completed.sql")),
        ("sp_daily_report_task_hours_total", include_str!("../../../docs/store-procedure/sp_daily_report_task_hours_total.sql")),
        ("sp_daily_report_project_select", include_str!("../../../docs/store-procedure/sp_daily_report_project_select.sql")),
        // === Category ===
        ("sp_category_select_list", include_str!("../../../docs/store-procedure/sp_category_select_list.sql")),
        ("sp_category_select_task_list", include_str!("../../../docs/store-procedure/sp_category_select_task_list.sql")),
        // === User Management ===
        ("sp_user_insert", include_str!("../../../docs/store-procedure/sp_user_insert.sql")),
        ("sp_user_update", include_str!("../../../docs/store-procedure/sp_user_update.sql")),
        ("sp_user_select_by_id", include_str!("../../../docs/store-procedure/sp_user_select_by_id.sql")),
        ("sp_user_select_list", include_str!("../../../docs/store-procedure/sp_user_select_list.sql")),
        ("sp_user_delete", include_str!("../../../docs/store-procedure/sp_user_delete.sql")),
        ("sp_user_change_password", include_str!("../../../docs/store-procedure/sp_user_change_password.sql")),
        ("sp_user_username_exists", include_str!("../../../docs/store-procedure/sp_user_username_exists.sql")),
        ("sp_user_role_sync", include_str!("../../../docs/store-procedure/sp_user_role_sync.sql")),
        ("sp_role_select_list", include_str!("../../../docs/store-procedure/sp_role_select_list.sql")),
        // === Menu Config ===
        ("sp_menu_config_select_list", include_str!("../../../docs/store-procedure/sp_menu_config_select_list.sql")),
        ("sp_menu_config_upsert", include_str!("../../../docs/store-procedure/sp_menu_config_upsert.sql")),
        ("sp_menu_config_delete_all", include_str!("../../../docs/store-procedure/sp_menu_config_delete_all.sql")),
        // === API Keys ===
        ("sp_api_key_select_by_user", include_str!("../../../docs/store-procedure/sp_api_key_select_by_user.sql")),
        ("sp_api_key_delete_by_user", include_str!("../../../docs/store-procedure/sp_api_key_delete_by_user.sql")),
        ("sp_api_key_insert", include_str!("../../../docs/store-procedure/sp_api_key_insert.sql")),
        ("sp_api_key_get_value", include_str!("../../../docs/store-procedure/sp_api_key_get_value.sql")),
        // === AWS Storage ===
        ("sp_aws_storage_select_by_upload", include_str!("../../../docs/store-procedure/sp_aws_storage_select_by_upload.sql")),
        ("sp_aws_storage_select_by_download", include_str!("../../../docs/store-procedure/sp_aws_storage_select_by_download.sql")),
        ("sp_aws_storage_select_by_codes", include_str!("../../../docs/store-procedure/sp_aws_storage_select_by_codes.sql")),
        ("sp_aws_storage_select_by_code_list", include_str!("../../../docs/store-procedure/sp_aws_storage_select_by_code_list.sql")),
        ("sp_aws_work_folder_get_name", include_str!("../../../docs/store-procedure/sp_aws_work_folder_get_name.sql")),
    ];

    let mut errors = Vec::new();

    for (name, sql) in procedures {
        if let Err(e) = client.batch_execute(sql).await {
            errors.push(format!("Failed to create {name}: {e}"));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::new(errors.join("\n")))
    }
}
