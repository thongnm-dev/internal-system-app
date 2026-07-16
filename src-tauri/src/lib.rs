//! Entry point của ứng dụng Tauri.
//!
//! Khởi tạo Tauri Builder, đăng ký plugin, thiết lập database khi khởi động,
//! và đăng ký tất cả các Tauri command handlers cho frontend gọi qua IPC.

include!("modules.rs");

use commands::auth_commands::login;
use commands::daily_note_commands::{
    create_daily_note, delete_daily_note, get_daily_note_counts, get_daily_notes_by_date,
    get_daily_notes_by_month, update_daily_note_content, update_daily_note_status,
};
use commands::daily_report_commands::{
    clear_daily_report_entry, create_daily_report_task, delete_daily_report_task,
    get_daily_report_entries, get_daily_report_phases, get_daily_report_projects,
    get_daily_report_task_hours, get_daily_report_tasks, get_task_categories,
    save_daily_report_entry, set_daily_report_task_completed, set_project_task_completed,
};
use commands::db_config_commands::{
    check_database_status, get_database_config, save_database_config, test_database_config,
};
use commands::import_commands::{
    get_import_batch_detail, import_monthly_report_csv, list_import_batches,
    preview_monthly_report_csv, search_import_batches,
};
use commands::project_commands::{
    create_project, create_project_task, delete_project, delete_project_task,
    get_project_detail, list_project_tasks, list_projects,
    update_project, update_project_task,
};
use commands::settings_commands::{get_settings, save_settings};
use commands::system_commands::{check_internet_connection, get_system_info};
use commands::user_commands::{
    change_user_password, create_user, delete_user, get_user_detail, list_roles, list_users,
    update_user,
};
use commands::backlog_commands::{
    backlog_create_issue, backlog_get_issue, backlog_get_project, backlog_get_project_lookup,
    backlog_list_categories, backlog_list_issue_types, backlog_list_issues,
    backlog_list_priorities, backlog_list_project_users, backlog_list_statuses,
};
use commands::excel2md_commands::excel2md;
use commands::menu_config_commands::{list_menu_configs, save_all_menu_configs, save_menu_config};
use commands::s3_commands::{
    s3_create_folder, s3_delete_objects, s3_download_objects, s3_list_objects,
    s3_test_connection, s3_upload_file,
};

/// Khởi chạy ứng dụng Tauri desktop.
///
/// Thứ tự khởi tạo:
/// 1. Đăng ký plugin dialog (cho file picker, message box, v.v.)
/// 2. Setup hook: khởi tạo database (tạo bảng + stored procedure) chạy nền
/// 3. Đăng ký toàn bộ IPC command handlers
/// 4. Chạy event loop của ứng dụng
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            // Khởi tạo database chạy nền để không block UI thread
            tauri::async_runtime::spawn(async {
                if let Err(e) = database::startup_store::init().await {
                    eprintln!("Failed to initialize database tables: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // === Auth commands ===
            login,
            // === System commands ===
            get_system_info,
            check_internet_connection,
            // === Database config commands ===
            check_database_status,
            get_database_config,
            test_database_config,
            save_database_config,
            // === Settings commands ===
            get_settings,
            save_settings,
            // === Import CSV commands ===
            preview_monthly_report_csv,
            import_monthly_report_csv,
            list_import_batches,
            search_import_batches,
            get_import_batch_detail,
            // === Excel → Markdown command ===
            excel2md,
            // === Project CRUD commands ===
            create_project,
            update_project,
            get_project_detail,
            list_projects,
            delete_project,
            // === Project Task commands ===
            create_project_task,
            update_project_task,
            list_project_tasks,
            delete_project_task,
            // === Daily Work Notes commands ===
            create_daily_note,
            get_daily_notes_by_date,
            get_daily_notes_by_month,
            get_daily_note_counts,
            update_daily_note_content,
            update_daily_note_status,
            delete_daily_note,
            // === Daily Report commands ===
            save_daily_report_entry,
            clear_daily_report_entry,
            get_daily_report_entries,
            create_daily_report_task,
            get_daily_report_tasks,
            get_daily_report_task_hours,
            get_daily_report_phases,
            get_task_categories,
            set_daily_report_task_completed,
            set_project_task_completed,
            delete_daily_report_task,
            get_daily_report_projects,
            // === User management commands ===
            create_user,
            update_user,
            get_user_detail,
            list_users,
            delete_user,
            change_user_password,
            list_roles,
            // === Menu config commands ===
            list_menu_configs,
            save_menu_config,
            save_all_menu_configs,
            // === Backlog API commands ===
            backlog_get_project,
            backlog_list_issue_types,
            backlog_list_statuses,
            backlog_list_categories,
            backlog_list_priorities,
            backlog_list_project_users,
            backlog_list_issues,
            backlog_get_issue,
            backlog_get_project_lookup,
            backlog_create_issue,
            // === S3 commands ===
            s3_test_connection,
            s3_list_objects,
            s3_download_objects,
            s3_upload_file,
            s3_delete_objects,
            s3_create_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
