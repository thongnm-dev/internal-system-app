//! Tiện ích kết nối PostgreSQL dùng chung cho toàn bộ ứng dụng.
//!
//! Đọc thông tin kết nối từ file `config.ini` (section `[database]`),
//! tạo bảng và stored procedure khi khởi động ứng dụng.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use ini::Ini;
use std::path::PathBuf;
use tokio_postgres::{Client, NoTls};

impl PgConfig {
    /// Ghi cấu hình xuống `config.ini` (section `[database]`).
    /// Đọc file hiện tại trước để giữ nguyên các section khác, chỉ cập nhật `[database]`.
    pub fn save_to_ini(&self) -> AppResult<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::new(format!("Failed to create config directory {}: {e}", parent.display()))
            })?;
        }

        let mut ini = if path.exists() {
            Ini::load_from_file(&path).unwrap_or_else(|_| Ini::new())
        } else {
            Ini::new()
        };

        ini.with_section(Some("database"))
            .set("host", &self.host)
            .set("port", self.port.to_string())
            .set("dbname", &self.dbname)
            .set("user", &self.user)
            .set("password", &self.password);

        ini.write_to_file(&path).map_err(|e| {
            AppError::new(format!("Failed to write config.ini at {}: {e}", path.display()))
        })?;
        Ok(())
    }
}

/// Thời gian chờ tối đa cho một lần thử kết nối (giây).
const CONNECT_TIMEOUT_SECS: u64 = 5;

/// Thử kết nối tới database với một cấu hình cho trước.
///
/// Mở kết nối tạm, chạy `SELECT 1` để xác nhận database phản hồi, rồi trả về.
/// Có timeout để không treo UI khi host/port sai (không phản hồi).
/// Dùng để kiểm tra cấu hình trước khi lưu.
pub async fn test_connection(config: &PgConfig) -> AppResult<()> {
    let timeout = std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS);

    let attempt = async {
        let (client, connection) = tokio_postgres::connect(&config.connection_string(), NoTls)
            .await
            .map_err(|e| AppError::new(format!("Cannot connect to database: {e}")))?;

        tokio::spawn(async move {
            let _ = connection.await;
        });

        client
            .query_one("SELECT 1", &[])
            .await
            .map_err(|e| AppError::new(format!("Database did not respond: {e}")))?;

        Ok::<(), AppError>(())
    };

    tokio::time::timeout(timeout, attempt).await.map_err(|_| {
        AppError::new(format!(
            "Kết nối database quá thời gian chờ ({CONNECT_TIMEOUT_SECS}s). Vui lòng kiểm tra host/port."
        ))
    })?
}

/// Cấu hình kết nối PostgreSQL, được đọc từ file `config.ini`.
#[derive(Debug, Clone)]
pub struct PgConfig {
    /// Địa chỉ máy chủ PostgreSQL (mặc định: localhost).
    pub host: String,
    /// Cổng kết nối (mặc định: 5432).
    pub port: u16,
    /// Tên cơ sở dữ liệu (bắt buộc).
    pub dbname: String,
    /// Tên người dùng PostgreSQL (mặc định: postgres).
    pub user: String,
    /// Mật khẩu kết nối.
    pub password: String,
}

impl PgConfig {
    /// Đọc cấu hình từ file `config.ini`, section `[database]`.
    ///
    /// Trả về lỗi nếu file không tồn tại, thiếu section, hoặc thiếu `dbname`.
    pub fn from_ini() -> AppResult<Self> {
        let path = config_path();
        let ini = Ini::load_from_file(&path).map_err(|e| {
            AppError::new(format!(
                "Failed to load config.ini at {}: {e}",
                path.display()
            ))
        })?;

        let section = ini
            .section(Some("database"))
            .ok_or_else(|| AppError::new("[database] section not found in config.ini"))?;

        let host = section.get("host").unwrap_or("localhost").to_string();
        let port = section
            .get("port")
            .unwrap_or("5432")
            .parse::<u16>()
            .map_err(|_| AppError::new("Invalid port in config.ini"))?;
        let dbname = section
            .get("dbname")
            .ok_or_else(|| AppError::new("dbname is required in config.ini"))?
            .to_string();
        let user = section.get("user").unwrap_or("postgres").to_string();
        let password = section.get("password").unwrap_or("").to_string();

        Ok(Self {
            host,
            port,
            dbname,
            user,
            password,
        })
    }

    /// Tạo chuỗi kết nối libpq dạng key=value.
    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.dbname, self.user, self.password
        )
    }
}

/// Tạo kết nối mới tới PostgreSQL.
///
/// Mỗi lần gọi sẽ mở một kết nối riêng biệt. Task xử lý connection
/// được spawn chạy nền để duy trì kết nối cho đến khi `Client` bị drop.
pub async fn connect() -> AppResult<Client> {
    let config = PgConfig::from_ini()?;
    let conn_str = config.connection_string();

    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
        .await
        .map_err(|e| AppError::new(format!("PostgreSQL connection failed: {e}")))?;

    // Spawn task chạy nền để xử lý I/O cho kết nối
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {e}");
        }
    });

    Ok(client)
}

/// Tạo kết nối mới tới PostgreSQL với một cấu hình cho trước (không đọc `config.ini`).
///
/// Dùng cho SQL Editor — mỗi kết nối do người dùng cấu hình được mở riêng.
pub async fn connect_with(config: &PgConfig) -> AppResult<Client> {
    let (client, connection) = tokio_postgres::connect(&config.connection_string(), NoTls)
        .await
        .map_err(|e| AppError::new(format!("PostgreSQL connection failed: {e}")))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {e}");
        }
    });

    Ok(client)
}

/// Trích xuất thông báo lỗi chi tiết từ một `tokio_postgres::Error`.
///
/// `Display` của `tokio_postgres::Error` chỉ in ra thông báo chung chung
/// (ví dụ `"db error"`); toàn bộ chi tiết do PostgreSQL trả về (mã lỗi,
/// message, detail, hint) nằm trong `as_db_error()`. Hàm này gộp chúng lại
/// để lỗi hiển thị cho người dùng có ý nghĩa.
pub fn pg_error_detail(e: &tokio_postgres::Error) -> String {
    match e.as_db_error() {
        Some(db) => {
            let mut msg = format!("{}: {}", db.code().code(), db.message());
            if let Some(detail) = db.detail() {
                msg.push_str(" — ");
                msg.push_str(detail);
            }
            if let Some(hint) = db.hint() {
                msg.push_str(" (hint: ");
                msg.push_str(hint);
                msg.push(')');
            }
            msg
        }
        None => e.to_string(),
    }
}

/// Chạy một closure bên trong transaction (BEGIN … COMMIT).
///
/// Nếu closure trả về `Err`, transaction sẽ được ROLLBACK tự động.
/// Nếu thành công, COMMIT và trả về kết quả.
pub async fn with_transaction<F, Fut, T>(client: &Client, f: F) -> AppResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    client
        .batch_execute("BEGIN")
        .await
        .map_err(|e| AppError::new(format!("Failed to begin transaction: {e}")))?;

    match f().await {
        Ok(result) => {
            client
                .batch_execute("COMMIT")
                .await
                .map_err(|e| AppError::new(format!("Failed to commit transaction: {e}")))?;
            Ok(result)
        }
        Err(e) => {
            let _ = client.batch_execute("ROLLBACK").await;
            Err(e)
        }
    }
}

/// Xác định đường dẫn tới file `config.ini`.
///
/// Production: `exe_dir/config/config.ini`.
/// Development: fallback về `CARGO_MANIFEST_DIR/config.ini`.
pub fn config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    if let Some(dir) = &exe_dir {
        // Ưu tiên config/config.ini (có file hoặc có thư mục config/)
        let config_dir = dir.join("config");
        let candidate = config_dir.join("config.ini");
        if candidate.exists() || config_dir.exists() {
            return candidate;
        }
    }

    // Fallback: cạnh .exe trực tiếp (tương thích cũ)
    if let Some(dir) = &exe_dir {
        let candidate = dir.join("config.ini");
        if candidate.exists() {
            return candidate;
        }
    }

    // Development: config.ini nằm trong thư mục src-tauri/
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("config.ini")
}
