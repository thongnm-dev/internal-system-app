use crate::models::s3::{S3ListResult, S3OperationResult};
use crate::services::s3_service;

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
) -> Result<S3OperationResult, String> {
    s3_service::download_objects(keys, destination_dir)
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
