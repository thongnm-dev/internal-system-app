/// Tầng business logic — xử lý nghiệp vụ, validation, điều phối.
#[path = "../services"]
mod services {
    /// Service cho module xác thực người dùng.
    pub mod auth_service;
    /// Service cho module ghi chú công việc hằng ngày.
    pub mod daily_note_service;
    /// Service cho màn hình daily report (giờ công + task người dùng thêm).
    pub mod daily_report_service;
    /// Service cho cấu hình kết nối database.
    pub mod db_config_service;
    /// Service chuyển đổi Excel → Markdown (gọi script Python).
    pub mod excel2md_service;
    /// Service nén ZIP (AES-256) và cắt file thành các phần `.001`.
    pub mod file_split_service;
    /// Service so sánh khác biệt giữa 2 file (text/markdown/word/excel).
    pub mod file_compare_service;
    /// Service resize ảnh evidence (hardcopy) trong Excel (XML splicing trực tiếp).
    pub mod excel_helper_service;
    /// Đọc file CSV công việc (Shift-JIS) và parse thành `WorkRecord`.
    pub mod csv_reader_service;
    /// Service preview và so sánh dữ liệu CSV báo cáo tháng.
    pub mod monthly_report_service;
    /// Service kiểm tra kết nối internet.
    pub mod network_service;
    /// Service cho module quản lý dự án.
    pub mod project_service;
    /// Service cho module cài đặt ứng dụng.
    pub mod settings_service;
    /// Service lấy thông tin hệ thống.
    pub mod system_service;
    /// Service cho module quản lý người dùng.
    pub mod user_service;
    /// Service cho module quản lý role (governance).
    pub mod role_service;
    /// Service cho module quản lý menu (governance).
    pub mod menu_config_service;
    /// Service cho module phân quyền menu theo user/role (governance).
    pub mod menu_permission_service;
    /// Service cho Backlog API integration.
    pub mod backlog_service;
    /// Service cho S3 browser operations.
    pub mod s3_service;
    /// Theo dõi nền storage S3 và bắn notification khi có tài liệu mới.
    pub mod s3_watch_service;
    /// Service đồng bộ dữ liệu lên hệ thống nội bộ (Selenium automation).
    pub mod sync_service;
    /// Service gom/copy file (collect input) theo keyword hoặc danh sách.
    pub mod collect_service;
    /// Service copy file theo danh sách folder.
    pub mod collect_folders_service;
    /// Service cho file explorer nhanh (đọc thư mục, tìm kiếm file).
    pub mod explorer_service;
    /// Service cho module AI Usage — quản lý account AI, priority, auto-switch.
    pub mod ai_usage_service;
    /// API công khai CRUD account + settings AI Usage (dùng chung mọi provider).
    pub mod ai_acc_service;
    /// Nghiệp vụ AI Usage riêng cho provider Claude (dò/import/capture login local).
    /// Các provider khác (vd Codex) sẽ có service riêng tương tự trong tương lai.
    pub mod claude_service;
    /// Probe tình trạng usage cho từng account AI (rate-limit header, v.v.).
    pub mod ai_usage_probe;
    /// Probe usage riêng cho provider Claude (subscription OAuth usage + anthropic ratelimit).
    pub mod claude_probe;
    /// Probe usage riêng cho provider Codex/OpenAI (x-ratelimit header).
    pub mod codex_probe;
    /// Dò các login Claude đã tồn tại trên máy (đọc `.claude.json` + Keychain).
    pub mod claude_detected;
    /// Đọc credential store cho Windows + Linux (file `.credentials.json`, không có Keychain).
    #[cfg(not(target_os = "macos"))]
    pub mod claude_credentials_windows;
    /// Đọc credential store cho macOS (Keychain, lệnh `security`).
    #[cfg(target_os = "macos")]
    pub mod claude_credentials_macos;
    /// Mở terminal desktop chạy `claude` — tách biệt khỏi nghiệp vụ quản lý account AI Usage.
    pub mod claude_terminal;
    /// Mở terminal Windows (`cmd /k` qua file `.bat` tạm).
    #[cfg(target_os = "windows")]
    pub mod claude_terminal_windows;
    /// Mở terminal macOS (Terminal.app qua file `.command` tạm).
    #[cfg(target_os = "macos")]
    pub mod claude_terminal_macos;
    /// Capture login Claude đang active → lưu token vào profile (app data dir).
    pub mod claude_capture;
    /// Service cho module AI Chat — gọi API các nhà cung cấp LLM.
    pub mod ai_chat_service;
    /// Service đọc file schedule Excel và trích xuất dữ liệu giờ công.
    pub mod schedule_service;
    /// Service cho màn hình SQL Editor (quản lý kết nối + chạy query).
    pub mod sql_editor_service;
    /// Service parse CSV issue (dùng CsvReader có sẵn).
    pub mod issue_csv_service;
    /// Service đọc/ghi toàn bộ config.ini.
    pub mod app_config_service;
    /// Service quản lý Store Procedure — liệt kê và thực thi CREATE OR REPLACE.
    pub mod sp_management_service;
    /// Service cho module AI Workflow — quản lý workflow và steps.
    pub mod ai_workflow_service;
    /// Service cho module AI Task — tìm kiếm/thêm task.
    pub mod ai_task_service;
    /// Service cho state màn hình AI Translate Cowork (lịch sử làm việc gần nhất).
    pub mod ai_translate_cowork_service;
    /// Service cho state màn hình AI Cowork (lịch sử làm việc gần nhất).
    pub mod ai_cowork_service;
    /// Service đọc cấu hình phân trang DataTable từ config.ini.
    pub mod pagination_service;
    /// Service quản lý phiên terminal nhúng (PTY): spawn/write/resize/kill.
    pub mod terminal_service;
    /// Service cho màn hình Git Desktop — gọi `git` CLI cho mọi thao tác.
    pub mod git_service;
    /// Theo dõi thay đổi file trên đĩa của repo Git đang mở (auto-refresh tab Changes).
    pub mod git_watch_service;
}
