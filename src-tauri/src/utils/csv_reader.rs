//! Tiện ích đọc file CSV với hỗ trợ encoding Shift-JIS.
//!
//! File CSV công việc từ hệ thống Nhật Bản thường dùng encoding Shift-JIS.
//! Module này tự động decode sang UTF-8 trước khi parse.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use encoding_rs::SHIFT_JIS;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Parser CSV hỗ trợ Shift-JIS, lưu trữ headers và rows đã parse.
pub struct CsvReader {
    /// Danh sách tên cột (header row).
    headers: Vec<String>,
    /// Dữ liệu từng dòng, mỗi dòng là một vector các giá trị đã trim.
    rows: Vec<Vec<String>>,
}

impl CsvReader {
    /// Đọc và parse file CSV từ đường dẫn.
    /// Tự động decode Shift-JIS → UTF-8.
    pub fn from_path(path: &Path) -> AppResult<Self> {
        let content = read_with_encoding(path)?;
        Self::from_str(&content)
    }

    /// Parse nội dung CSV từ chuỗi UTF-8.
    pub fn from_str(content: &str) -> AppResult<Self> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        let headers: Vec<String> = reader
            .headers()?
            .iter()
            .map(|h| h.to_string())
            .collect();

        let mut rows = Vec::new();
        for record in reader.records() {
            let record = record?;
            rows.push(record.iter().map(|v| v.trim().to_string()).collect());
        }

        Ok(Self { headers, rows })
    }

    /// Trả về danh sách tên cột.
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// Trả về tất cả các dòng dữ liệu (không bao gồm header).
    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    /// Tìm chỉ số cột theo tên. Trả về lỗi nếu không tìm thấy.
    pub fn column_index(&self, name: &str) -> AppResult<usize> {
        self.headers
            .iter()
            .position(|h| h == name)
            .ok_or_else(|| AppError::new(format!("Required column was not found: {name}")))
    }

    /// Lấy giá trị tại vị trí `column_index` trong một dòng.
    /// Trả về chuỗi rỗng nếu chỉ số nằm ngoài phạm vi.
    pub fn get_value<'a>(&self, row: &'a [String], column_index: usize) -> &'a str {
        row.get(column_index).map(|v| v.as_str()).unwrap_or_default()
    }

    /// Parse chuỗi số (có thể chứa dấu phẩy ngăn cách hàng nghìn) thành i64.
    /// Trả về 0 nếu parse thất bại.
    pub fn parse_i64(value: &str) -> i64 {
        value.trim().replace(',', "").parse::<i64>().unwrap_or(0)
    }
}

/// Phân giải đường dẫn file CSV đầu vào.
///
/// Thử lần lượt: đường dẫn tuyệt đối → thư mục hiện tại → parent → grandparent.
/// Trả về lỗi nếu không tìm thấy file ở bất kỳ vị trí nào.
pub fn resolve_input_path(path: &str) -> AppResult<PathBuf> {
    let raw = path.trim();
    if raw.is_empty() {
        return Err(AppError::new("CSV file path is empty"));
    }
    let candidate = PathBuf::from(raw);

    if candidate.is_absolute() && candidate.exists() {
        return Ok(candidate);
    }

    let current = env::current_dir()?;
    let possible_paths = [
        current.join(&candidate),
        current.join("..").join(&candidate),
        current.join("..").join("..").join(&candidate),
    ];

    possible_paths
        .into_iter()
        .find(|path| path.exists())
        .ok_or_else(|| AppError::new(format!("CSV file was not found: {}", candidate.display())))
}

/// Hiển thị đường dẫn dạng canonical (absolute).
/// Fallback về đường dẫn gốc nếu canonicalize thất bại.
pub fn display_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

/// Đọc file với encoding Shift-JIS và trả về chuỗi UTF-8.
fn read_with_encoding(path: &Path) -> AppResult<String> {
    let bytes = fs::read(path)?;
    let (content, _, _) = SHIFT_JIS.decode(&bytes);
    Ok(content.into_owned())
}
