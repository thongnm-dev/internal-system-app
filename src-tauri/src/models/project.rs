//! Các kiểu dữ liệu (model) cho module quản lý dự án.

use serde::{Deserialize, Serialize};

/// Thông tin một thành viên trong dự án.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectMember {
    /// Tên đăng nhập (username) của thành viên.
    pub username: String,
    /// Tên hiển thị đầy đủ.
    pub name: String,
}

/// Thông tin chi tiết của một dự án, bao gồm danh sách thành viên.
/// Dùng cho cả response trả về frontend và lưu/đọc từ database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectDetail {
    /// ID dự án (auto-increment từ PostgreSQL).
    pub id: i32,
    /// Mã dự án (unique, viết tắt, ví dụ: "YUJI", "HRP").
    pub code: String,
    /// Tên đầy đủ của dự án.
    pub name: String,
    /// Tên khách hàng hoặc bộ phận sở hữu.
    pub client: String,
    /// Mã project trên Backlog (nếu có liên kết).
    pub backlog_key: String,
    /// Mã số (ID) dự án trên Backlog.
    pub backlog_code: String,
    /// Tên dự án trên Backlog.
    pub backlog_name: String,
    /// Trạng thái hoạt động (true = đang hoạt động).
    pub is_active: bool,
    /// Danh sách thành viên tham gia dự án.
    pub members: Vec<ProjectMember>,
    /// Thời điểm tạo (ISO timestamp dạng text).
    pub created_at: String,
    /// Thời điểm cập nhật gần nhất.
    pub updated_at: String,
}

/// Thông tin tóm tắt của dự án, dùng cho danh sách (list view).
/// Không chứa chi tiết thành viên mà chỉ đếm số lượng.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub client: String,
    pub is_active: bool,
    /// Số lượng thành viên trong dự án.
    pub member_count: i64,
    pub created_at: String,
    /// Backlog key của dự án; `None`/rỗng nghĩa là chưa thiết lập Backlog.
    pub backlog_key: Option<String>,
}

/// Dữ liệu request từ frontend khi tạo mới hoặc cập nhật dự án.
/// Các trường optional sẽ được gán giá trị mặc định nếu không truyền.
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub code: String,
    pub name: String,
    pub client: Option<String>,
    pub backlog_key: Option<String>,
    pub backlog_code: Option<String>,
    pub backlog_name: Option<String>,
    pub members: Vec<ProjectMember>,
}

/// Thông tin chi tiết của một task trong dự án.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectTask {
    pub id: String,
    pub project_id: i32,
    pub short_name: String,
    pub description: String,
    pub categories: Vec<String>,
    pub assignee: String,
    pub estimate_hour: String,
    pub due_date: String,
    pub issue_key: String,
    pub is_user_added: bool,
    pub created_at: String,
}

/// Dữ liệu request từ frontend khi tạo task mới cho dự án.
#[derive(Debug, Deserialize)]
pub struct CreateProjectTaskRequest {
    pub short_name: String,
    pub description: String,
    pub categories: Vec<String>,
    pub assignee: String,
    pub estimate_hour: String,
    pub due_date: String,
    pub issue_key: String,
}
