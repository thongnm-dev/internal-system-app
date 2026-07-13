//! Type alias cho `Result` dùng chung trong toàn bộ ứng dụng.

use super::error::AppError;

/// Kiểu Result thống nhất: `Ok(T)` hoặc `Err(AppError)`.
/// Sử dụng ở tất cả các tầng service, database, và utility.
pub type AppResult<T> = Result<T, AppError>;
