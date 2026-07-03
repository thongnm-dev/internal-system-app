use crate::app::error::AppError;
use crate::app::result::AppResult;
use encoding_rs::SHIFT_JIS;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CsvReader {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl CsvReader {
    pub fn from_path(path: &Path) -> AppResult<Self> {
        let content = read_with_encoding(path)?;
        Self::from_str(&content)
    }

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

    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    pub fn column_index(&self, name: &str) -> AppResult<usize> {
        self.headers
            .iter()
            .position(|h| h == name)
            .ok_or_else(|| AppError::new(format!("Required column was not found: {name}")))
    }

    pub fn get_value<'a>(&self, row: &'a [String], column_index: usize) -> &'a str {
        row.get(column_index).map(|v| v.as_str()).unwrap_or_default()
    }

    pub fn parse_i64(value: &str) -> i64 {
        value.trim().replace(',', "").parse::<i64>().unwrap_or(0)
    }
}

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

pub fn display_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

fn read_with_encoding(path: &Path) -> AppResult<String> {
    let bytes = fs::read(path)?;
    let (content, _, _) = SHIFT_JIS.decode(&bytes);
    Ok(content.into_owned())
}
