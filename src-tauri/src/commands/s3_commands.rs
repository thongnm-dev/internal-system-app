use crate::models::s3::{S3Config, S3ListResult, S3OperationResult};
use crate::services::s3_service;

#[tauri::command]
pub fn s3_load_config() -> Result<S3Config, String> {
    s3_service::load_config_from_ini().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_test_connection(config: S3Config) -> Result<String, String> {
    s3_service::test_connection(config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_list_objects(config: S3Config, prefix: String) -> Result<S3ListResult, String> {
    s3_service::list_objects(config, prefix)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_download_objects(
    config: S3Config,
    keys: Vec<String>,
    destination_dir: String,
) -> Result<S3OperationResult, String> {
    s3_service::download_objects(config, keys, destination_dir)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_upload_file(
    config: S3Config,
    local_path: String,
    s3_key: String,
) -> Result<S3OperationResult, String> {
    s3_service::upload_file(config, local_path, s3_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_delete_objects(
    config: S3Config,
    keys: Vec<String>,
) -> Result<S3OperationResult, String> {
    s3_service::delete_objects(config, keys)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn s3_create_folder(config: S3Config, prefix: String) -> Result<S3OperationResult, String> {
    s3_service::create_folder(config, prefix)
        .await
        .map_err(|e| e.to_string())
}
