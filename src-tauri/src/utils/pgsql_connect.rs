//! Tiện ích kết nối PostgreSQL dùng chung cho toàn bộ ứng dụng.
//!
//! Đọc thông tin kết nối từ file `config.ini` (section `[database]`),
//! tạo bảng và stored procedure khi khởi động ứng dụng.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use ini::Ini;
use std::path::PathBuf;
use tokio_postgres::{Client, NoTls};

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

/// Tạo các bảng cần thiết nếu chưa tồn tại, sau đó tạo/cập nhật stored procedure.
///
/// Được gọi một lần khi ứng dụng khởi động (trong `setup` hook của Tauri).
pub async fn ensure_tables(client: &Client) -> AppResult<()> {
    client
        .batch_execute(
            "
            -- Bảng danh sách dự án
            CREATE TABLE IF NOT EXISTS projects (
                id            SERIAL       PRIMARY KEY,
                code          VARCHAR(20)  NOT NULL UNIQUE,
                name          VARCHAR(200) NOT NULL,
                client        VARCHAR(200) NOT NULL DEFAULT '',
                backlog_key   VARCHAR(20)  NOT NULL DEFAULT '',
                backlog_url   TEXT         NOT NULL DEFAULT '',
                backlog_space VARCHAR(100) NOT NULL DEFAULT '',
                is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
                created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
                updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
            );

            -- Bảng thành viên dự án (quan hệ N-N giữa project và user)
            CREATE TABLE IF NOT EXISTS project_members (
                id         SERIAL       PRIMARY KEY,
                project_id INTEGER      NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                username   VARCHAR(100) NOT NULL,
                name       VARCHAR(300) NOT NULL,
                UNIQUE(project_id, username)
            );

            -- Bảng ghi chú công việc hằng ngày
            CREATE TABLE IF NOT EXISTS daily_work_notes (
                id         SERIAL       PRIMARY KEY,
                username   VARCHAR(100) NOT NULL,
                content    TEXT         NOT NULL,
                note_date  DATE         NOT NULL,
                status     VARCHAR(15)  NOT NULL DEFAULT 'completed'
                    CHECK (status IN ('completed', 'incomplete', 'reserved')),
                created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_daily_notes_user_date
                ON daily_work_notes(username, note_date);
            ",
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to create tables: {e}")))?;

    ensure_stored_procedures(client).await?;

    Ok(())
}

/// Tạo hoặc cập nhật toàn bộ stored procedure từ các file SQL
/// trong thư mục `docs/store-procedure/`.
///
/// Sử dụng `include_str!` để nhúng nội dung SQL vào binary tại compile-time,
/// đảm bảo stored procedure luôn đồng bộ với mã nguồn.
async fn ensure_stored_procedures(client: &Client) -> AppResult<()> {
    // === Project stored procedures ===

    client
        .batch_execute(include_str!("../../../docs/store-procedure/sp_project_insert.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_insert: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_select_by_id.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_select_by_id: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_select_list.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_select_list: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_select_code_exists.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_project_select_code_exists: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!("../../../docs/store-procedure/sp_project_update.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_update: {e}")))?;

    client
        .batch_execute(include_str!("../../../docs/store-procedure/sp_project_delete.sql"))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_delete: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_member_upsert.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_member_upsert: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_member_delete_by_project.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_project_member_delete_by_project: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_project_member_select.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_project_member_select: {e}")))?;

    // === Daily Work Notes stored procedures ===

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_insert.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_daily_note_insert: {e}")))?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_select_by_date.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_note_select_by_date: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_select_by_month.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_note_select_by_month: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_count_by_month.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_note_count_by_month: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_update_status.sql"
        ))
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to create sp_daily_note_update_status: {e}"
            ))
        })?;

    client
        .batch_execute(include_str!(
            "../../../docs/store-procedure/sp_daily_note_delete.sql"
        ))
        .await
        .map_err(|e| AppError::new(format!("Failed to create sp_daily_note_delete: {e}")))?;

    Ok(())
}

/// Xác định đường dẫn tới file `config.ini`.
///
/// Ưu tiên tìm cạnh file thực thi (production build),
/// nếu không tìm thấy thì fallback về thư mục `CARGO_MANIFEST_DIR` (development).
fn config_path() -> PathBuf {
    // Production: config.ini nằm cạnh file .exe
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    if let Some(dir) = exe_dir {
        let candidate = dir.join("config.ini");
        if candidate.exists() {
            return candidate;
        }
    }

    // Development: config.ini nằm trong thư mục src-tauri/
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("config.ini")
}
