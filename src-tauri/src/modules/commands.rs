/// Tauri command handlers — điểm vào từ frontend qua IPC invoke.
#[path = "../commands"]
mod commands {
    /// Commands cho module xác thực người dùng.
    pub mod auth_commands;
    /// Commands cho module ghi chú công việc hằng ngày.
    pub mod daily_note_commands;
    /// Commands cho màn hình daily report (giờ công + task người dùng thêm).
    pub mod daily_report_commands;
    /// Commands cho cấu hình kết nối database.
    pub mod db_config_commands;
    /// Command chuyển đổi Excel → Markdown.
    pub mod excel2md_commands;
    /// Command nén ZIP (AES-256) và cắt file thành các phần `.001`.
    pub mod file_split_commands;
    /// Commands cho công cụ so sánh khác biệt giữa 2 file.
    pub mod file_compare_commands;
    /// Commands cho công cụ đồng bộ tài liệu thiết kế chi tiết VN → JP.
    pub mod vnjp_sync_commands;
    /// Commands cho resize ảnh evidence (hardcopy) trong Excel.
    pub mod excel_helper_commands;
    /// Commands cho import CSV báo cáo tháng.
    pub mod monthly_report_commands;
    /// Commands cho module quản lý dự án.
    pub mod project_commands;
    /// Commands cho module cài đặt ứng dụng.
    pub mod settings_commands;
    /// Commands lấy thông tin hệ thống và kiểm tra mạng.
    pub mod system_commands;
    /// Commands cho module quản lý người dùng.
    pub mod user_commands;
    /// Commands cho module quản lý role (governance).
    pub mod role_commands;
    /// Commands cho module quản lý menu (governance).
    pub mod menu_config_commands;
    /// Commands cho module phân quyền menu theo user/role (governance).
    pub mod menu_permission_commands;
    /// Commands cho Backlog API integration.
    pub mod backlog_commands;
    /// Commands cho S3 browser (list, download, upload, delete).
    pub mod s3_commands;
    /// Commands cho đồng bộ dữ liệu lên hệ thống nội bộ (Selenium).
    pub mod sync_commands;
    /// Commands cho công cụ collect/copy file (gom tài liệu nguồn).
    pub mod collect_commands;
    /// Commands cho file explorer nhanh.
    pub mod explorer_commands;
    /// Commands cho module AI Usage (quản lý account AI).
    pub mod ai_usage_commands;
    /// Commands cho module AI Chat (gọi API các nhà cung cấp LLM).
    pub mod ai_chat_commands;
    /// Commands cho đọc file schedule Excel.
    pub mod schedule_commands;
    /// Commands cho màn hình SQL Editor (quản lý kết nối + chạy query).
    pub mod sql_editor_commands;
    /// Commands cho import CSV issue (parse file CSV issue từ frontend).
    pub mod issue_csv_commands;
    /// Commands cho cấu hình ứng dụng (config.ini) và quản lý Store Procedure.
    pub mod app_config_commands;
    /// Commands cho module AI Workflow (quản lý workflow và steps).
    pub mod ai_workflow_commands;
    /// Commands cho module AI Task (tìm kiếm/thêm task cho AI Cowork).
    pub mod ai_task_commands;
    /// Commands cho state màn hình AI Translate Cowork (lịch sử làm việc gần nhất).
    pub mod ai_translate_cowork_commands;
    /// Commands cho state màn hình AI Cowork (lịch sử làm việc gần nhất).
    pub mod ai_cowork_commands;
    /// Commands cho module Terminal nhúng (PTY): spawn/write/resize/kill.
    pub mod terminal_commands;
    /// Commands cho màn hình Git Desktop (thao tác git + quản lý danh sách repo).
    pub mod git_commands;
    /// Commands cho cấu hình phân trang DataTable (đọc từ config.ini).
    pub mod pagination_commands;
}
