//! Service kiểm tra kết nối internet.

use crate::utils::network::is_internet_reachable;

/// Kiểm tra ứng dụng có thể kết nối internet hay không.
/// Delegate sang `utils::network::is_internet_reachable()`.
pub async fn check_connection() -> bool {
    is_internet_reachable().await
}
