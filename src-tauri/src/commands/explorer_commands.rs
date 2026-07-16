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
        .map_err(|e| format!("Thread error: {e}"))?
}

#[tauri::command]
pub fn explorer_open(path: String) -> Result<(), String> {
    explorer_service::open_in_explorer(&path)
}

#[tauri::command]
pub fn explorer_get_drives() -> Vec<String> {
    explorer_service::get_drives()
}
