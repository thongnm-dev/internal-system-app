//! Business logic cho module cài đặt ứng dụng.
//!
//! Tất cả dữ liệu settings đều đọc/ghi từ database:
//! - User profile → bảng `users`
//! - Theme, language → bảng `user_settings`

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::settings::{AppSettings, SaveSettingsRequest, UserProfile};
use crate::utils::pgsql_connect;

pub async fn get_settings(user_id: i32) -> AppResult<AppSettings> {
    let client = pgsql_connect::connect().await?;

    let user_row = client
        .query_opt("SELECT * FROM sp_user_select_by_id($1)", &[&user_id])
        .await
        .map_err(|e| AppError::new(format!("Failed to load user profile: {e}")))?
        .ok_or_else(|| AppError::new(format!("User '{user_id}' not found.")))?;

    let user = UserProfile {
        username: user_row.get("username"),
        password: String::new(),
        full_name: user_row.get("full_name"),
        email: user_row.get("email"),
        phone: user_row.get("phone"),
        address: user_row.get("address"),
        position: user_row.get("position"),
    };

    let prefs_row = client
        .query_opt(
            "SELECT theme, language FROM user_settings WHERE user_id = $1",
            &[&user_id],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to load user settings: {e}")))?;

    let (theme, language) = match prefs_row {
        Some(row) => (row.get::<_, String>("theme"), row.get::<_, String>("language")),
        None => ("light".to_string(), "vi".to_string()),
    };

    Ok(AppSettings {
        user,
        theme,
        language,
    })
}

pub async fn save_settings(request: SaveSettingsRequest) -> AppResult<AppSettings> {
    let theme = request.theme.trim().to_string();
    if theme != "light" && theme != "dark" {
        return Err(AppError::new(
            "Theme không hợp lệ (chỉ chấp nhận light hoặc dark).",
        ));
    }

    let language = request.language.trim().to_string();
    if language != "vi" && language != "en" && language != "ja" {
        return Err(AppError::new(
            "Ngôn ngữ không hợp lệ (chỉ chấp nhận vi, en hoặc ja).",
        ));
    }

    let user_id = request.user_id;
    let client = pgsql_connect::connect().await?;

    client
        .query_opt(
            "SELECT * FROM sp_user_update($1, $2, $3, $4, $5, $6, $7)",
            &[
                &user_id,
                &request.user.full_name.trim(),
                &request.user.email.trim(),
                &request.user.phone.trim(),
                &request.user.address.trim(),
                &request.user.position.trim(),
                &true,
            ],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to update user profile: {e}")))?
        .ok_or_else(|| AppError::new(format!("User '{user_id}' not found.")))?;

    client
        .execute(
            "INSERT INTO user_settings (user_id, theme, language)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id) DO UPDATE SET theme = $2, language = $3, updated_at = NOW()",
            &[&user_id, &theme, &language],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to save user settings: {e}")))?;

    get_settings(user_id).await
}
