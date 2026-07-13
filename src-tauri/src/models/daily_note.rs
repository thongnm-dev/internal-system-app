//! Các kiểu dữ liệu (model) cho module ghi chú công việc hằng ngày.

use serde::{Deserialize, Serialize};

/// Một bản ghi ghi chú công việc hằng ngày.
/// Mỗi note thuộc về một user (theo username) và gắn với một ngày cụ thể.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyWorkNote {
    /// ID ghi chú (auto-increment từ PostgreSQL).
    pub id: i32,
    /// Tên đăng nhập của người tạo ghi chú.
    pub username: String,
    /// Nội dung công việc.
    pub content: String,
    /// Ngày công việc (định dạng YYYY-MM-DD).
    pub note_date: String,
    /// Trạng thái: "completed", "incomplete", hoặc "reserved".
    pub status: String,
    /// Thời điểm tạo ghi chú (ISO timestamp dạng text).
    pub created_at: String,
}

/// Dữ liệu request từ frontend khi tạo ghi chú mới.
#[derive(Debug, Deserialize)]
pub struct CreateDailyNoteRequest {
    /// Nội dung công việc (bắt buộc, không được rỗng).
    pub content: String,
    /// Ngày công việc (định dạng YYYY-MM-DD, bắt buộc).
    pub note_date: String,
    /// Trạng thái ban đầu: "completed" | "incomplete" | "reserved".
    pub status: String,
}

/// Số lượng ghi chú theo ngày, dùng để hiển thị badge trên lịch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyNoteDateCount {
    /// Ngày (định dạng YYYY-MM-DD).
    pub note_date: String,
    /// Tổng số ghi chú trong ngày đó.
    pub note_count: i64,
}
