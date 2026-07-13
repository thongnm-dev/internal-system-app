//! Tauri command handlers cho module quản lý dự án.
//!
//! Mỗi hàm là một `#[tauri::command]` async, nhận tham số từ frontend
//! qua IPC invoke, gọi service tương ứng và trả kết quả về frontend.

use crate::models::project::{
    BacklogProjectLookup, CreateProjectRequest, ProjectDetail, ProjectSummary,
};
use crate::services::project_service;

/// Tạo dự án mới. Frontend gửi `CreateProjectRequest` chứa thông tin dự án
/// và danh sách thành viên.
#[tauri::command]
pub async fn create_project(request: CreateProjectRequest) -> Result<ProjectDetail, String> {
    project_service::create_project(request)
        .await
        .map_err(|e| e.to_string())
}

/// Cập nhật thông tin dự án theo `project_id`.
#[tauri::command]
pub async fn update_project(
    project_id: i32,
    request: CreateProjectRequest,
) -> Result<ProjectDetail, String> {
    project_service::update_project(project_id, request)
        .await
        .map_err(|e| e.to_string())
}

/// Lấy thông tin chi tiết dự án theo `project_id`, bao gồm danh sách thành viên.
#[tauri::command]
pub async fn get_project_detail(project_id: i32) -> Result<ProjectDetail, String> {
    project_service::get_project_detail(project_id)
        .await
        .map_err(|e| e.to_string())
}

/// Lấy danh sách tóm tắt toàn bộ dự án (không bao gồm chi tiết thành viên).
#[tauri::command]
pub async fn list_projects() -> Result<Vec<ProjectSummary>, String> {
    project_service::list_projects()
        .await
        .map_err(|e| e.to_string())
}

/// Xóa dự án theo `project_id`. Thành viên bị xóa theo (ON DELETE CASCADE).
#[tauri::command]
pub async fn delete_project(project_id: i32) -> Result<(), String> {
    project_service::delete_project(project_id)
        .await
        .map_err(|e| e.to_string())
}

/// Tra cứu dự án từ Backlog API theo `project_key`.
/// Hiện tại chưa cấu hình — luôn trả về lỗi.
#[tauri::command]
pub fn get_backlog_project_by_key(project_key: String) -> Result<BacklogProjectLookup, String> {
    project_service::get_backlog_project_by_key(project_key).map_err(|e| e.to_string())
}
