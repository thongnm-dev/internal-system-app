use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::user_store;
use crate::models::user::{
    ChangePasswordRequest, CreateUserRequest, UpdateUserRequest, UserDetail, UserSummary,
};

pub async fn create_user(request: CreateUserRequest) -> AppResult<UserDetail> {
    let username = request.username.trim().to_string();
    let password = request.password.trim().to_string();
    let full_name = request.full_name.trim().to_string();

    if username.is_empty() {
        return Err(AppError::new("Username is required."));
    }
    if password.is_empty() {
        return Err(AppError::new("Password is required."));
    }
    if full_name.is_empty() {
        return Err(AppError::new("Full name is required."));
    }
    if request.roles.is_empty() {
        return Err(AppError::new("At least one role is required."));
    }

    if user_store::username_exists(&username, None).await? {
        return Err(AppError::new(format!(
            "Username '{}' already exists.",
            username
        )));
    }

    let password_hash = bcrypt::hash(&password, 12)
        .map_err(|e| AppError::new(format!("Failed to hash password: {e}")))?;

    user_store::insert_user(
        &username,
        &password_hash,
        &full_name,
        request.email.as_deref().unwrap_or(""),
        request.phone.as_deref().unwrap_or(""),
        request.address.as_deref().unwrap_or(""),
        request.position.as_deref().unwrap_or(""),
        &request.roles,
    )
    .await
}

pub async fn update_user(user_id: i32, request: UpdateUserRequest) -> AppResult<UserDetail> {
    let full_name = request.full_name.trim().to_string();

    if full_name.is_empty() {
        return Err(AppError::new("Full name is required."));
    }
    if request.roles.is_empty() {
        return Err(AppError::new("At least one role is required."));
    }

    user_store::find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::new(format!("User '{user_id}' not found.")))?;

    user_store::update_user(
        user_id,
        &full_name,
        request.email.as_deref().unwrap_or(""),
        request.phone.as_deref().unwrap_or(""),
        request.address.as_deref().unwrap_or(""),
        request.position.as_deref().unwrap_or(""),
        request.is_active,
        &request.roles,
    )
    .await
}

pub async fn get_user_detail(user_id: i32) -> AppResult<UserDetail> {
    user_store::find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::new(format!("User '{user_id}' not found.")))
}

pub async fn list_users() -> AppResult<Vec<UserSummary>> {
    user_store::list_all().await
}

pub async fn delete_user(user_id: i32) -> AppResult<()> {
    if !user_store::delete_by_id(user_id).await? {
        return Err(AppError::new(format!("User '{user_id}' not found.")));
    }
    Ok(())
}

pub async fn change_password(user_id: i32, request: ChangePasswordRequest) -> AppResult<()> {
    let password = request.new_password.trim().to_string();

    if password.is_empty() {
        return Err(AppError::new("New password is required."));
    }

    user_store::find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::new(format!("User '{user_id}' not found.")))?;

    let password_hash = bcrypt::hash(&password, 12)
        .map_err(|e| AppError::new(format!("Failed to hash password: {e}")))?;

    if !user_store::change_password(user_id, &password_hash).await? {
        return Err(AppError::new("Failed to change password."));
    }

    Ok(())
}

pub async fn list_roles() -> AppResult<Vec<String>> {
    user_store::list_roles().await
}
