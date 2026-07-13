//! Hằng số và hàm chuyển đổi dùng chung trong ứng dụng.

/// Chuyển mã phase (process code) thành tên hiển thị tiếng Việt/viết tắt.
///
/// Nếu mã không nằm trong danh sách đã biết, sử dụng `fallback` làm tên hiển thị.
/// Nếu `fallback` cũng rỗng, trả về `"Other (mã)"`.
pub fn phase_label(code: &str, fallback: &str) -> String {
    match code.trim() {
        "10" => "PG".to_string(),
        "11" => "UT".to_string(),
        "48" => "Review PG".to_string(),
        "49" => "Review UT".to_string(),
        "43" => "Bug".to_string(),
        "45" => "Thay doi qui cach".to_string(),
        "59" => "Trong tay".to_string(),
        "24" => "Delivery".to_string(),
        other if fallback.trim().is_empty() => format!("Other ({other})"),
        _ => fallback.to_string(),
    }
}
