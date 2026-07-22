use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::auth_store;
use crate::models::auth::{LoginRequest, LoginResponse};
use crate::utils::email;

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

pub async fn request_password_reset(username: &str) -> AppResult<String> {
    let username = username.trim();
    if username.is_empty() {
        return Err(AppError::new("Vui lòng nhập tên đăng nhập."));
    }

    let user = auth_store::find_user_by_username(username)
        .await?
        .ok_or_else(|| AppError::new("Tài khoản không tồn tại."))?;

    if !user.is_active {
        return Err(AppError::new("Tài khoản đã bị vô hiệu hóa. Vui lòng liên hệ quản trị viên."));
    }

    if user.email.trim().is_empty() {
        return Err(AppError::new("Tài khoản chưa được cấu hình email. Vui lòng liên hệ quản trị viên."));
    }

    let code = generate_code();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);

    email::send_reset_code(&user.email, &user.full_name, &code).await?;

    auth_store::save_reset_code(user.id, &code, expires_at).await?;

    Ok(mask_email(&user.email))
}

pub async fn verify_password_reset(username: &str, code: &str) -> AppResult<String> {
    let username = username.trim();
    let code = code.trim();

    if username.is_empty() || code.is_empty() {
        return Err(AppError::new("Vui lòng nhập đầy đủ thông tin."));
    }

    let user = auth_store::find_user_by_username(username)
        .await?
        .ok_or_else(|| AppError::new("Tài khoản không tồn tại."))?;

    let valid = auth_store::verify_reset_code(user.id, code).await?;
    if !valid {
        if !auth_store::has_unexpired_code(user.id).await? {
            return Err(AppError::new("Mã xác nhận đã hết hạn. Vui lòng yêu cầu mã mới."));
        }
        return Err(AppError::new("Mã xác nhận không đúng."));
    }

    let default_password = "Aa@123456";
    let password_hash = bcrypt::hash(default_password, 12)
        .map_err(|e| AppError::new(format!("Failed to hash password: {e}")))?;

    let updated = auth_store::reset_password(user.id, &password_hash).await?;
    if !updated {
        return Err(AppError::new("Đặt lại mật khẩu thất bại."));
    }

    Ok(default_password.to_string())
}

fn generate_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let n: u32 = rng.gen_range(0..1_000_000);
    format!("{n:06}")
}

fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 {
        return "***".to_string();
    }
    let local = parts[0];
    let domain = parts[1];
    let masked_local = if local.len() <= 2 {
        format!("{}***", &local[..1])
    } else {
        format!("{}***{}", &local[..2], &local[local.len() - 1..])
    };
    format!("{masked_local}@{domain}")
}
