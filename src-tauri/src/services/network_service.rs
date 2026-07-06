use crate::utils::network::is_internet_reachable;

/// Checks whether the application can currently reach the internet.
pub async fn check_connection() -> bool {
    is_internet_reachable().await
}
