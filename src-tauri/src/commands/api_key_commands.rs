use crate::domain::api_keys::models::ApiKeyApplication;
use crate::domain::api_keys::service;
use tauri::Manager;

#[tauri::command]
pub fn list_api_key_applications(app: tauri::AppHandle) -> Result<Vec<ApiKeyApplication>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
    service::list_api_key_applications(&app_data_dir).map_err(|error| error.to_string())
}
