use crate::models::explorer::{ReadDirResult, SearchResult};
use crate::services::explorer_service;

#[tauri::command]
pub fn explorer_read_dir(path: String) -> Result<ReadDirResult, String> {
    explorer_service::read_dir(&path)
}

#[tauri::command]
pub async fn explorer_search(root: String, query: String) -> Result<SearchResult, String> {
    tauri::async_runtime::spawn_blocking(move || explorer_service::search_files(&root, &query))
        .await
        .map_err(crate::app::error::log_err)?
}

#[tauri::command]
pub fn explorer_open(path: String) -> Result<(), String> {
    explorer_service::open_in_explorer(&path)
}

#[tauri::command]
pub fn explorer_read_text_file(path: String) -> Result<String, String> {
    explorer_service::read_text_file(&path)
}

#[tauri::command]
pub fn explorer_get_drives() -> Vec<String> {
    explorer_service::get_drives()
}

#[tauri::command]
pub fn explorer_rename(path: String, new_name: String) -> Result<(), String> {
    explorer_service::rename_entry(&path, &new_name)
}

#[tauri::command]
pub fn explorer_delete(paths: Vec<String>) -> Result<(), String> {
    explorer_service::delete_entries(&paths)
}

#[tauri::command]
pub fn explorer_create_file(dir: String, name: String) -> Result<String, String> {
    explorer_service::create_file(&dir, &name)
}

#[tauri::command]
pub fn explorer_create_folder(dir: String, name: String) -> Result<String, String> {
    explorer_service::create_folder(&dir, &name)
}

#[tauri::command]
pub async fn explorer_copy_bugs(
    source_dir: String,
    dest_dir: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        explorer_service::copy_bug_files(&source_dir, &dest_dir)
    })
    .await
    .map_err(crate::app::error::log_err)?
}

#[tauri::command]
pub async fn explorer_paste(
    sources: Vec<String>,
    dest_dir: String,
    cut: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        explorer_service::paste_entries(&sources, &dest_dir, cut)
    })
    .await
    .map_err(crate::app::error::log_err)?
}
