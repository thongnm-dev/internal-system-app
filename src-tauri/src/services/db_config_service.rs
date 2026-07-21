//! Business logic cho cấu hình kết nối database.
//!
//! Kiểm tra trạng thái cấu hình khi khởi động, đọc và lưu `config.ini`,
//! đồng thời khởi tạo bảng + stored procedure sau khi cấu hình hợp lệ.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::db_config::{DatabaseConfig, DatabaseStatus, SaveDatabaseConfigRequest};
use crate::utils::pgsql_connect::{self, PgConfig};

/// Kiểm tra trạng thái cấu hình database.
///
/// - Nếu `config.ini` chưa có cấu hình hợp lệ → `configured = false`.
/// - Nếu đã cấu hình nhưng không kết nối được → `connected = false` kèm lỗi.
/// - Nếu kết nối được → `configured = true, connected = true`.
pub async fn check_status() -> DatabaseStatus {
    let config = match PgConfig::from_ini() {
        Ok(config) => config,
        Err(_) => {
            return DatabaseStatus {
                configured: false,
                connected: false,
                message: "Database chưa được cấu hình.".to_string(),
            };
        }
    };

    match pgsql_connect::test_connection(&config).await {
        Ok(_) => DatabaseStatus {
            configured: true,
            connected: true,
            message: String::new(),
        },
        Err(e) => DatabaseStatus {
            configured: true,
            connected: false,
            message: e.to_string(),
        },
    }
}

/// Đọc cấu hình database hiện tại (dùng để prefill form cấu hình).
/// Trả về `None` nếu chưa cấu hình.
pub fn get_config() -> Option<DatabaseConfig> {
    PgConfig::from_ini().ok().map(|config| DatabaseConfig {
        host: config.host,
        port: config.port,
        dbname: config.dbname,
        user: config.user,
        password: config.password,
    })
}

/// Validate các trường bắt buộc và dựng `PgConfig` từ request.
fn build_config(request: &SaveDatabaseConfigRequest) -> AppResult<PgConfig> {
    let host = request.host.trim();
    if host.is_empty() {
        return Err(AppError::new("Host là bắt buộc."));
    }
    let dbname = request.dbname.trim();
    if dbname.is_empty() {
        return Err(AppError::new("Tên database (dbname) là bắt buộc."));
    }
    if request.port == 0 {
        return Err(AppError::new("Port không hợp lệ."));
    }

    Ok(PgConfig {
        host: host.to_string(),
        port: request.port,
        dbname: dbname.to_string(),
        user: request.user.trim().to_string(),
        password: request.password.clone(),
    })
}

/// Chỉ thử kết nối với cấu hình cho trước, không ghi file.
/// Dùng cho nút "Kiểm tra" trên màn hình cấu hình.
pub async fn test_config(request: SaveDatabaseConfigRequest) -> AppResult<()> {
    let config = build_config(&request)?;
    pgsql_connect::test_connection(&config).await
}

/// Lưu cấu hình database.
///
/// Trình tự:
/// 1. Validate các trường bắt buộc (`host`, `dbname`).
/// 2. Thử kết nối với cấu hình mới — nếu thất bại thì trả lỗi, không ghi file.
/// 3. Ghi `config.ini`.
/// 4. Khởi tạo bảng + stored procedure.
pub async fn save_config(request: SaveDatabaseConfigRequest) -> AppResult<DatabaseStatus> {
    let config = build_config(&request)?;

    // Xác nhận kết nối được trước khi ghi cấu hình.
    pgsql_connect::test_connection(&config).await?;

    // Ghi cấu hình xuống config.ini.
    config.save_to_ini()?;

    Ok(DatabaseStatus {
        configured: true,
        connected: true,
        message: String::new(),
    })
}
