//! Các kiểu dữ liệu (model) cho màn hình báo cáo công việc hằng ngày (daily report).
//!
//! Gồm hai domain:
//! - `DailyReportEntry`: số giờ công nhập theo từng task / từng ngày.
//! - `DailyReportUserTask`: task do người dùng tự thêm vào một project.

use serde::{Deserialize, Serialize};

/// Một ô nhập giờ công của một task trong một ngày cụ thể.
/// Khóa duy nhất theo (username, task_id, entry_date).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyReportEntry {
    /// ID bản ghi (auto-increment từ PostgreSQL).
    pub id: i32,
    /// Tên đăng nhập của người nhập.
    pub username: String,
    /// ID task ở frontend (ví dụ: "yuji-planning" hoặc "pj-yuji-task-123").
    pub task_id: String,
    /// ID project chứa task (dùng để nhóm khi hiển thị).
    pub project_id: String,
    /// Ngày công việc (định dạng YYYY-MM-DD).
    pub entry_date: String,
    /// Ghi chú cho ô nhập.
    pub comment: String,
    /// Số giờ công.
    pub hour: f64,
    /// Có phải giờ OT hay không.
    pub is_ot: bool,
    /// Số giờ OT thường (chỉ có ý nghĩa khi `is_ot = true`).
    pub regular_ot: f64,
    /// Số giờ OT khuya (chỉ có ý nghĩa khi `is_ot = true`).
    pub midnight_ot: f64,
    /// Phase / công đoạn liên quan.
    pub phase: String,
    /// Thời điểm cập nhật gần nhất (ISO timestamp dạng text).
    pub updated_at: String,
}

/// Dữ liệu request từ frontend khi lưu (upsert) một ô nhập giờ công.
#[derive(Debug, Deserialize)]
pub struct SaveDailyReportEntryRequest {
    /// ID task ở frontend (bắt buộc).
    pub task_id: String,
    /// ID project chứa task (bắt buộc).
    pub project_id: String,
    /// Ngày công việc (định dạng YYYY-MM-DD, bắt buộc).
    pub entry_date: String,
    /// Ghi chú.
    #[serde(default)]
    pub comment: String,
    /// Số giờ công (bắt buộc, > 0).
    pub hour: f64,
    /// Có phải giờ OT hay không.
    #[serde(default)]
    pub is_ot: bool,
    /// Số giờ OT thường.
    #[serde(default)]
    pub regular_ot: f64,
    /// Số giờ OT khuya.
    #[serde(default)]
    pub midnight_ot: f64,
    /// Phase / công đoạn.
    #[serde(default)]
    pub phase: String,
}

/// Một task do người dùng tự thêm vào project trên màn hình daily report.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyReportUserTask {
    /// ID bản ghi (auto-increment từ PostgreSQL).
    pub id: i32,
    /// Tên đăng nhập của người tạo task.
    pub username: String,
    /// ID task ở frontend (unique theo user).
    pub task_id: String,
    /// ID project chứa task.
    pub project_id: String,
    /// Mã task viết tắt (thường là category đầu tiên hoặc "TASK").
    pub code: String,
    /// Tên (short name) của task.
    pub name: String,
    /// Mô tả chi tiết.
    pub description: String,
    /// Danh sách phân loại (PG, Review PG, UT, ...).
    pub categories: Vec<String>,
    /// Username người được giao.
    pub assignee: String,
    /// Số giờ ước lượng (giữ dạng chuỗi để khớp form frontend).
    pub estimate_hour: String,
    /// Ngày hết hạn (YYYY-MM-DD hoặc rỗng).
    pub due_date: String,
    /// Issue key liên kết với backlog.
    pub issue_key: String,
    /// Đã hoàn thành (delivery) hay chưa. Task chưa xong sẽ mang sang tháng sau.
    pub is_completed: bool,
    /// Thời điểm đánh dấu delivery (ISO timestamp dạng text), rỗng nếu chưa.
    pub completed_at: String,
    /// Thời điểm tạo (ISO timestamp dạng text).
    pub created_at: String,
    /// Nguồn task: TRUE = user tự thêm (daily_report_tasks),
    /// FALSE = task dự án được giao (project_tasks). Quyết định lệnh delivery/xoá.
    pub is_user_added: bool,
}

/// Một công đoạn (process/phase) cho dropdown khi nhập giờ ở Daily Report.
/// `process_code` chứa luôn tên phase (PG, UT, Review PG…).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyReportPhase {
    pub process_code: String,
    pub display_order: i32,
}

/// Tổng số giờ tích luỹ (mọi tháng) của một task, dùng cho badge tiến độ.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyReportTaskHours {
    /// ID task ở frontend.
    pub task_id: String,
    /// Tổng số giờ (cột hour) đã nhập của mọi tháng.
    pub total_hour: f64,
}

/// Thông tin project cho màn hình daily report.
/// Bao gồm cờ `is_member` cho biết user có phải thành viên của project không.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyReportProject {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub client: String,
    pub is_member: bool,
}

/// Dữ liệu request từ frontend khi tạo task người dùng tự thêm.
#[derive(Debug, Deserialize)]
pub struct CreateDailyReportTaskRequest {
    /// ID task ở frontend (bắt buộc).
    pub task_id: String,
    /// ID project chứa task (bắt buộc).
    pub project_id: String,
    /// Mã task viết tắt.
    #[serde(default)]
    pub code: String,
    /// Tên (short name) của task (bắt buộc).
    pub name: String,
    /// Mô tả chi tiết.
    #[serde(default)]
    pub description: String,
    /// Danh sách phân loại.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Username người được giao.
    #[serde(default)]
    pub assignee: String,
    /// Số giờ ước lượng (chuỗi).
    #[serde(default)]
    pub estimate_hour: String,
    /// Ngày hết hạn.
    #[serde(default)]
    pub due_date: String,
    /// Issue key liên kết backlog.
    #[serde(default)]
    pub issue_key: String,
}
