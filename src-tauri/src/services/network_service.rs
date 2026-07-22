//! Service kiểm tra kết nối mạng (internet + nội bộ).

use crate::utils::network::{is_internet_reachable, is_internal_reachable};

/// Kiểm tra ứng dụng có thể kết nối internet hay không.
pub async fn check_connection() -> bool {
    is_internet_reachable().await
}

/// Kiểm tra máy có kết nối mạng nội bộ công ty (hoặc VPN) hay không.
/// Trả `true` nếu chưa cấu hình IP hoặc ít nhất một IP phản hồi.
pub async fn check_internal() -> bool {
    is_internal_reachable().await
}
