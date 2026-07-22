use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::utils::app_config;
use ini::Ini;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    from_name: String,
    from_email: String,
}

fn load_smtp_config() -> AppResult<SmtpConfig> {
    let path = app_config::config_path();
    let ini = Ini::load_from_file(&path)
        .map_err(|e| AppError::new(format!("Failed to load config.ini: {e}")))?;

    let section = ini
        .section(Some("SMTP"))
        .ok_or_else(|| AppError::new("Chưa cấu hình SMTP trong config.ini. Vui lòng thêm section [SMTP]."))?;

    let host = section
        .get("HOST")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::new("SMTP HOST chưa được cấu hình."))?
        .to_string();

    let port: u16 = section
        .get("PORT")
        .unwrap_or("587")
        .parse()
        .unwrap_or(587);

    let username = section
        .get("USERNAME")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::new("SMTP USERNAME chưa được cấu hình."))?
        .to_string();

    let password = section
        .get("PASSWORD")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::new("SMTP PASSWORD chưa được cấu hình."))?
        .to_string();

    let from_name = section
        .get("FROM_NAME")
        .unwrap_or("Manager System Helps")
        .to_string();

    let from_email = section
        .get("FROM_EMAIL")
        .filter(|s| !s.is_empty())
        .unwrap_or(&username)
        .to_string();

    Ok(SmtpConfig {
        host,
        port,
        username,
        password,
        from_name,
        from_email,
    })
}

pub async fn send_reset_code(to_email: &str, to_name: &str, code: &str) -> AppResult<()> {
    let cfg = load_smtp_config()?;

    let from = format!("{} <{}>", cfg.from_name, cfg.from_email);
    let to = if to_name.is_empty() {
        to_email.to_string()
    } else {
        format!("{to_name} <{to_email}>")
    };

    let html_body = format!(
        r#"<div style="font-family:Arial,sans-serif;max-width:480px;margin:0 auto;padding:24px">
  <h2 style="color:#2563eb;margin:0 0 16px">Đặt lại mật khẩu</h2>
  <p>Mã xác nhận của bạn là:</p>
  <div style="background:#f1f5f9;border-radius:8px;padding:16px;text-align:center;margin:16px 0">
    <span style="font-size:32px;font-weight:bold;letter-spacing:8px;color:#1e293b">{code}</span>
  </div>
  <p style="color:#64748b;font-size:14px">Mã này có hiệu lực trong <strong>30 phút</strong>. Nếu bạn không yêu cầu đặt lại mật khẩu, vui lòng bỏ qua email này.</p>
  <hr style="border:none;border-top:1px solid #e2e8f0;margin:24px 0">
  <p style="color:#94a3b8;font-size:12px;margin:0">Manager System Helps</p>
</div>"#
    );

    let email = Message::builder()
        .from(from.parse().map_err(|e| AppError::new(format!("Invalid FROM address: {e}")))?)
        .to(to.parse().map_err(|e| AppError::new(format!("Invalid TO address: {e}")))?)
        .subject("Mã xác nhận đặt lại mật khẩu")
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .map_err(|e| AppError::new(format!("Failed to build email: {e}")))?;

    let creds = Credentials::new(cfg.username, cfg.password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host)
        .map_err(|e| AppError::new(format!("Failed to create SMTP transport: {e}")))?
        .port(cfg.port)
        .credentials(creds)
        .build();

    mailer
        .send(email)
        .await
        .map_err(|e| AppError::new(format!("Gửi email thất bại: {e}")))?;

    Ok(())
}
