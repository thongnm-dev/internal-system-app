//! Entry point của ứng dụng Tauri.
//!
//! Khởi tạo Tauri Builder, đăng ký plugin, thiết lập database khi khởi động,
//! và đăng ký tất cả các Tauri command handlers cho frontend gọi qua IPC.

include!("modules.rs");

use tauri::Manager;
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
use commands::monthly_report_commands::{
    compare_monthly_report, preview_monthly_report_csv,
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
    backlog_check_config, backlog_create_issue, backlog_get_base_url, backlog_get_issue,
    backlog_get_project, backlog_get_project_lookup, backlog_list_categories,
    backlog_list_issue_types, backlog_list_issues, backlog_list_priorities,
    backlog_list_project_users, backlog_list_statuses,
};
use commands::excel2md_commands::excel2md;
use commands::sync_commands::sync_daily_report;
use commands::collect_commands::{collect_by_folders, collect_load_ini, collect_run};
use commands::explorer_commands::{
    explorer_copy_bugs, explorer_create_file, explorer_create_folder, explorer_delete,
    explorer_get_drives, explorer_open, explorer_paste, explorer_read_dir, explorer_rename,
    explorer_search,
};
use commands::menu_config_commands::{list_menu_configs, save_all_menu_configs, save_menu_config};
use commands::ai_usage_commands::{
    ai_usage_add_account, ai_usage_add_config_dir, ai_usage_capture_add, ai_usage_capture_preview,
    ai_usage_config_dir_preview, ai_usage_delete_account, ai_usage_detect_local,
    ai_usage_get_settings, ai_usage_get_token, ai_usage_import_detected, ai_usage_list_accounts,
    ai_usage_refresh, ai_usage_report_signal, ai_usage_save_settings, ai_usage_set_active,
    ai_usage_update_account,
};
use commands::ai_chat_commands::ai_chat_complete;
use commands::schedule_commands::read_schedule_excel;
use commands::sql_editor_commands::{
    sql_delete_connection, sql_get_schema, sql_list_connections, sql_run_query,
    sql_save_connection, sql_test_connection,
};
use commands::s3_commands::{
    s3_check_config, s3_check_download_available, s3_create_folder, s3_delete_by_storage,
    s3_delete_objects, s3_delete_uploaded_items, s3_download_by_storage, s3_download_objects,
    s3_get_config, s3_get_download_list, s3_list_delete_options, s3_list_download_storages,
    s3_list_objects, s3_list_upload_storages, s3_move_browser_objects, s3_move_objects,
    s3_save_config, s3_scan_local_folder, s3_scan_upload_folder, s3_test_connection,
    s3_upload_file, s3_upload_files, s3_upload_folder,
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.ico"))
                    .expect("failed to load app icon");
                let _ = window.set_icon(icon);
            }

            // Khởi tạo database chạy nền để không block UI thread
            tauri::async_runtime::spawn(async {
                if let Err(e) = database::startup_store::init().await {
                    eprintln!("Failed to initialize database tables: {e}");
                }
            });

            // Nạp trước dữ liệu AI Usage và chạy poll nền để theo dõi usage + auto-switch.
            services::ai_usage_service::preload();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                services::ai_usage_service::run_poll_loop(handle).await;
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
            compare_monthly_report,
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
            backlog_check_config,
            backlog_get_base_url,
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
            s3_check_config,
            s3_get_config,
            s3_save_config,
            s3_test_connection,
            s3_list_objects,
            s3_download_objects,
            s3_upload_file,
            s3_upload_files,
            s3_upload_folder,
            s3_delete_objects,
            s3_create_folder,
            s3_list_upload_storages,
            s3_scan_local_folder,
            s3_scan_upload_folder,
            s3_list_delete_options,
            s3_delete_uploaded_items,
            s3_list_download_storages,
            s3_check_download_available,
            s3_get_download_list,
            s3_download_by_storage,
            s3_move_objects,
            s3_move_browser_objects,
            s3_delete_by_storage,
            // === Sync commands ===
            sync_daily_report,
            // === Collect/Copy tools commands ===
            collect_load_ini,
            collect_run,
            collect_by_folders,
            // === Explorer commands ===
            explorer_read_dir,
            explorer_search,
            explorer_open,
            explorer_get_drives,
            explorer_rename,
            explorer_delete,
            explorer_create_file,
            explorer_create_folder,
            explorer_paste,
            explorer_copy_bugs,
            // === AI Usage commands ===
            ai_usage_add_account,
            ai_usage_detect_local,
            ai_usage_import_detected,
            ai_usage_capture_preview,
            ai_usage_capture_add,
            ai_usage_config_dir_preview,
            ai_usage_add_config_dir,
            ai_usage_list_accounts,
            ai_usage_update_account,
            ai_usage_delete_account,
            ai_usage_set_active,
            ai_usage_get_token,
            ai_usage_report_signal,
            ai_usage_refresh,
            ai_usage_get_settings,
            ai_usage_save_settings,
            // === AI Chat commands ===
            ai_chat_complete,
            // === Schedule commands ===
            read_schedule_excel,
            // === SQL Editor commands ===
            sql_list_connections,
            sql_save_connection,
            sql_delete_connection,
            sql_test_connection,
            sql_get_schema,
            sql_run_query
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
