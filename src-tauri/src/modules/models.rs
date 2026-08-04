/// Các kiểu dữ liệu (model/DTO) chia theo domain.
#[path = "../models"]
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
    /// Model cho công cụ nén ZIP (AES-256) và cắt file `.001`.
    pub mod file_split;
    /// Model cho công cụ so sánh khác biệt giữa 2 file.
    pub mod file_compare;
    /// Model kết quả resize ảnh evidence (hardcopy) trong Excel.
    pub mod excel_helper;
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
    /// Model cho module AI Workflow (workflow và steps).
    pub mod ai_workflow;
    /// Model cho module AI Task (task code + phân loại).
    pub mod ai_task;
    /// Model cho màn hình Git Desktop (repo, status, diff, commit, branch, stash).
    pub mod git;
}
