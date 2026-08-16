// Danh sách toàn bộ Tauri command handlers, tách riêng khỏi `lib.rs` cho gọn.
//
// `tauri::generate_handler!` chỉ nhận được MỘT danh sách token duy nhất tại chỗ gọi
// (không thể chia thành nhiều lời gọi macro rồi gộp lại), nên toàn bộ danh sách vẫn
// nằm trong một khối — nhưng được gom về đây thay vì làm phình `lib.rs`.

use commands::auth_commands::*;
use commands::daily_note_commands::*;
use commands::daily_report_commands::*;
use commands::db_config_commands::*;
use commands::monthly_report_commands::*;
use commands::project_commands::*;
use commands::settings_commands::*;
use commands::system_commands::*;
use commands::user_commands::*;
use commands::role_commands::*;
use commands::backlog_commands::*;
use commands::excel2md_commands::*;
use commands::excel_helper_commands::*;
use commands::file_compare_commands::*;
use commands::vnjp_sync_commands::*;
use commands::file_split_commands::*;
use commands::issue_csv_commands::*;
use commands::sync_commands::*;
use commands::collect_commands::*;
use commands::explorer_commands::*;
use commands::git_commands::*;
use commands::menu_config_commands::*;
use commands::menu_permission_commands::*;
use commands::ai_usage_commands::*;
use commands::ai_chat_commands::*;
use commands::ai_workflow_commands::*;
use commands::ai_task_commands::*;
use commands::ai_translate_cowork_commands::*;
use commands::ai_cowork_commands::*;
use commands::terminal_commands::*;
use commands::schedule_commands::*;
use commands::sql_editor_commands::*;
use commands::app_config_commands::*;
use commands::pagination_commands::*;
use commands::s3_commands::*;

/// Xây dựng handler cho `Builder::invoke_handler`, gộp toàn bộ command đã đăng ký.
///
/// Dùng thẳng runtime `tauri::Wry` (desktop) thay vì generic `R: Runtime`, vì một số
/// command nhận tham số kiểu cụ thể `AppHandle`/`Window` (mặc định là `AppHandle<Wry>`)
/// — nếu để generic, trình biên dịch không thể suy ra `AppHandle<Wry>: CommandArg<'_, R>`.
fn build_invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // === Auth commands ===
        login,
        request_password_reset,
        verify_password_reset,
        // === System commands ===
        get_system_info,
        check_internet_connection,
        check_internal_connection,
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
        fetch_csv_from_system,
        parse_issue_csv,
        // === Excel → Markdown command ===
        excel2md,
        // === File split (zip AES-256 + cắt .001) command ===
        file_split_run,
        file_split_calc_size,
        // === File compare (text/markdown/word/excel diff) command ===
        file_compare_run,
        file_compare_export,
        // === VN→JP document sync commands ===
        vnjp_sync_analyze,
        vnjp_sync_translate,
        vnjp_sync_export_report,
        vnjp_sync_apply,
        // === Resize evidence images command ===
        list_excel_sheet_names,
        resize_excel_images,
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
        get_staff_no,
        list_roles,
        // === Role management commands ===
        list_role_details,
        create_role,
        update_role,
        delete_role,
        // === Menu config commands ===
        list_menu_configs,
        save_menu_config,
        save_all_menu_configs,
        // === Menu permission commands ===
        list_role_menu_permissions,
        save_role_menu_permissions,
        list_user_menu_permissions,
        save_user_menu_permissions,
        list_effective_menu_permissions,
        // === Backlog API commands ===
        backlog_check_config,
        backlog_get_config,
        backlog_save_config,
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
        s3_get_local_sync_workdir,
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
        s3_scan_upload_folders,
        s3_list_all_bug_folders,
        s3_list_all_bug_folders_by_code,
        s3_list_bug_folder_tabs,
        s3_list_delete_options,
        s3_delete_uploaded_items,
        s3_list_download_storages,
        s3_check_download_available,
        s3_get_download_list,
        s3_download_by_storage,
        s3_move_objects,
        s3_move_browser_objects,
        s3_delete_by_storage,
        s3_get_download_history,
        s3_search_download_history,
        s3_update_download_moved_local,
        s3_search_upload_history,
        s3_get_browser_allowed_prefixes,
        // === Sync commands ===
        sync_daily_report,
        // === Collect/Copy tools commands ===
        collect_load_ini,
        collect_run,
        collect_by_folders,
        collect_scan_duplicates,
        // === Explorer commands ===
        explorer_read_dir,
        explorer_search,
        explorer_open,
        explorer_open_file,
        explorer_read_text_file,
        explorer_read_file_base64,
        explorer_get_drives,
        explorer_rename,
        explorer_delete,
        explorer_create_file,
        explorer_create_folder,
        explorer_ensure_dir,
        explorer_paste,
        explorer_paste_from_os_clipboard,
        explorer_copy_bugs,
        // === Git Desktop commands ===
        git_list_repos,
        git_add_repo,
        git_remove_repo,
        git_touch_repo,
        git_repo_info,
        git_status,
        git_file_diff,
        git_commit_file_diff,
        git_log,
        git_log_search,
        git_graph,
        git_commit_detail,
        git_branches,
        git_stash_list,
        git_stage,
        git_unstage,
        git_discard,
        git_commit,
        git_amend_commit,
        git_checkout_branch,
        git_create_branch,
        git_delete_branch,
        git_fetch,
        git_pull,
        git_push,
        git_stash_save,
        git_stash_apply,
        git_stash_drop,
        git_clone,
        git_worktree_list,
        git_undo_last_commit,
        git_reset,
        git_revert,
        git_revert_abort,
        git_rebase,
        git_rebase_abort,
        git_rebase_continue,
        git_cherry_pick,
        git_cherry_pick_abort,
        git_cherry_pick_continue,
        git_tag_list,
        git_tag_create,
        git_tag_delete,
        git_merge,
        git_merge_abort,
        git_commit_no_edit,
        git_list_conflicts,
        git_resolve_conflict,
        git_cleanup_scan,
        git_cleanup_delete,
        git_compare,
        git_compare_file_diff,
        git_blame,
        git_create_pull_request,
        git_list_pull_requests,
        git_open_url,
        git_open_terminal,
        git_open_vscode,
        git_worktree_add,
        git_worktree_remove,
        git_watch_start,
        git_watch_stop,
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
        ai_usage_refresh_account,
        ai_usage_get_settings,
        ai_usage_save_settings,
        ai_usage_open_login,
        ai_usage_open_terminal,
        // === AI Chat commands ===
        ai_chat_complete,
        // === AI Workflow commands ===
        ai_workflow_create,
        ai_workflow_list,
        ai_workflow_update,
        ai_workflow_delete,
        ai_workflow_step_list,
        ai_workflow_step_create,
        ai_workflow_step_update,
        ai_workflow_step_delete,
        ai_workflow_step_reorder,
        ai_workflow_save_layout,
        ai_model_list,
        // === AI Task commands ===
        ai_task_create,
        ai_task_list,
        ai_task_update,
        // === AI Task WF Proc commands ===
        ai_task_wf_proc_create,
        ai_task_wf_proc_list,
        ai_task_wf_proc_update,
        // === AI Task WF Proc Step commands ===
        ai_task_wf_proc_step_create,
        ai_task_wf_proc_step_list,
        ai_task_wf_proc_step_update,
        // === AI Translate Cowork commands ===
        ai_translate_cowork_get_state,
        ai_translate_cowork_save_state,
        // === AI Cowork commands ===
        ai_cowork_get_state,
        ai_cowork_save_state,
        // === Terminal (PTY) commands ===
        terminal_spawn,
        terminal_write,
        terminal_resize,
        terminal_kill,
        // === Schedule commands ===
        read_schedule_excel,
        // === SQL Editor commands ===
        sql_list_connections,
        sql_save_connection,
        sql_delete_connection,
        sql_test_connection,
        sql_get_schema,
        sql_run_query,
        // === App Config commands ===
        get_app_config,
        save_app_config,
        // === Store Procedure management commands ===
        list_stored_procedures,
        get_stored_procedure_content,
        execute_single_stored_procedure,
        execute_stored_procedures,
        // === Pagination config command ===
        get_pagination_config
    ]
}
