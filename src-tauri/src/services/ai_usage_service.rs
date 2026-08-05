//! Service cho module AI Usage — lõi lưu trữ (`STATE`/`with_data`/`read_data`) dùng chung
//! cho mọi provider, cùng vòng lặp poll nền cập nhật số liệu usage.
//!
//! - CRUD account + settings (API công khai cho frontend) nằm ở
//!   [`crate::services::ai_acc_service`], gọi lại `with_data`/`read_data`/`to_public`/
//!   `recompute_active`/`recompute_all` ở đây thay vì tự quản lý file/lock riêng.
//! - Nghiệp vụ riêng theo từng provider (dò/import/capture login local) nằm ở
//!   [`crate::services::claude_service`] cho Claude — provider khác (vd Codex) sẽ có
//!   service riêng tương tự trong tương lai.
//! - Poll nền (xem [`run_poll_loop`]) cập nhật số liệu qua [`ai_usage_probe`].

use std::sync::Mutex;

use tauri::{AppHandle, Emitter};

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::ai_account_store::{self, AiAccountData, StoredAccount};
use crate::models::ai_usage::{AiAccount, ProbeOutcome};
use crate::services::ai_acc_service;
use crate::services::ai_usage_probe;
use crate::utils::time::current_timestamp;

/// Event bắn tới frontend khi số liệu usage / active account thay đổi.
const USAGE_UPDATED_EVENT: &str = "ai-usage-updated";

/// Cache dữ liệu trong bộ nhớ, đồng bộ với file. `None` = chưa load lần nào.
static STATE: Mutex<Option<AiAccountData>> = Mutex::new(None);

// ─────────────────────────────── Helpers nội bộ ───────────────────────────────

/// Nạp dữ liệu vào guard nếu chưa có (từ file, hoặc mặc định khi lỗi/không tồn tại).
fn ensure_loaded(guard: &mut Option<AiAccountData>) {
    if guard.is_none() {
        let mut data = ai_account_store::load().unwrap_or_default();
        if data.next_id < 1 {
            data.next_id = 1;
        }
        // Migrate account cũ (tạo trước khi có field `source`): suy ra nguồn để logic
        // "ưu tiên detected" hoạt động. Có API key → `manual`; còn lại → `detected`.
        for account in data.accounts.iter_mut() {
            if account.source.trim().is_empty() {
                account.source = if account.api_key.is_empty() {
                    "detected".to_string()
                } else {
                    "manual".to_string()
                };
            }
        }
        for account in data.accounts.iter_mut() {
            if account.priority < 1 {
                account.priority = 1;
            }
        }
        // Migrate chu kỳ poll cũ (60s) → 300s: endpoint usage bị rate-limit nếu gọi dày.
        if data.settings.poll_interval_secs == 60 {
            data.settings.poll_interval_secs = 300;
        }
        *guard = Some(data);
    }
}

/// Thao tác ghi: chạy `f` trên dữ liệu rồi lưu xuống file.
///
/// `pub(crate)` để các service riêng theo provider (vd [`crate::services::claude_service`])
/// tái sử dụng cùng "data engine" thay vì tự quản lý file/lock riêng.
pub(crate) fn with_data<T>(f: impl FnOnce(&mut AiAccountData) -> AppResult<T>) -> AppResult<T> {
    let mut guard = STATE.lock().unwrap();
    ensure_loaded(&mut guard);
    let data = guard.as_mut().unwrap();
    let result = f(data)?;
    ai_account_store::save(data)?;
    Ok(result)
}

/// Thao tác chỉ đọc: không ghi file. `pub(crate)` — xem [`with_data`].
pub(crate) fn read_data<T>(f: impl FnOnce(&AiAccountData) -> T) -> T {
    let mut guard = STATE.lock().unwrap();
    ensure_loaded(&mut guard);
    f(guard.as_ref().unwrap())
}

/// Che API key, chỉ giữ 4 ký tự cuối (ví dụ `••••abcd`). Rỗng (subscription) → `—`.
fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return "—".to_string();
    }
    let tail: String = key
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("••••{tail}")
}

/// `pub(crate)` — dùng chung bởi các service riêng theo provider (vd [`crate::services::claude_service`]).
pub(crate) fn to_public(account: &StoredAccount) -> AiAccount {
    AiAccount {
        id: account.id,
        name: account.name.clone(),
        api_key_masked: mask_api_key(&account.api_key),
        account_type: account.account_type.clone(),
        provider: account.provider.clone(),
        config_dir: account.config_dir.clone(),
        email: account.email.clone(),
        subscription_type: account.subscription_type.clone(),
        source: account.source.clone(),
        priority: account.priority,
        is_active: account.is_active,
        status: account.status.clone(),
        usage_source: account.usage_source.clone(),
        usage_percent: account.usage_percent,
        reset_at: account.reset_at.clone(),
        session_percent: account.session_percent,
        session_reset_at: account.session_reset_at.clone(),
        weekly_percent: account.weekly_percent,
        weekly_reset_at: account.weekly_reset_at.clone(),
        usage_window: account.usage_window.clone(),
        session_count: account.session_count,
        last_checked_at: account.last_checked_at.clone(),
        created_at: account.created_at.clone(),
    }
}

/// Account có đủ điều kiện được chọn làm active hay không.
///
/// Chỉ chấp nhận trạng thái `healthy`/`unknown`; loại `low`/`exhausted`/`error`.
/// (Ngưỡng `switch_threshold_percent` đã được gấp vào `status` khi probe.)
fn is_eligible(account: &StoredAccount) -> bool {
    account.status == "healthy" || account.status == "unknown"
}

/// Danh sách provider đang có account.
fn providers(data: &AiAccountData) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    for account in &data.accounts {
        if !list.contains(&account.provider) {
            list.push(account.provider.clone());
        }
    }
    list
}

/// Tính lại active account cho một provider.
///
/// - Nếu active hiện tại vẫn eligible → giữ nguyên (tôn trọng cả auto lẫn override
///   thủ công), chỉ đảm bảo duy nhất một active.
/// - Nếu không → chọn account eligible có priority nhỏ nhất (tie theo id nhỏ nhất).
///
/// `pub(crate)` — dùng chung bởi các service riêng theo provider (vd [`crate::services::claude_service`]).
pub(crate) fn recompute_active(data: &mut AiAccountData, provider: &str) {
    let keep = data
        .accounts
        .iter()
        .filter(|a| a.provider == provider && a.is_active && is_eligible(a))
        .min_by_key(|a| (a.priority, a.id))
        .map(|a| a.id);

    let chosen = keep.or_else(|| {
        data.accounts
            .iter()
            .filter(|a| a.provider == provider && is_eligible(a))
            .min_by_key(|a| (a.priority, a.id))
            .map(|a| a.id)
    });

    for account in data.accounts.iter_mut().filter(|a| a.provider == provider) {
        account.is_active = Some(account.id) == chosen;
    }
}

/// Tính lại active cho toàn bộ provider.
/// `pub(crate)` — dùng chung bởi [`crate::services::ai_acc_service`].
pub(crate) fn recompute_all(data: &mut AiAccountData) {
    for provider in providers(data) {
        recompute_active(data, &provider);
    }
}

// ─────────────────────────────── Poll nền ───────────────────────────────

/// Bản sao danh sách account để probe bất đồng bộ (không giữ lock qua `.await`).
fn snapshot() -> Vec<StoredAccount> {
    read_data(|data| data.accounts.clone())
}

/// Áp dụng kết quả probe vào dữ liệu + auto-switch, rồi lưu file.
fn apply_probe_results(outcomes: Vec<ProbeOutcome>) -> AppResult<()> {
    with_data(|data| {
        let threshold = data.settings.switch_threshold_percent;
        for outcome in &outcomes {
            if let Some(account) = data.accounts.iter_mut().find(|a| a.id == outcome.id) {
                account.last_checked_at = current_timestamp();
                account.usage_source = outcome.usage_source.clone();
                if let Some(reset_at) = outcome.reset_at.clone() {
                    account.reset_at = reset_at;
                }
                if let Some(percent) = outcome.session_percent {
                    account.session_percent = percent;
                }
                if let Some(reset_at) = outcome.session_reset_at.clone() {
                    account.session_reset_at = reset_at;
                }
                if let Some(percent) = outcome.weekly_percent {
                    account.weekly_percent = percent;
                }
                if let Some(reset_at) = outcome.weekly_reset_at.clone() {
                    account.weekly_reset_at = reset_at;
                }
                if let Some(window) = outcome.usage_window.clone() {
                    account.usage_window = window;
                }
                if let Some(percent) = outcome.usage_percent {
                    account.usage_percent = percent;
                    // Gấp ngưỡng cấu hình vào status.
                    account.status = if percent <= 0.0 {
                        "exhausted".to_string()
                    } else if percent <= threshold {
                        "low".to_string()
                    } else {
                        "healthy".to_string()
                    };
                } else {
                    // Không có số liệu mới → dùng status thô từ probe.
                    account.status = outcome.status.clone();
                }
            }
        }
        recompute_all(data);
        Ok(())
    })
}

/// Probe một account theo ID, cập nhật dữ liệu và trả về account mới nhất.
pub async fn poll_account(id: i64) -> AppResult<AiAccount> {
    let account = read_data(|data| data.accounts.iter().find(|a| a.id == id).cloned())
        .ok_or_else(|| AppError::new("Account not found."))?;
    let outcome = ai_usage_probe::probe(&account).await;
    apply_probe_results(vec![outcome])?;
    let updated = read_data(|data| data.accounts.iter().find(|a| a.id == id).cloned())
        .ok_or_else(|| AppError::new("Account not found after probe."))?;
    Ok(to_public(&updated))
}

/// Probe toàn bộ account một lần, cập nhật dữ liệu và bắn event cho frontend.
pub async fn poll_once(app: &AppHandle) -> AppResult<()> {
    let accounts = snapshot();
    if accounts.is_empty() {
        return Ok(());
    }

    let mut outcomes = Vec::with_capacity(accounts.len());
    for account in &accounts {
        outcomes.push(ai_usage_probe::probe(account).await);
    }

    apply_probe_results(outcomes)?;
    let _ = app.emit(USAGE_UPDATED_EVENT, ());
    Ok(())
}

/// Vòng lặp poll nền — chạy suốt vòng đời ứng dụng.
pub async fn run_poll_loop(app: AppHandle) {
    loop {
        // Tối thiểu 60s để tránh bị rate-limit endpoint usage của Claude.
        let interval = ai_acc_service::get_settings()
            .map(|s| s.poll_interval_secs)
            .unwrap_or(300)
            .max(60);
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        if let Err(e) = poll_once(&app).await {
            log::error!("AI usage poll error: {e}");
        }
    }
}

