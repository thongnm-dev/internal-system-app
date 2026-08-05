//! Nghiệp vụ AI Usage riêng cho provider Claude: dò/import login local (`.claude.json` +
//! Keychain — xem [`claude_detected`]), capture login đang active, đăng ký thêm
//! `CLAUDE_CONFIG_DIR` thứ 2 (xem [`claude_capture`]).
//!
//! Đây là service theo từng provider — các provider khác (vd Codex) trong tương lai sẽ có
//! file riêng tương tự (vd `codex_service.rs`) thay vì gom chung vào một file. CRUD account
//! dùng chung (add/list/delete…) vẫn ở [`crate::services::ai_acc_service`]; store/lock
//! (`with_data`/`read_data`/`to_public`/`recompute_active`) ở [`crate::services::ai_usage_service`]
//! — service ở đây chỉ gọi lại các hàm `pub(crate)` đó.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::ai_account_store::StoredAccount;
use crate::database::ai_profile_store;
use crate::models::ai_usage::{AddAiAccountRequest, AiAccount, CapturedLogin, DetectedLogin};
use crate::services::ai_acc_service;
use crate::services::ai_usage_service;
use crate::services::{claude_capture, claude_detected};
use crate::utils::time::current_timestamp;

/// Dò các login Claude đã tồn tại trên máy (đọc `.claude.json` + Keychain).
pub fn detect_local() -> AppResult<Vec<DetectedLogin>> {
    let accounts = ai_usage_service::read_data(|data| data.accounts.clone());
    Ok(claude_detected::scan(&accounts))
}

/// Dò login local rồi tự thêm những login chưa có (dạng account subscription).
/// Trả về danh sách account sau khi thêm. Lỗi từng account (vd trùng) được bỏ qua.
pub fn import_detected() -> AppResult<Vec<AiAccount>> {
    for login in detect_local()? {
        // Ưu tiên bản detected: nếu email đã có ở account khác nguồn (captured/manual)
        // thì thay bằng bản detected; nếu đã là detected thì giữ nguyên, bỏ qua.
        let existing = ai_usage_service::read_data(|data| {
            data.accounts
                .iter()
                .find(|a| !a.email.is_empty() && a.email.eq_ignore_ascii_case(&login.email))
                .map(|a| (a.id, a.source.clone()))
        });
        match existing {
            Some((_, source)) if source == "detected" => continue,
            Some((id, _)) => {
                let _ = ai_acc_service::delete_account(id);
            }
            None => {}
        }

        let name = if login.display_name.trim().is_empty() {
            login.email.clone()
        } else {
            login.display_name.clone()
        };
        let request = AddAiAccountRequest {
            name,
            api_key: None,
            provider: Some("claude".to_string()),
            account_type: Some("subscription".to_string()),
            config_dir: Some(login.config_dir.clone()),
            email: Some(login.email.clone()),
            subscription_type: Some(login.subscription_type.clone()),
            source: Some("detected".to_string()),
            priority: None,
        };
        let _ = ai_acc_service::add_account(request);
    }
    ai_acc_service::list_accounts()
}

/// Xem trước login Claude đang active trên máy (để capture) — không kèm token.
pub fn capture_preview() -> AppResult<Option<CapturedLogin>> {
    Ok(claude_capture::preview())
}

/// Capture login Claude đang active: lưu OAuth token vào profile (app data dir) rồi
/// thêm account subscription dạng `captured`. Ưu tiên bản detected nếu đã có cùng email.
pub fn capture_add(name: Option<String>) -> AppResult<AiAccount> {
    let captured = claude_capture::capture_current().ok_or_else(|| {
        AppError::new("Không tìm thấy login Claude đang hoạt động. Hãy chạy `claude /login` trước.")
    })?;

    let email = captured.email.trim().to_string();

    if !email.is_empty() {
        // Đã có account cùng email (detected/manual/legacy) → không tạo bản capture
        // trùng, ưu tiên bản kia (thường là detected — token trong Keychain luôn mới).
        let authoritative_exists = ai_usage_service::read_data(|data| {
            data.accounts
                .iter()
                .any(|a| a.source != "captured" && a.email.eq_ignore_ascii_case(&email))
        });
        if authoritative_exists {
            return Err(AppError::new(
                "Đã có account cùng email trên máy (ưu tiên bản detect). Không tạo bản capture trùng.",
            ));
        }
        // Đã capture trước đó (cùng email) → xoá bản cũ để capture token mới.
        let old = ai_usage_service::read_data(|data| {
            data.accounts
                .iter()
                .find(|a| a.source == "captured" && a.email.eq_ignore_ascii_case(&email))
                .map(|a| a.id)
        });
        if let Some(id) = old {
            let _ = ai_acc_service::delete_account(id);
        }
    }

    let account_name = name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| {
            if captured.display_name.trim().is_empty() {
                email.clone()
            } else {
                captured.display_name.clone()
            }
        });
    let subscription_type = captured.subscription_type.clone();

    // Tạo account (id do store cấp) — config_dir = thư mục profile theo id.
    let id = ai_usage_service::with_data(|data| {
        if data.accounts.iter().any(|a| a.name == account_name) {
            return Err(AppError::new(format!(
                "Account \"{account_name}\" already exists."
            )));
        }
        let priority = data
            .accounts
            .iter()
            .filter(|a| a.provider == "claude")
            .map(|a| a.priority)
            .max()
            .map(|max| max + 1)
            .unwrap_or(1);

        let id = data.next_id;
        data.next_id += 1;
        let config_dir = ai_profile_store::profile_dir(id).to_string_lossy().to_string();

        data.accounts.push(StoredAccount {
            id,
            name: account_name.clone(),
            api_key: String::new(),
            account_type: "subscription".to_string(),
            provider: "claude".to_string(),
            config_dir,
            email: email.clone(),
            subscription_type,
            source: "captured".to_string(),
            priority,
            is_active: false,
            status: "unknown".to_string(),
            usage_source: "manual".to_string(),
            usage_percent: 100.0,
            reset_at: String::new(),
            session_percent: 100.0,
            session_reset_at: String::new(),
            weekly_percent: 100.0,
            weekly_reset_at: String::new(),
            usage_window: String::new(),
            session_count: 0,
            last_checked_at: String::new(),
            created_at: current_timestamp(),
        });
        ai_usage_service::recompute_active(data, "claude");
        Ok(id)
    })?;

    // Ghi token capture ra profile (ngoài lock ghi file JSON chính).
    ai_profile_store::save_credentials(id, &captured.claude_ai_oauth)?;

    ai_usage_service::read_data(|data| {
        data.accounts.iter().find(|a| a.id == id).map(ai_usage_service::to_public)
    })
    .ok_or_else(|| AppError::new("Account not found after capture."))
}

/// Xem trước login Claude tại một `CLAUDE_CONFIG_DIR` (để thêm account thứ 2).
pub fn config_dir_preview(config_dir: String) -> AppResult<Option<CapturedLogin>> {
    Ok(claude_capture::read_login_at(config_dir.trim()))
}

/// Đăng ký account subscription thứ 2 từ một `CLAUDE_CONFIG_DIR` đã login sẵn
/// (token nằm trong Keychain của dir đó → account `detected`, luôn mới).
pub fn add_config_dir(config_dir: String, name: Option<String>) -> AppResult<AiAccount> {
    let dir = config_dir.trim().to_string();
    if dir.is_empty() {
        return Err(AppError::new("Config directory is required."));
    }
    let login = claude_capture::read_login_at(&dir).ok_or_else(|| {
        AppError::new(
            "Không tìm thấy login Claude trong thư mục này. Chạy `CLAUDE_CONFIG_DIR=<dir> claude /login` trước.",
        )
    })?;

    let account_name = name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| {
            if login.display_name.trim().is_empty() {
                login.email.clone()
            } else {
                login.display_name.clone()
            }
        });

    ai_acc_service::add_account(AddAiAccountRequest {
        name: account_name,
        api_key: None,
        provider: Some("claude".to_string()),
        account_type: Some("subscription".to_string()),
        config_dir: Some(dir),
        email: Some(login.email),
        subscription_type: Some(login.subscription_type),
        source: Some("detected".to_string()),
        priority: None,
    })
}
