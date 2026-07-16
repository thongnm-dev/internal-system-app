use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::models::explorer::{FileEntry, ReadDirResult, SearchResult};

#[cfg(target_os = "windows")]
fn is_hidden(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    fs::metadata(path)
        .map(|m| m.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .map(|n| n.to_string_lossy().starts_with('.'))
        .unwrap_or(false)
}

const MAX_SEARCH_RESULTS: usize = 500;

fn to_file_entry(path: &Path) -> Option<FileEntry> {
    let meta = fs::metadata(path).ok()?;
    let name = path.file_name()?.to_string_lossy().to_string();
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| {
            let secs = d.as_secs() as i64;
            chrono::DateTime::from_timestamp(secs, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    Some(FileEntry {
        name,
        path: path.to_string_lossy().to_string(),
        is_dir: meta.is_dir(),
        size: if meta.is_file() { meta.len() } else { 0 },
        modified,
        extension,
    })
}

pub fn read_dir(dir_path: &str) -> Result<ReadDirResult, String> {
    let path = Path::new(dir_path);
    if !path.exists() {
        return Err(format!("Path does not exist: {dir_path}"));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {dir_path}"));
    }

    let mut entries: Vec<FileEntry> = Vec::new();
    let read = fs::read_dir(path).map_err(|e| format!("Cannot read directory: {e}"))?;

    for entry in read.flatten() {
        let ep = entry.path();
        if is_hidden(&ep) {
            continue;
        }
        if let Some(fe) = to_file_entry(&ep) {
            entries.push(fe);
        }
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(ReadDirResult {
        path: path.to_string_lossy().to_string(),
        entries,
    })
}

pub fn search_files(root: &str, query: &str) -> Result<SearchResult, String> {
    let root_path = Path::new(root);
    if !root_path.is_dir() {
        return Err(format!("Not a directory: {root}"));
    }

    let query_lower = query.to_lowercase();
    let mut results: Vec<FileEntry> = Vec::new();
    let mut truncated = false;

    fn walk(dir: &Path, query: &str, results: &mut Vec<FileEntry>, truncated: &mut bool) {
        if *truncated {
            return;
        }
        let Ok(read) = fs::read_dir(dir) else {
            return;
        };
        for entry in read.flatten() {
            if results.len() >= MAX_SEARCH_RESULTS {
                *truncated = true;
                return;
            }
            let path = entry.path();
            if is_hidden(&path) {
                continue;
            }
            if let Some(name) = path.file_name() {
                let name_lower = name.to_string_lossy().to_lowercase();
                if name_lower.contains(query) {
                    if let Some(fe) = to_file_entry(&path) {
                        results.push(fe);
                    }
                }
            }
            if path.is_dir() {
                walk(&path, query, results, truncated);
            }
        }
    }

    walk(root_path, &query_lower, &mut results, &mut truncated);

    results.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(SearchResult {
        query: query.to_string(),
        root: root.to_string(),
        entries: results,
        truncated,
    })
}

pub fn open_in_explorer(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("Path does not exist: {path}"));
    }

    #[cfg(target_os = "windows")]
    {
        if p.is_dir() {
            std::process::Command::new("explorer")
                .arg(path)
                .spawn()
                .map_err(|e| format!("Failed to open explorer: {e}"))?;
        } else {
            std::process::Command::new("explorer")
                .args(["/select,", path])
                .spawn()
                .map_err(|e| format!("Failed to open explorer: {e}"))?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        let target = if p.is_file() {
            p.parent().map(|pp| pp.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string())
        } else {
            path.to_string()
        };
        std::process::Command::new("open")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("Failed to open Finder: {e}"))?;
    }

    Ok(())
}

pub fn get_drives() -> Vec<String> {
    let mut drives = Vec::new();
    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            if Path::new(&drive).exists() {
                drives.push(drive);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        drives.push("/".to_string());
    }
    drives
}
