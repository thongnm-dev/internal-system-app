//! Business logic cho module quản lý menu (governance).

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::menu_config_store;
use crate::models::menu_config::{MenuConfig, SaveAllMenuConfigsRequest, SaveMenuConfigRequest};

pub async fn list_menu_configs() -> AppResult<Vec<MenuConfig>> {
    menu_config_store::list_all().await
}

pub async fn save_menu_config(request: SaveMenuConfigRequest) -> AppResult<()> {
    if request.key.trim().is_empty() {
        return Err(AppError::new("Menu key không được để trống."));
    }
    if request.title.trim().is_empty() {
        return Err(AppError::new("Menu title không được để trống."));
    }

    menu_config_store::upsert(&request).await
}

pub async fn save_all_menu_configs(request: SaveAllMenuConfigsRequest) -> AppResult<Vec<MenuConfig>> {
    for item in &request.items {
        if item.key.trim().is_empty() {
            return Err(AppError::new("Menu key không được để trống."));
        }
        if item.title.trim().is_empty() {
            return Err(AppError::new(format!(
                "Menu title không được để trống (key: {}).",
                item.key
            )));
        }
    }

    menu_config_store::save_all(&request.items).await?;
    menu_config_store::list_all().await
}
