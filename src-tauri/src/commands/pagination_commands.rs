use crate::services::pagination_service::{self, PaginationConfig};

#[tauri::command]
pub fn get_pagination_config() -> PaginationConfig {
    pagination_service::get_pagination_config()
}
