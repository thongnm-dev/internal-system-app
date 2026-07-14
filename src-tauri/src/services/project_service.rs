//! Business logic cho module quản lý dự án.
//!
//! Xử lý validation, chuyển đổi dữ liệu request, và gọi tầng database.
//! Không chứa SQL — mọi truy vấn đều delegate sang `project_store`.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::project_store;
use crate::models::project::{
    BacklogProjectLookup, CreateProjectRequest, CreateProjectTaskRequest, ProjectDetail,
    ProjectSummary, ProjectTask,
};

/// Tạo dự án mới.
///
/// Validate mã dự án và tên không được rỗng, kiểm tra trùng mã dự án,
/// sau đó lưu vào database và trả về bản ghi hoàn chỉnh.
pub async fn create_project(request: CreateProjectRequest) -> AppResult<ProjectDetail> {
    let code = request.code.trim().to_string();
    let name = request.name.trim().to_string();

    if code.is_empty() {
        return Err(AppError::new("Project code is required."));
    }
    if name.is_empty() {
        return Err(AppError::new("Project name is required."));
    }

    // Kiểm tra trùng mã dự án (case-insensitive)
    if project_store::code_exists(&code, None).await? {
        return Err(AppError::new(format!(
            "Project code '{}' already exists.",
            code
        )));
    }

    // Xây dựng ProjectDetail với các trường optional có giá trị mặc định
    let project = ProjectDetail {
        id: 0, // Database tự sinh ID
        code,
        name,
        client: request.client.unwrap_or_default(),
        backlog_key: request.backlog_key.unwrap_or_default(),
        backlog_url: request.backlog_url.unwrap_or_default(),
        backlog_space: request.backlog_space.unwrap_or_default(),
        is_active: true,
        members: request.members,
        created_at: String::new(), // Database tự sinh timestamp
        updated_at: String::new(),
    };

    project_store::insert_project(&project).await
}

/// Cập nhật thông tin dự án theo ID.
///
/// Validate dữ liệu, kiểm tra dự án tồn tại, kiểm tra trùng mã
/// (loại trừ chính dự án đang sửa), rồi cập nhật database.
pub async fn update_project(
    project_id: i32,
    request: CreateProjectRequest,
) -> AppResult<ProjectDetail> {
    let code = request.code.trim().to_string();
    let name = request.name.trim().to_string();

    if code.is_empty() {
        return Err(AppError::new("Project code is required."));
    }
    if name.is_empty() {
        return Err(AppError::new("Project name is required."));
    }

    // Lấy bản ghi hiện tại để giữ lại is_active và created_at
    let existing = project_store::find_by_id(project_id)
        .await?
        .ok_or_else(|| AppError::new(format!("Project '{}' not found.", project_id)))?;

    // Kiểm tra trùng mã, loại trừ chính dự án đang sửa
    if project_store::code_exists(&code, Some(project_id)).await? {
        return Err(AppError::new(format!(
            "Project code '{}' already exists.",
            code
        )));
    }

    let project = ProjectDetail {
        id: project_id,
        code,
        name,
        client: request.client.unwrap_or_default(),
        backlog_key: request.backlog_key.unwrap_or_default(),
        backlog_url: request.backlog_url.unwrap_or_default(),
        backlog_space: request.backlog_space.unwrap_or_default(),
        is_active: existing.is_active,
        members: request.members,
        created_at: existing.created_at,
        updated_at: String::new(), // Database tự cập nhật timestamp
    };

    project_store::update_project(project_id, &project).await
}

/// Lấy thông tin chi tiết dự án theo ID (bao gồm danh sách thành viên).
pub async fn get_project_detail(project_id: i32) -> AppResult<ProjectDetail> {
    project_store::find_by_id(project_id)
        .await?
        .ok_or_else(|| AppError::new(format!("Project '{}' not found.", project_id)))
}

/// Lấy danh sách tóm tắt toàn bộ dự án.
pub async fn list_projects() -> AppResult<Vec<ProjectSummary>> {
    project_store::list_all().await
}

/// Xóa dự án theo ID. Trả về lỗi nếu không tìm thấy.
pub async fn delete_project(project_id: i32) -> AppResult<()> {
    if !project_store::delete_by_id(project_id).await? {
        return Err(AppError::new(format!(
            "Project '{}' not found.",
            project_id
        )));
    }
    Ok(())
}

/// Tra cứu dự án từ Backlog API theo project key.
/// Hiện tại chưa được cấu hình — luôn trả về lỗi.
pub fn get_backlog_project_by_key(_project_key: String) -> AppResult<BacklogProjectLookup> {
    Err(AppError::new("Backlog project lookup is not configured."))
}

/// Tạo task mới cho dự án.
pub async fn create_project_task(
    project_id: i32,
    request: CreateProjectTaskRequest,
) -> AppResult<ProjectTask> {
    let short_name = request.short_name.trim().to_string();
    if short_name.is_empty() {
        return Err(AppError::new("Task short name is required."));
    }

    let task_id = format!("task-{}", chrono::Utc::now().timestamp_millis());

    let task = ProjectTask {
        id: task_id,
        project_id,
        short_name,
        description: request.description.trim().to_string(),
        categories: request.categories,
        assignee: request.assignee.trim().to_string(),
        estimate_hour: request.estimate_hour.trim().to_string(),
        due_date: request.due_date.trim().to_string(),
        issue_key: request.issue_key.trim().to_string(),
        is_user_added: true,
        created_at: String::new(),
    };

    project_store::insert_task(&task).await
}

/// Cập nhật task đã có của dự án.
pub async fn update_project_task(
    task_id: String,
    request: CreateProjectTaskRequest,
) -> AppResult<ProjectTask> {
    let short_name = request.short_name.trim().to_string();
    if short_name.is_empty() {
        return Err(AppError::new("Task short name is required."));
    }

    let task = ProjectTask {
        id: task_id,
        project_id: 0,
        short_name,
        description: request.description.trim().to_string(),
        categories: request.categories,
        assignee: request.assignee.trim().to_string(),
        estimate_hour: request.estimate_hour.trim().to_string(),
        due_date: request.due_date.trim().to_string(),
        issue_key: request.issue_key.trim().to_string(),
        is_user_added: true,
        created_at: String::new(),
    };

    project_store::update_task(&task).await
}

/// Lấy danh sách task của dự án.
pub async fn list_project_tasks(project_id: i32) -> AppResult<Vec<ProjectTask>> {
    project_store::list_tasks_by_project(project_id).await
}

/// Xóa task theo ID.
pub async fn delete_project_task(task_id: String) -> AppResult<()> {
    if !project_store::delete_task(&task_id).await? {
        return Err(AppError::new(format!("Task '{}' not found.", task_id)));
    }
    Ok(())
}
