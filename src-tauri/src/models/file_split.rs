//! Model cho công cụ nén file/folder thành ZIP (AES-256 tuỳ chọn) và cắt thành
//! các phần `.001/.002/...` để gửi kèm email.

use serde::{Deserialize, Serialize};

/// Một mục nguồn được người dùng chọn để nén (file hoặc folder).
///
/// Loại file/folder được xác định lại bằng filesystem (`is_dir()`) nên không
/// cần lưu `kind` từ frontend.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSplitSource {
    /// Đường dẫn đầy đủ tới file/folder.
    pub path: String,
    /// Tên hiển thị (chỉ tên, không có thư mục cha).
    pub name: String,
}

/// Thông số cấu hình cho một lần nén + tách.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSplitOptions {
    /// Danh sách nguồn cần nén.
    pub sources: Vec<FileSplitSource>,
    /// Thư mục xuất (nơi ghi file zip và các phần tách).
    pub output_dir: String,
    /// Tên cơ sở cho file zip (khi gộp 1 file). Có thể kèm hoặc không đuôi `.zip`.
    pub archive_name: String,
    /// Giới hạn dung lượng mỗi phần (MB). `None` = không tách.
    pub limit_mb: Option<f64>,
    /// Mật khẩu zip. Chuỗi rỗng = không mã hoá.
    pub password: String,
    /// `true` = gộp tất cả nguồn vào 1 file zip; `false` = mỗi nguồn 1 file zip.
    pub single_archive: bool,
}

/// Một file kết quả được ghi ra đĩa (file zip nguyên hoặc một phần `.001`).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSplitPart {
    /// Tên file (chỉ tên, không có thư mục).
    pub name: String,
    /// Dung lượng file (byte).
    pub size_bytes: u64,
}

/// Kết quả tổng hợp trả về frontend.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSplitResult {
    /// Thư mục chứa toàn bộ file kết quả.
    pub output_dir: String,
    /// Danh sách toàn bộ file đã tạo (zip nguyên và/hoặc các phần tách).
    pub files: Vec<FileSplitPart>,
    /// Tổng dung lượng của tất cả file kết quả (byte).
    pub total_bytes: u64,
    /// Số file zip đã tạo.
    pub archive_count: usize,
    /// Số file zip đã bị cắt thành nhiều phần.
    pub split_count: usize,
    /// Có mã hoá AES-256 hay không.
    pub encrypted: bool,
}
