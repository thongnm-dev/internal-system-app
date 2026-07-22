// Đăng ký tất cả các module trong ứng dụng.
//
// File này được `include!()` từ `lib.rs` để khai báo cấu trúc module tree.
// Tổ chức theo kiến trúc phân tầng: `commands → services → database → utils`.

/// Tầng command: Tauri IPC handlers, nhận request từ frontend.
mod app {
    /// Hằng số dùng chung (mapping mã phase → tên hiển thị).
    pub mod consts;
    /// Kiểu lỗi thống nhất `AppError`.
    pub mod error;
    /// Type alias `AppResult<T>` cho `Result<T, AppError>`.
    pub mod result;
}

/// Tauri command handlers — điểm vào từ frontend qua IPC invoke.
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
}

/// Tầng truy cập dữ liệu — đọc/ghi database và file.
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
}

/// Các kiểu dữ liệu (model/DTO) chia theo domain.
mod models {
    /// Model cho module xác thực người dùng.
    pub mod auth;
    /// Model cho module ghi chú công việc hằng ngày.
    pub mod daily_note;
    /// Model cho màn hình daily report (giờ công + task người dùng thêm).
    pub mod daily_report;
    /// Model cho cấu hình kết nối database.
    pub mod db_config;
    /// Model kết quả chuyển đổi Excel → Markdown.
    pub mod excel2md;
    /// Model cho import CSV (preview row, minute totals, work record).
    pub mod import_csv;
    /// Model cho lịch sử import báo cáo tháng.
    pub mod monthly_report;
    /// Model cho module quản lý dự án.
    pub mod project;
    /// Model cho module cài đặt ứng dụng.
    pub mod settings;
    /// Model thông tin hệ thống (username, IP, version).
    pub mod system;
    /// Model cho module quản lý người dùng.
    pub mod user;
    /// Model cho module quản lý role (governance).
    pub mod role;
    /// Model cho module quản lý menu (governance).
    pub mod menu_config;
    /// Model cho module phân quyền menu theo user/role (governance).
    pub mod menu_permission;
    /// Model cho Backlog API responses.
    pub mod backlog;
    /// Model cho S3 browser (config, object, operation result).
    pub mod s3;
    /// Model cho đồng bộ dữ liệu (Selenium daily report sync).
    pub mod sync;
    /// Model cho công cụ collect/copy file.
    pub mod collect;
    /// Model cho file explorer nhanh.
    pub mod explorer;
    /// Model cho module AI Usage (quản lý account AI).
    pub mod ai_usage;
    /// Model cho module AI Chat (hội thoại với các nhà cung cấp LLM).
    pub mod ai_chat;
    /// Model cho đọc file schedule Excel.
    pub mod schedule;
    /// Model cho màn hình SQL Editor (kết nối + kết quả query).
    pub mod sql_editor;
    /// Model cho import CSV issue.
    pub mod issue_csv;
    /// Model cho cấu hình ứng dụng (config.ini) và quản lý Store Procedure.
    pub mod app_config;
}

/// Tầng business logic — xử lý nghiệp vụ, validation, điều phối.
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
    /// Probe tình trạng usage cho từng account AI (rate-limit header, v.v.).
    pub mod ai_usage_probe;
    /// Dò các login Claude đã tồn tại trên máy (đọc `.claude.json` + Keychain).
    pub mod ai_usage_detect;
    /// Capture login Claude đang active → lưu token vào profile (app data dir).
    pub mod ai_usage_capture;
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
}

/// Tiện ích hạ tầng dùng chung (network, time, encoding, database).
mod utils {
    /// HTTP client wrapper cho gọi API bên ngoài (chưa sử dụng).
    #[allow(dead_code)]
    pub mod api_client;
    /// Đọc file CSV với hỗ trợ encoding Shift-JIS.
    pub mod csv_reader;
    /// Kiểm tra kết nối internet và lấy IP local.
    pub mod network;
    /// Kết nối PostgreSQL, tạo bảng và stored procedure.
    pub mod pgsql_connect;
    /// Hàm tiện ích lấy timestamp hiện tại.
    pub mod time;
    /// Đường dẫn dữ liệu và cấu hình ứng dụng (AppData + config.ini).
    pub mod app_config;
    /// Gửi email qua SMTP (dùng cho reset password, v.v.).
    pub mod email;
}
