//! Tauri command handlers cho module quản lý menu (governance).

use crate::models::menu_config::{MenuConfig, SaveAllMenuConfigsRequest, SaveMenuConfigRequest};
use crate::services::menu_config_service;

#[tauri::command]
pub async fn list_menu_configs() -> Result<Vec<MenuConfig>, String> {
    menu_config_service::list_menu_configs()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_menu_config(request: SaveMenuConfigRequest) -> Result<(), String> {
    menu_config_service::save_menu_config(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_all_menu_configs(
    request: SaveAllMenuConfigsRequest,
) -> Result<Vec<MenuConfig>, String> {
    menu_config_service::save_all_menu_configs(request)
        .await
        .map_err(|e| e.to_string())
}
