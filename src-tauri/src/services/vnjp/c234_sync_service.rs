//! Method xử lý riêng cho C2.3.4 画面仕様書（編集要領）.
//!
//! Loại tài liệu này dùng chung MỌI xử lý (đọc file, quét ô đỏ/strikethrough, dọn dẹp, ghi ô đỏ,
//! canh dòng, xuất báo cáo...) với các loại khác — xem `super::sync_service`. Khác biệt DUY NHẤT
//! là vùng cột nội dung, khai báo ở đây.

/// Nội dung cột A ~ M (0-based 12).
pub fn content_bounds(sheet_name: &str) -> super::sync_service::ContentBounds {
    super::sync_service::content_bounds_for_sheet(sheet_name, 12)
}
