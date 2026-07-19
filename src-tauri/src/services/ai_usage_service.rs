//! Service cho module AI Usage.
//!
//! Quản lý danh sách account AI (lưu file cục bộ qua [`ai_account_store`]),
//! thứ tự ưu tiên, đánh dấu account "active" theo provider, và auto-switch khi
//! account đang dùng hết usage. Số liệu usage được cập nhật bởi poll nền
//! (xem [`run_poll_loop`]) thông qua [`ai_usage_probe`].

use std::sync::Mutex;

use tauri::{AppHandle, Emitter};

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::ai_account_store::{self, AiAccountData, StoredAccount};
use crate::models::ai_usage::{
    AddAiAccountRequest, AiAccount, AiUsageSettings, DetectedLogin, ReportUsageSignalRequest,
    UpdateAiAccountRequest,
};
use crate::services::ai_usage_detect;
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
        *guard = Some(data);
    }
}

/// Thao tác ghi: chạy `f` trên dữ liệu rồi lưu xuống file.
fn with_data<T>(f: impl FnOnce(&mut AiAccountData) -> AppResult<T>) -> AppResult<T> {
    let mut guard = STATE.lock().unwrap();
    ensure_loaded(&mut guard);
    let data = guard.as_mut().unwrap();
    let result = f(data)?;
    ai_account_store::save(data)?;
    Ok(result)
}

/// Thao tác chỉ đọc: không ghi file.
fn read_data<T>(f: impl FnOnce(&AiAccountData) -> T) -> T {
    let mut guard = STATE.lock().unwrap();
    ensure_loaded(&mut guard);
    f(guard.as_ref().unwrap())
}

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

fn to_public(account: &StoredAccount) -> AiAccount {
    AiAccount {
        id: account.id,
        name: account.name.clone(),
        api_key_masked: mask_api_key(&account.api_key),
        account_type: account.account_type.clone(),
        provider: account.provider.clone(),
        config_dir: account.config_dir.clone(),
        email: account.email.clone(),
        subscription_type: account.subscription_type.clone(),
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
fn recompute_active(data: &mut AiAccountData, provider: &str) {
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
fn recompute_all(data: &mut AiAccountData) {
    for provider in providers(data) {
        recompute_active(data, &provider);
    }
}

fn find_mut(data: &mut AiAccountData, id: i64) -> AppResult<&mut StoredAccount> {
    data.accounts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| AppError::new("Account not found."))
}

// ─────────────────────────────── API công khai ───────────────────────────────

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
                .unwrap_or(0)
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

/// Dò các login Claude đã tồn tại trên máy (đọc `.claude.json` + Keychain).
pub fn detect_local() -> AppResult<Vec<DetectedLogin>> {
    let accounts = read_data(|data| data.accounts.clone());
    Ok(ai_usage_detect::scan(&accounts))
}

/// Dò login local rồi tự thêm những login chưa có (dạng account subscription).
/// Trả về danh sách account sau khi thêm. Lỗi từng account (vd trùng) được bỏ qua.
pub fn import_detected() -> AppResult<Vec<AiAccount>> {
    for login in detect_local()? {
        if login.already_added {
            continue;
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
            priority: None,
        };
        let _ = add_account(request);
    }
    list_accounts()
}

/// Danh sách account, nhóm theo provider rồi tăng dần theo priority.
pub fn list_accounts() -> AppResult<Vec<AiAccount>> {
    Ok(read_data(|data| {
        let mut list: Vec<&StoredAccount> = data.accounts.iter().collect();
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
    })
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

// ─────────────────────────────── Poll nền ───────────────────────────────

/// Bản sao danh sách account để probe bất đồng bộ (không giữ lock qua `.await`).
fn snapshot() -> Vec<StoredAccount> {
    read_data(|data| data.accounts.clone())
}

/// Áp dụng kết quả probe vào dữ liệu + auto-switch, rồi lưu file.
fn apply_probe_results(outcomes: Vec<ai_usage_probe::ProbeOutcome>) -> AppResult<()> {
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
        let interval = get_settings()
            .map(|s| s.poll_interval_secs)
            .unwrap_or(60)
            .max(15);
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        if let Err(e) = poll_once(&app).await {
            eprintln!("AI usage poll error: {e}");
        }
    }
}
