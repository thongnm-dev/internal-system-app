/// Tiện ích hạ tầng dùng chung (network, time, encoding, database).
#[path = "../utils"]
mod utils {
    /// HTTP client wrapper cho gọi API bên ngoài (chưa sử dụng).
    #[allow(dead_code)]
    pub mod api_client;
    /// Đọc file CSV với hỗ trợ encoding Shift-JIS.
    pub mod csv_reader;
    /// Kiểm tra kết nối internet và lấy IP local.
    pub mod network;
    /// Kết nối PostgreSQL, tạo bảng và stored procedure.
    pub mod pgsql_connect;
    /// Hàm tiện ích lấy timestamp hiện tại.
    pub mod time;
    /// Đường dẫn dữ liệu và cấu hình ứng dụng (AppData + config.ini).
    pub mod app_config;
    /// Gửi email qua SMTP (dùng cho reset password, v.v.).
    pub mod email;
    /// Ghi log lỗi ra file (logs/errors_log.log).
    pub mod logger;
}
