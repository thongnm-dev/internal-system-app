use crate::models::s3::{AwsStorage, DeleteUploadedItem, DownloadAvailability, LocalFileEntry, S3Config, S3ListResult, S3OperationResult, ScannedFile, UploadFileRequest};
use std::collections::HashMap;
use crate::services::s3_service;

#[tauri::command]
pub fn s3_check_config() -> Result<(), String> {
    s3_service::check_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn s3_get_config() -> Result<S3Config, String> {
    s3_service::get_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn s3_save_config(config: S3Config) -> Result<(), String> {
    s3_service::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_test_connection() -> Result<String, String> {
    s3_service::test_connection()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_list_objects(prefix: String) -> Result<S3ListResult, String> {
    s3_service::list_objects(prefix)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_download_objects(
    keys: Vec<String>,
    destination_dir: String,
    strip_prefix: String,
) -> Result<S3OperationResult, String> {
    s3_service::download_objects(keys, destination_dir, strip_prefix)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_upload_file(
    local_path: String,
    s3_key: String,
) -> Result<S3OperationResult, String> {
    s3_service::upload_file(local_path, s3_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_delete_objects(
    keys: Vec<String>,
) -> Result<S3OperationResult, String> {
    s3_service::delete_objects(keys)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_create_folder(prefix: String) -> Result<S3OperationResult, String> {
    s3_service::create_folder(prefix)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_scan_local_folder(folder_path: String) -> Result<Vec<LocalFileEntry>, String> {
    s3_service::scan_local_folder(folder_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_upload_folder(
    folder_path: String,
    s3_prefix: String,
) -> Result<S3OperationResult, String> {
    s3_service::upload_folder(folder_path, s3_prefix)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_list_upload_storages() -> Result<Vec<AwsStorage>, String> {
    s3_service::list_upload_storages()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_scan_upload_folder(dir_path: String) -> Result<Vec<ScannedFile>, String> {
    s3_service::scan_upload_folder(dir_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_upload_files(
    files: Vec<UploadFileRequest>,
    storage_name: String,
    subscribe: String,
    create_folder_same_name: bool,
) -> Result<S3OperationResult, String> {
    s3_service::upload_files(files, storage_name, subscribe, create_folder_same_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_list_delete_options(destination_code: String) -> Result<Vec<AwsStorage>, String> {
    s3_service::list_delete_options(destination_code)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_delete_uploaded_items(
    items: Vec<DeleteUploadedItem>,
) -> Result<S3OperationResult, String> {
    s3_service::delete_uploaded_items(items)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_get_browser_allowed_prefixes() -> Result<Vec<String>, String> {
    s3_service::get_browser_allowed_prefixes()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_list_download_storages() -> Result<Vec<AwsStorage>, String> {
    s3_service::list_download_storages()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_check_download_available(
    codes: Vec<String>,
) -> Result<HashMap<String, DownloadAvailability>, String> {
    s3_service::check_download_available(codes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_get_download_list(code: String) -> Result<Vec<String>, String> {
    s3_service::get_download_list(code)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_download_by_storage(
    code: String,
    bug_list: Vec<String>,
    local_path: String,
) -> Result<S3OperationResult, String> {
    s3_service::download_by_storage(code, bug_list, local_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_move_objects(
    code: String,
    items: Vec<String>,
) -> Result<S3OperationResult, String> {
    s3_service::move_s3_objects(code, items)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_move_browser_objects(
    keys: Vec<String>,
    destination_prefix: String,
) -> Result<S3OperationResult, String> {
    s3_service::move_browser_objects(keys, destination_prefix)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_delete_by_storage(
    code: String,
    items: Vec<String>,
) -> Result<S3OperationResult, String> {
    s3_service::delete_s3_objects_by_storage(code, items)
        .await
        .map_err(|e| e.to_string())
}
