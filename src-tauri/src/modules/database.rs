/// Tầng truy cập dữ liệu — đọc/ghi database và file.
#[path = "../database"]
mod database {
    /// Data access cho module xác thực người dùng.
    pub mod auth_store;
    /// Data access cho bảng `daily_work_notes` (PostgreSQL).
    pub mod daily_note_store;
    /// Data access cho bảng `daily_report_entries` và `daily_report_tasks`.
    pub mod daily_report_store;
    /// Data access cho bảng `projects` và `project_members` (PostgreSQL).
    pub mod project_store;
    /// Khởi tạo database (tạo bảng + stored procedure) khi app khởi động.
    pub mod startup_store;
    /// Data access cho module quản lý người dùng.
    pub mod user_store;
    /// Data access cho module quản lý role (governance).
    pub mod role_store;
    /// Data access cho bảng `menu_configs` (PostgreSQL).
    pub mod menu_config_store;
    /// Data access cho bảng `role_menu_permissions` và `user_menu_permissions`.
    pub mod menu_permission_store;
    /// Data access cho bảng `aws_storage` (PostgreSQL).
    pub mod aws_storage_store;
    /// Data access cho bảng `download_hdr` và `download_dtl` (lịch sử download S3).
    pub mod download_store;
    /// Data access cho bảng `upload_hdr`, `upload_dtl`, `upload_attach` (lịch sử upload S3).
    pub mod upload_store;
    /// Lưu trữ cục bộ (JSON file) danh sách account AI + cấu hình usage.
    pub mod ai_account_store;
    /// Lưu OAuth token đã capture của account subscription vào profile (app data dir).
    pub mod ai_profile_store;
    /// Lưu trữ cục bộ (JSON file) danh sách kết nối của SQL Editor.
    pub mod sql_connection_store;
    /// Data access cho bảng `ai_workflows` và `ai_workflow_steps` (PostgreSQL).
    pub mod ai_workflow_store;
    /// Data access cho bảng `ai_tasks` (PostgreSQL).
    pub mod ai_task_store;
    /// Lưu trữ cục bộ (JSON file) state màn hình AI Translate Cowork.
    pub mod ai_translate_cowork_store;
    /// Lưu trữ cục bộ (JSON file) state màn hình AI Cowork.
    pub mod ai_cowork_store;
    /// Lưu trữ cục bộ (JSON file) danh sách repository của màn hình Git Desktop.
    pub mod git_repo_store;
}
