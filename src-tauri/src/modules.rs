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
    /// Commands cho module ghi chú công việc hằng ngày.
    pub mod daily_note_commands;
    /// Commands cho màn hình daily report (giờ công + task người dùng thêm).
    pub mod daily_report_commands;
    /// Commands cho cấu hình kết nối database.
    pub mod db_config_commands;
    /// Command chuyển đổi Excel → Markdown.
    pub mod excel2md_commands;
    /// Commands cho import CSV báo cáo tháng.
    pub mod import_commands;
    /// Commands cho module quản lý dự án.
    pub mod project_commands;
    /// Commands lấy thông tin hệ thống và kiểm tra mạng.
    pub mod system_commands;
}

/// Tầng truy cập dữ liệu — đọc/ghi database và file.
mod database {
    /// Đọc file CSV công việc (Shift-JIS) và parse thành `WorkRecord`.
    pub mod csv_reader;
    /// Data access cho bảng `daily_work_notes` (PostgreSQL).
    pub mod daily_note_store;
    /// Data access cho bảng `daily_report_entries` và `daily_report_tasks`.
    pub mod daily_report_store;
    /// Data access cho bảng `projects` và `project_members` (PostgreSQL).
    pub mod project_store;
    /// Lưu trữ lịch sử import CSV dưới dạng JSON file.
    pub mod report_store;
}

/// Các kiểu dữ liệu (model/DTO) chia theo domain.
mod models {
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
    /// Model thông tin hệ thống (username, IP, version).
    pub mod system;
}

/// Tầng business logic — xử lý nghiệp vụ, validation, điều phối.
mod services {
    /// Service cho module ghi chú công việc hằng ngày.
    pub mod daily_note_service;
    /// Service cho màn hình daily report (giờ công + task người dùng thêm).
    pub mod daily_report_service;
    /// Service cho cấu hình kết nối database.
    pub mod db_config_service;
    /// Service chuyển đổi Excel → Markdown (gọi script Python).
    pub mod excel2md_service;
    /// Service preview và đọc dữ liệu import CSV.
    pub mod import_csv_service;
    /// Service lưu trữ và truy vấn lịch sử import báo cáo tháng.
    pub mod monthly_report_service;
    /// Service kiểm tra kết nối internet.
    pub mod network_service;
    /// Service cho module quản lý dự án.
    pub mod project_service;
    /// Service lấy thông tin hệ thống.
    pub mod system_service;
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
}
