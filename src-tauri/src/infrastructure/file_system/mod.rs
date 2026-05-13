use crate::app::error::AppError;
use crate::app::result::AppResult;
use std::env;
use std::path::{Path, PathBuf};

pub fn resolve_input_path(path: &str) -> AppResult<PathBuf> {
    let raw = path.trim();
    let candidate = if raw.is_empty() {
        PathBuf::from("pjjyuji_data_csv_20260513.csv")
    } else {
        PathBuf::from(raw)
    };

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
