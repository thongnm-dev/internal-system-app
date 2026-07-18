//! Business logic cho màn hình SQL Editor.
//!
//! Quản lý danh sách kết nối (CRUD, lưu cục bộ) và chạy native query
//! trên kết nối được chọn. Phiên bản này chỉ hỗ trợ PostgreSQL.

use tokio_postgres::SimpleQueryMessage;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::sql_connection_store::{self, SqlConnectionData};
use crate::models::sql_editor::{
    DbTable, QueryResult, SaveSqlConnectionRequest, SqlConnection,
};
use crate::utils::pgsql_connect::{self, PgConfig};
use crate::utils::time::current_timestamp;

/// Thời gian chờ tối đa cho một lần chạy query (giây).
const QUERY_TIMEOUT_SECS: u64 = 30;

/// Chỉ hỗ trợ loại DB này ở phiên bản hiện tại.
const SUPPORTED_DB_TYPE: &str = "postgres";

/// Liệt kê tất cả kết nối đã lưu.
pub fn list_connections() -> AppResult<Vec<SqlConnection>> {
    Ok(sql_connection_store::load()?.connections)
}

/// Kiểm tra các trường bắt buộc của request.
fn validate(request: &SaveSqlConnectionRequest) -> AppResult<()> {
    if request.name.trim().is_empty() {
        return Err(AppError::new("Tên kết nối là bắt buộc."));
    }
    if request.host.trim().is_empty() {
        return Err(AppError::new("Host là bắt buộc."));
    }
    if request.database.trim().is_empty() {
        return Err(AppError::new("Tên database là bắt buộc."));
    }
    if request.port == 0 {
        return Err(AppError::new("Port không hợp lệ."));
    }
    Ok(())
}

/// Thêm mới hoặc cập nhật một kết nối, trả về bản ghi sau khi lưu.
pub fn save_connection(request: SaveSqlConnectionRequest) -> AppResult<SqlConnection> {
    validate(&request)?;

    let mut data = sql_connection_store::load()?;

    if request.id > 0 {
        // Cập nhật kết nối đã có.
        let existing = data
            .connections
            .iter_mut()
            .find(|c| c.id == request.id)
            .ok_or_else(|| AppError::new("Không tìm thấy kết nối cần cập nhật."))?;
        existing.name = request.name.trim().to_string();
        existing.db_type = request.db_type.trim().to_string();
        existing.host = request.host.trim().to_string();
        existing.port = request.port;
        existing.database = request.database.trim().to_string();
        existing.username = request.username.trim().to_string();
        existing.password = request.password.clone();
        let saved = existing.clone();
        sql_connection_store::save(&data)?;
        return Ok(saved);
    }

    // Thêm mới.
    let id = next_id(&mut data);
    let connection = SqlConnection {
        id,
        name: request.name.trim().to_string(),
        db_type: request.db_type.trim().to_string(),
        host: request.host.trim().to_string(),
        port: request.port,
        database: request.database.trim().to_string(),
        username: request.username.trim().to_string(),
        password: request.password.clone(),
        created_at: current_timestamp(),
    };
    data.connections.push(connection.clone());
    sql_connection_store::save(&data)?;
    Ok(connection)
}

/// Xoá một kết nối theo id.
pub fn delete_connection(id: i64) -> AppResult<()> {
    let mut data = sql_connection_store::load()?;
    let before = data.connections.len();
    data.connections.retain(|c| c.id != id);
    if data.connections.len() == before {
        return Err(AppError::new("Không tìm thấy kết nối cần xoá."));
    }
    sql_connection_store::save(&data)?;
    Ok(())
}

/// Thử kết nối với cấu hình cho trước (không lưu). Dùng cho nút "Test".
pub async fn test_connection(request: SaveSqlConnectionRequest) -> AppResult<()> {
    validate(&request)?;
    ensure_supported(&request.db_type)?;
    let config = request_to_pg_config(&request);
    pgsql_connect::test_connection(&config).await
}

/// Chạy native query trên kết nối được chọn và trả về kết quả.
pub async fn run_query(connection_id: i64, sql: String) -> AppResult<QueryResult> {
    let statement = sql.trim();
    if statement.is_empty() {
        return Err(AppError::new("Câu lệnh SQL đang trống."));
    }

    let connection = find_connection(connection_id)?;
    ensure_supported(&connection.db_type)?;
    let config = connection_to_pg_config(&connection);

    let sql_owned = statement.to_string();
    let timeout = std::time::Duration::from_secs(QUERY_TIMEOUT_SECS);

    let work = async move {
        let start = std::time::Instant::now();
        let client = pgsql_connect::connect_with(&config).await?;
        let messages = client
            .simple_query(&sql_owned)
            .await
            .map_err(|e| AppError::new(format!("Lỗi khi chạy query: {e}")))?;
        let execution_ms = start.elapsed().as_millis() as i64;

        let mut columns: Vec<String> = Vec::new();
        let mut rows: Vec<Vec<Option<String>>> = Vec::new();
        let mut affected: i64 = 0;

        for message in messages {
            match message {
                SimpleQueryMessage::Row(row) => {
                    if columns.is_empty() {
                        columns = row
                            .columns()
                            .iter()
                            .map(|c| c.name().to_string())
                            .collect();
                    }
                    let mut record = Vec::with_capacity(row.len());
                    for index in 0..row.len() {
                        record.push(row.get(index).map(|value| value.to_string()));
                    }
                    rows.push(record);
                }
                SimpleQueryMessage::CommandComplete(count) => {
                    affected = count as i64;
                }
                _ => {}
            }
        }

        let has_result_set = !columns.is_empty();
        Ok::<QueryResult, AppError>(QueryResult {
            row_count: rows.len() as i64,
            columns,
            rows,
            affected,
            execution_ms,
            has_result_set,
        })
    };

    tokio::time::timeout(timeout, work).await.map_err(|_| {
        AppError::new(format!(
            "Query quá thời gian chờ ({QUERY_TIMEOUT_SECS}s)."
        ))
    })?
}

/// Đọc schema (danh sách bảng + cột) của kết nối để phục vụ autocomplete.
///
/// Chỉ lấy các schema do người dùng tạo (bỏ `pg_catalog`, `information_schema`).
pub async fn get_schema(connection_id: i64) -> AppResult<Vec<DbTable>> {
    let connection = find_connection(connection_id)?;
    ensure_supported(&connection.db_type)?;
    let config = connection_to_pg_config(&connection);
    let timeout = std::time::Duration::from_secs(QUERY_TIMEOUT_SECS);

    let work = async move {
        let client = pgsql_connect::connect_with(&config).await?;
        let rows = client
            .query(
                "SELECT table_schema::text, table_name::text, column_name::text \
                 FROM information_schema.columns \
                 WHERE table_schema NOT IN ('pg_catalog', 'information_schema') \
                 ORDER BY table_schema, table_name, ordinal_position",
                &[],
            )
            .await
            .map_err(|e| AppError::new(format!("Không đọc được schema: {e}")))?;

        // Gom cột theo từng bảng (rows đã được sắp theo schema, table).
        let mut tables: Vec<DbTable> = Vec::new();
        for row in rows {
            let schema: String = row.get(0);
            let name: String = row.get(1);
            let column: String = row.get(2);
            match tables.last_mut() {
                Some(last) if last.schema == schema && last.name == name => {
                    last.columns.push(column);
                }
                _ => tables.push(DbTable {
                    schema,
                    name,
                    columns: vec![column],
                }),
            }
        }
        Ok::<Vec<DbTable>, AppError>(tables)
    };

    tokio::time::timeout(timeout, work)
        .await
        .map_err(|_| AppError::new("Đọc schema quá thời gian chờ."))?
}

/// Lấy id kế tiếp và tăng bộ đếm; khởi tạo từ 1 nếu file còn mới.
fn next_id(data: &mut SqlConnectionData) -> i64 {
    if data.next_id < 1 {
        // Suy ra từ id lớn nhất hiện có để tránh trùng khi file thiếu `next_id`.
        let max = data.connections.iter().map(|c| c.id).max().unwrap_or(0);
        data.next_id = max + 1;
    }
    let id = data.next_id;
    data.next_id += 1;
    id
}

/// Tìm một kết nối theo id.
fn find_connection(id: i64) -> AppResult<SqlConnection> {
    sql_connection_store::load()?
        .connections
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| AppError::new("Không tìm thấy kết nối."))
}

/// Chỉ cho phép loại DB đang được hỗ trợ.
fn ensure_supported(db_type: &str) -> AppResult<()> {
    if db_type.trim().to_lowercase() != SUPPORTED_DB_TYPE {
        return Err(AppError::new(format!(
            "Phiên bản hiện tại chỉ hỗ trợ PostgreSQL (loại '{db_type}' chưa được hỗ trợ)."
        )));
    }
    Ok(())
}

fn request_to_pg_config(request: &SaveSqlConnectionRequest) -> PgConfig {
    PgConfig {
        host: request.host.trim().to_string(),
        port: request.port,
        dbname: request.database.trim().to_string(),
        user: request.username.trim().to_string(),
        password: request.password.clone(),
    }
}

fn connection_to_pg_config(connection: &SqlConnection) -> PgConfig {
    PgConfig {
        host: connection.host.clone(),
        port: connection.port,
        dbname: connection.database.clone(),
        user: connection.username.clone(),
        password: connection.password.clone(),
    }
}
