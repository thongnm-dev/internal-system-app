//! API công khai cho module AI Usage — CRUD account + settings dùng chung mọi provider,
//! gọi cho frontend qua [`crate::commands::ai_usage_commands`].
//!
//! Đây KHÔNG phải "data engine": store/lock (`with_data`/`read_data`), `to_public`,
//! `recompute_active`/`recompute_all` vẫn nằm ở
//! [`crate::services::ai_usage_service`] (cũng giữ vòng lặp poll nền) — file này chỉ gọi
//! lại các hàm `pub(crate)` đó. Nghiệp vụ riêng theo từng provider (dò/import/capture login
//! local) nằm ở [`crate::services::claude_service`].

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::ai_account_store::{AiAccountData, StoredAccount};
use crate::database::ai_profile_store;
use crate::models::ai_usage::{
    AddAiAccountRequest, AiAccount, AiUsageSettings, ReportUsageSignalRequest, UpdateAiAccountRequest,
};
use crate::services::ai_usage_service::{read_data, recompute_active, recompute_all, to_public, with_data};
use crate::utils::time::current_timestamp;

/// Detect loại tài khoản từ prefix của API key.
fn detect_account_type(api_key: &str) -> String {
    if api_key.starts_with("sk-ant-admin") {
        "admin".to_string()
    } else if api_key.starts_with("sk-ant-oat") {
        "oauth".to_string()
    } else if api_key.starts_with("sk-ant-") {
        "api".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Xác định loại account: ưu tiên `explicit` (vd `subscription`) do frontend chỉ định,
/// nếu không có key → coi là `subscription`, còn lại detect từ prefix key.
fn resolve_account_type(explicit: Option<&str>, api_key: &str) -> String {
    if let Some(kind) = explicit.map(str::trim).filter(|s| !s.is_empty()) {
        return kind.to_string();
    }
    if api_key.is_empty() {
        return "subscription".to_string();
    }
    detect_account_type(api_key)
}

/// Chuẩn hoá provider: chỉ chấp nhận `claude` | `codex`, mặc định `claude`.
fn normalize_provider(provider: Option<String>) -> String {
    match provider.as_deref().map(str::trim) {
        Some("codex") => "codex".to_string(),
        _ => "claude".to_string(),
    }
}

fn find_mut(data: &mut AiAccountData, id: i64) -> AppResult<&mut StoredAccount> {
    data.accounts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| AppError::new("Account not found."))
}

/// Nạp trước dữ liệu vào cache (gọi lúc app khởi động).
pub fn preload() {
    read_data(|_| ());
}

/// Thêm account AI mới. Trả về account vừa thêm (đã che key).
pub fn add_account(request: AddAiAccountRequest) -> AppResult<AiAccount> {
    let name = request.name.trim().to_string();
    let api_key = request.api_key.unwrap_or_default().trim().to_string();
    let config_dir = request.config_dir.unwrap_or_default().trim().to_string();
    let email = request.email.unwrap_or_default().trim().to_string();
    let subscription_type = request.subscription_type.unwrap_or_default().trim().to_string();
    let source = {
        let s = request.source.unwrap_or_default().trim().to_string();
        if s.is_empty() { "manual".to_string() } else { s }
    };

    if name.is_empty() {
        return Err(AppError::new("Account name is required."));
    }

    let provider = normalize_provider(request.provider);
    let account_type = resolve_account_type(request.account_type.as_deref(), &api_key);
    let is_subscription = account_type == "subscription";

    if is_subscription {
        if config_dir.is_empty() {
            return Err(AppError::new(
                "Config directory is required for subscription accounts.",
            ));
        }
    } else if api_key.is_empty() {
        return Err(AppError::new("API key is required."));
    }

    with_data(|data| {
        if data.accounts.iter().any(|a| a.name == name) {
            return Err(AppError::new(format!("Account \"{name}\" already exists.")));
        }
        if !email.is_empty()
            && data
                .accounts
                .iter()
                .any(|a| a.email.eq_ignore_ascii_case(&email))
        {
            return Err(AppError::new(format!(
                "Account with email \"{email}\" already exists."
            )));
        }
        if !config_dir.is_empty() && data.accounts.iter().any(|a| a.config_dir == config_dir) {
            return Err(AppError::new(format!(
                "Account with config dir \"{config_dir}\" already exists."
            )));
        }

        // Priority mặc định: xuống cuối danh sách của provider.
        let priority = request.priority.unwrap_or_else(|| {
            data.accounts
                .iter()
                .filter(|a| a.provider == provider)
                .map(|a| a.priority)
                .max()
                .map(|max| max + 1)
                .unwrap_or(1)
        });

        let id = data.next_id;
        data.next_id += 1;

        let account = StoredAccount {
            id,
            name,
            api_key,
            account_type,
            provider: provider.clone(),
            config_dir,
            email,
            subscription_type,
            source,
            priority,
            is_active: false,
            status: "unknown".to_string(),
            // Subscription không probe được → nguồn usage là tín hiệu/thủ công.
            usage_source: if is_subscription {
                "manual".to_string()
            } else {
                "unknown".to_string()
            },
            usage_percent: 100.0,
            reset_at: String::new(),
            session_percent: 100.0,
            session_reset_at: String::new(),
            weekly_percent: 100.0,
            weekly_reset_at: String::new(),
            session_count: 0,
            last_checked_at: String::new(),
            created_at: current_timestamp(),
        };

        data.accounts.push(account);
        recompute_active(data, &provider);

        let public = data
            .accounts
            .iter()
            .find(|a| a.id == id)
            .map(to_public)
            .unwrap();
        Ok(public)
    })
}

/// Danh sách account, nhóm theo provider rồi tăng dần theo priority.
pub fn list_accounts() -> AppResult<Vec<AiAccount>> {
    Ok(read_data(|data| {
        // Ưu tiên detected: ẩn account `captured` nếu đã có account khác (không phải
        // captured) cùng email — tránh hiển thị trùng thông tin sau khi Refresh.
        let authoritative_emails: Vec<String> = data
            .accounts
            .iter()
            .filter(|a| a.source != "captured" && !a.email.is_empty())
            .map(|a| a.email.to_ascii_lowercase())
            .collect();

        let mut list: Vec<&StoredAccount> = data
            .accounts
            .iter()
            .filter(|a| {
                !(a.source == "captured"
                    && !a.email.is_empty()
                    && authoritative_emails.contains(&a.email.to_ascii_lowercase()))
            })
            .collect();
        list.sort_by(|a, b| {
            a.provider
                .cmp(&b.provider)
                .then(a.priority.cmp(&b.priority))
                .then(a.id.cmp(&b.id))
        });
        list.into_iter().map(to_public).collect()
    }))
}

/// Cập nhật name/provider/priority của một account.
pub fn update_account(request: UpdateAiAccountRequest) -> AppResult<AiAccount> {
    with_data(|data| {
        // Kiểm tra trùng tên (nếu đổi tên).
        if let Some(new_name) = request.name.as_ref().map(|s| s.trim().to_string()) {
            if !new_name.is_empty()
                && data
                    .accounts
                    .iter()
                    .any(|a| a.id != request.id && a.name == new_name)
            {
                return Err(AppError::new(format!(
                    "Account \"{new_name}\" already exists."
                )));
            }
        }

        {
            let provider = normalize_provider(request.provider.clone());
            let has_provider = request.provider.is_some();
            let account = find_mut(data, request.id)?;
            if let Some(name) = request.name.as_ref().map(|s| s.trim().to_string()) {
                if !name.is_empty() {
                    account.name = name;
                }
            }
            if let Some(priority) = request.priority {
                account.priority = priority;
            }
            if has_provider {
                account.provider = provider;
            }
            if let Some(config_dir) = request.config_dir.as_ref().map(|s| s.trim().to_string()) {
                account.config_dir = config_dir;
            }
        }

        recompute_all(data);

        let public = data
            .accounts
            .iter()
            .find(|a| a.id == request.id)
            .map(to_public)
            .unwrap();
        Ok(public)
    })
}

/// Xóa một account theo id.
pub fn delete_account(id: i64) -> AppResult<()> {
    with_data(|data| {
        let before = data.accounts.len();
        data.accounts.retain(|a| a.id != id);
        if data.accounts.len() == before {
            return Err(AppError::new("Account not found."));
        }
        recompute_all(data);
        Ok(())
    })?;
    // Dọn profile token đã capture (nếu có).
    ai_profile_store::delete(id);
    Ok(())
}

/// Đánh dấu một account làm active (override thủ công) trong provider của nó.
pub fn set_active(id: i64) -> AppResult<()> {
    with_data(|data| {
        let provider = find_mut(data, id)?.provider.clone();
        for account in data.accounts.iter_mut().filter(|a| a.provider == provider) {
            account.is_active = account.id == id;
        }
        Ok(())
    })
}

/// Lấy token gốc của một account (dùng để copy trong app desktop).
pub fn get_token(id: i64) -> AppResult<String> {
    read_data(|data| {
        data.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.api_key.clone())
            .ok_or_else(|| AppError::new("Account not found."))
    })
}

/// Nhận tín hiệu usage từ skill/automation (ví dụ dính "usage limit reached").
pub fn report_signal(request: ReportUsageSignalRequest) -> AppResult<()> {
    with_data(|data| {
        let provider = {
            let account = find_mut(data, request.id)?;
            if request.exhausted {
                account.status = "exhausted".to_string();
                account.usage_percent = 0.0;
            } else {
                account.status = "healthy".to_string();
            }
            account.usage_source = "error_signal".to_string();
            if let Some(reset_at) = request.reset_at.clone() {
                account.reset_at = reset_at;
            }
            account.last_checked_at = current_timestamp();
            account.provider.clone()
        };
        recompute_active(data, &provider);
        Ok(())
    })
}

/// Lấy cấu hình auto-switch / poll.
pub fn get_settings() -> AppResult<AiUsageSettings> {
    Ok(read_data(|data| data.settings.clone()))
}

/// Lưu cấu hình auto-switch / poll.
pub fn save_settings(settings: AiUsageSettings) -> AppResult<()> {
    with_data(|data| {
        data.settings = settings;
        Ok(())
    })
}
