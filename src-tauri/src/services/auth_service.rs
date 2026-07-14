//! Business logic cho module xác thực người dùng.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::auth_store;
use crate::models::auth::{LoginRequest, LoginResponse};

pub async fn login(request: LoginRequest) -> AppResult<LoginResponse> {
    let username = request.username.trim();
    let password = request.password.trim();

    if username.is_empty() || password.is_empty() {
        return Err(AppError::new("Username and password are required."));
    }

    let user = auth_store::find_user_by_username(username)
        .await?
        .ok_or_else(|| AppError::new("Invalid username or password."))?;

    if !user.is_active {
        return Err(AppError::new("Account is disabled. Please contact administrator."));
    }

    let valid = bcrypt::verify(password, &user.password_hash)
        .map_err(|e| AppError::new(format!("Password verification failed: {e}")))?;

    if !valid {
        return Err(AppError::new("Invalid username or password."));
    }

    let roles = auth_store::get_user_roles(user.id).await?;

    Ok(LoginResponse {
        user_id: user.id,
        username: user.username,
        full_name: user.full_name,
        email: user.email,
        roles,
    })
}
