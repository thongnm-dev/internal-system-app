use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::infrastructure::csv::encoding::read_shift_jis_text;
use std::path::Path;

pub fn read_shift_jis_csv(path: &Path) -> AppResult<String> {
    read_shift_jis_text(path)
}

pub fn get_required_index(headers: &csv::StringRecord, name: &str) -> AppResult<usize> {
    headers
        .iter()
        .position(|header| header == name)
        .ok_or_else(|| AppError::new(format!("Required column was not found: {name}")))
}

pub fn parse_minutes(value: Option<&str>) -> i64 {
    value
        .unwrap_or_default()
        .trim()
        .replace(',', "")
        .parse::<i64>()
        .unwrap_or(0)
}
