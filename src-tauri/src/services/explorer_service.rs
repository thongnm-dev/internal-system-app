use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use crate::models::explorer::{FileEntry, ReadDirResult, SearchResult};

const MAX_SEARCH_RESULTS: usize = 500;

const SKIP_SEARCH_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "__pycache__",
    ".svn",
    ".hg",
    ".next",
    ".nuxt",
];

#[cfg(target_os = "windows")]
fn is_hidden(_path: &Path, meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0
}

#[cfg(not(target_os = "windows"))]
fn is_hidden(path: &Path, _meta: &fs::Metadata) -> bool {
    path.file_name()
        .map(|n| n.to_string_lossy().starts_with('.'))
        .unwrap_or(false)
}

fn build_file_entry(path: &Path, meta: &fs::Metadata) -> Option<FileEntry> {
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

    for dir_entry in read.flatten() {
        let ep = dir_entry.path();
        let Ok(meta) = dir_entry.metadata() else {
            continue;
        };
        if is_hidden(&ep, &meta) {
            continue;
        }
        if let Some(fe) = build_file_entry(&ep, &meta) {
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
    let hit_count = Arc::new(AtomicUsize::new(0));
    let hit_limit = Arc::new(AtomicBool::new(false));

    let prune_count = hit_count.clone();

    let mut results: Vec<FileEntry> = jwalk::WalkDir::new(root)
        .skip_hidden(false)
        .process_read_dir(move |_depth, _path, _state, children| {
            if prune_count.load(Ordering::Relaxed) >= MAX_SEARCH_RESULTS {
                children.clear();
                return;
            }
            children.retain(|child_result| {
                child_result.as_ref().map_or(false, |entry| {
                    if entry.file_type.is_dir() {
                        let name = entry.file_name.to_string_lossy();
                        !SKIP_SEARCH_DIRS.contains(&name.as_ref())
                    } else {
                        true
                    }
                })
            });
        })
        .into_iter()
        .filter_map(|entry_result| {
            if hit_count.load(Ordering::Relaxed) >= MAX_SEARCH_RESULTS {
                hit_limit.store(true, Ordering::Relaxed);
                return None;
            }
            let entry = entry_result.ok()?;
            if entry.depth == 0 {
                return None;
            }

            let name_lower = entry.file_name().to_string_lossy().to_lowercase();
            if !name_lower.contains(&query_lower) {
                return None;
            }

            let path = entry.path();
            let meta = entry.metadata().ok()?;
            hit_count.fetch_add(1, Ordering::Relaxed);
            build_file_entry(&path, &meta)
        })
        .collect();

    let truncated = hit_limit.load(Ordering::Relaxed);

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

const MAX_TEXT_FILE_SIZE: u64 = 2 * 1024 * 1024;

pub fn read_text_file(path: &str) -> Result<String, String> {
    let p = Path::new(path);
    if !p.is_file() {
        return Err(format!("Not a file: {path}"));
    }
    let size = fs::metadata(p).map_err(|e| format!("Cannot read file metadata: {e}"))?.len();
    if size > MAX_TEXT_FILE_SIZE {
        return Err("File too large to preview (> 2MB).".to_string());
    }
    fs::read_to_string(p).map_err(|e| format!("Cannot read file: {e}"))
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
            p.parent()
                .map(|pp| pp.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string())
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

pub fn rename_entry(path: &str, new_name: &str) -> Result<(), String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("Path does not exist: {path}"));
    }
    let parent = p.parent().ok_or("Cannot determine parent directory")?;
    let new_path = parent.join(new_name);
    if new_path.exists() {
        return Err(format!("'{}' already exists", new_name));
    }
    fs::rename(p, &new_path).map_err(|e| format!("Rename failed: {e}"))
}

pub fn delete_entries(paths: &[String]) -> Result<(), String> {
    for path in paths {
        let p = Path::new(path);
        if !p.exists() {
            continue;
        }
        if p.is_dir() {
            fs::remove_dir_all(p).map_err(|e| format!("Delete folder failed: {e}"))?;
        } else {
            fs::remove_file(p).map_err(|e| format!("Delete file failed: {e}"))?;
        }
    }
    Ok(())
}

pub fn create_file(dir: &str, name: &str) -> Result<String, String> {
    let dir_path = Path::new(dir);
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {dir}"));
    }
    let file_path = dir_path.join(name);
    if file_path.exists() {
        return Err(format!("'{}' already exists", name));
    }
    fs::File::create(&file_path).map_err(|e| format!("Create file failed: {e}"))?;
    Ok(file_path.to_string_lossy().to_string())
}

pub fn create_folder(dir: &str, name: &str) -> Result<String, String> {
    let dir_path = Path::new(dir);
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {dir}"));
    }
    let folder_path = dir_path.join(name);
    if folder_path.exists() {
        return Err(format!("'{}' already exists", name));
    }
    fs::create_dir(&folder_path).map_err(|e| format!("Create folder failed: {e}"))?;
    Ok(folder_path.to_string_lossy().to_string())
}

/// Idempotent `mkdir -p`: ensure `path` (and any missing parents) exists, returning the
/// canonical path. Does nothing if it already exists; errors only if a non-directory sits
/// at `path`. Used by AI Translate Cowork to guarantee `<projectDir>/input` and
/// `<projectDir>/output` are present.
pub fn ensure_dir(path: &str) -> Result<String, String> {
    let p = Path::new(path);
    if p.exists() {
        if !p.is_dir() {
            return Err(format!("Path exists but is not a directory: {path}"));
        }
    } else {
        fs::create_dir_all(p).map_err(|e| format!("Create directory failed: {e}"))?;
    }
    Ok(p.to_string_lossy().to_string())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Create dir failed: {e}"))?;
    for entry in fs::read_dir(src).map_err(|e| format!("Read dir failed: {e}"))? {
        let entry = entry.map_err(|e| format!("Read entry failed: {e}"))?;
        let src_child = entry.path();
        let dst_child = dst.join(entry.file_name());
        if src_child.is_dir() {
            copy_dir_recursive(&src_child, &dst_child)?;
        } else {
            fs::copy(&src_child, &dst_child).map_err(|e| format!("Copy file failed: {e}"))?;
        }
    }
    Ok(())
}

pub fn paste_entries(sources: &[String], dest_dir: &str, cut: bool) -> Result<(), String> {
    let dest = Path::new(dest_dir);
    if !dest.is_dir() {
        return Err(format!("Destination is not a directory: {dest_dir}"));
    }
    for source in sources {
        let src = Path::new(source);
        if !src.exists() {
            continue;
        }
        let name = src
            .file_name()
            .ok_or_else(|| format!("Invalid filename: {source}"))?;
        let target = dest.join(name);
        if cut {
            if fs::rename(src, &target).is_err() {
                if src.is_dir() {
                    copy_dir_recursive(src, &target)?;
                    fs::remove_dir_all(src)
                        .map_err(|e| format!("Remove source failed: {e}"))?;
                } else {
                    fs::copy(src, &target).map_err(|e| format!("Copy failed: {e}"))?;
                    fs::remove_file(src)
                        .map_err(|e| format!("Remove source failed: {e}"))?;
                }
            }
        } else if src.is_dir() {
            copy_dir_recursive(src, &target)?;
        } else {
            fs::copy(src, &target).map_err(|e| format!("Copy failed: {e}"))?;
        }
    }
    Ok(())
}

pub fn copy_bug_files(source_dir: &str, dest_dir: &str) -> Result<String, String> {
    use regex::Regex;

    let src = Path::new(source_dir);
    let dst = Path::new(dest_dir);

    if !src.is_dir() {
        return Err(format!("Source is not a directory: {source_dir}"));
    }
    if !dst.is_dir() {
        return Err(format!("Destination is not a directory: {dest_dir}"));
    }

    let entries: Vec<_> = fs::read_dir(src)
        .map_err(|e| format!("Read dir failed: {e}"))?
        .flatten()
        .filter(|e| e.path().is_dir())
        .collect();

    if entries.is_empty() {
        return Err("Không tìm thấy thư mục nào trong thư mục nguồn.".to_string());
    }

    let now = chrono::Local::now();
    let ym = now.format("%Y%m").to_string();
    let ymd = now.format("%Y%m%d").to_string();

    let ym_dir = dst.join(&ym);
    fs::create_dir_all(&ym_dir).map_err(|e| format!("Create dir failed: {e}"))?;

    let history_sub = if !ym_dir.join(&ymd).exists() {
        ymd.clone()
    } else {
        let mut suffix = 2;
        loop {
            let name = format!("{}_{:02}", ymd, suffix);
            if !ym_dir.join(&name).exists() {
                break name;
            }
            suffix += 1;
        }
    };
    let history_dir = ym_dir.join(&history_sub);
    fs::create_dir_all(&history_dir).map_err(|e| format!("Create history dir failed: {e}"))?;

    let re = Regex::new(r"(\d{4})").unwrap();
    let mut copied = 0u32;

    for entry in &entries {
        let folder_name = entry.file_name().to_string_lossy().to_string();
        let folder_path = entry.path();

        if let Some(caps) = re.captures(&folder_name) {
            let num: u32 = caps[1].parse().unwrap_or(0);
            if num > 0 {
                let start = ((num - 1) / 100) * 100 + 1;
                let end = start + 99;
                let range_name = format!("{:04}\u{FF5E}{:04}", start, end);

                let range_dir = dst.join(&range_name);
                fs::create_dir_all(&range_dir)
                    .map_err(|e| format!("Create range dir failed: {e}"))?;
                let range_target = range_dir.join(&folder_name);
                if !range_target.exists() {
                    copy_dir_recursive(&folder_path, &range_target)?;
                }

                let history_target = history_dir.join(&folder_name);
                if !history_target.exists() {
                    copy_dir_recursive(&folder_path, &history_target)?;
                }

                copied += 1;
            }
        }
    }

    if copied == 0 {
        return Err("Không tìm thấy thư mục nào có mã số 4 chữ số.".to_string());
    }

    Ok(format!(
        "Đã copy {} thư mục thành công.\nLịch sử: {}",
        copied,
        history_dir.to_string_lossy()
    ))
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
