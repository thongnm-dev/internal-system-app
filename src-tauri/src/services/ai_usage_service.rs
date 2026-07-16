//! Service cho module AI Usage — lưu tạm danh sách account AI trong bộ nhớ.
//!
//! Danh sách chỉ tồn tại trong vòng đời của process; sẽ thay bằng
//! persistence thực (database) khi backend hoàn thiện.
//! Các số liệu usage (phần trăm còn lại, thời gian reset, số session)
//! hiện là giá trị khởi tạo mặc định — sẽ cập nhật từ API thực sau.

use std::sync::Mutex;

use chrono::{Duration, Local};

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::ai_usage::{AddAiAccountRequest, AiAccount};
use crate::utils::time::current_timestamp;

/// Khoảng thời gian mặc định đến lần reset usage tiếp theo (giờ).
const DEFAULT_RESET_HOURS: i64 = 5;

/// Detect loại tài khoản từ prefix của API key.
///
/// - `sk-ant-admin...` → Admin key (Anthropic Admin API)
/// - `sk-ant-oat...`   → OAuth token (subscription Claude Pro/Max)
/// - `sk-ant-...`      → Anthropic API key
/// - còn lại           → không xác định
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

/// Account đầy đủ (giữ API key gốc) — chỉ dùng nội bộ service, không trả ra frontend.
struct StoredAccount {
    id: i64,
    name: String,
    api_key: String,
    account_type: String,
    usage_percent: f64,
    reset_at: String,
    session_count: i32,
    created_at: String,
}

/// Danh sách account lưu tạm trong bộ nhớ.
static ACCOUNTS: Mutex<Vec<StoredAccount>> = Mutex::new(Vec::new());
/// Bộ đếm id tự tăng.
static NEXT_ID: Mutex<i64> = Mutex::new(1);

/// Che API key, chỉ giữ 4 ký tự cuối (ví dụ `••••abcd`).
fn mask_api_key(key: &str) -> String {
    let tail: String = key.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("••••{tail}")
}

fn to_public(account: &StoredAccount) -> AiAccount {
    AiAccount {
        id: account.id,
        name: account.name.clone(),
        api_key_masked: mask_api_key(&account.api_key),
        account_type: account.account_type.clone(),
        usage_percent: account.usage_percent,
        reset_at: account.reset_at.clone(),
        session_count: account.session_count,
        created_at: account.created_at.clone(),
    }
}

/// Thêm account AI mới vào danh sách tạm. Trả về account vừa thêm (đã che key).
pub fn add_account(request: AddAiAccountRequest) -> AppResult<AiAccount> {
    let name = request.name.trim().to_string();
    let api_key = request.api_key.trim().to_string();

    if name.is_empty() {
        return Err(AppError::new("Account name is required."));
    }
    if api_key.is_empty() {
        return Err(AppError::new("API key is required."));
    }
    let account_type = detect_account_type(&api_key);

    let mut accounts = ACCOUNTS.lock().unwrap();
    if accounts.iter().any(|a| a.name == name) {
        return Err(AppError::new(format!("Account \"{name}\" already exists.")));
    }

    let reset_at = (Local::now() + Duration::hours(DEFAULT_RESET_HOURS))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let mut next_id = NEXT_ID.lock().unwrap();
    let account = StoredAccount {
        id: *next_id,
        name,
        api_key,
        account_type,
        usage_percent: 100.0,
        reset_at,
        session_count: 0,
        created_at: current_timestamp(),
    };
    *next_id += 1;

    let public = to_public(&account);
    accounts.push(account);
    Ok(public)
}

/// Lấy toàn bộ danh sách account (đã che API key), mới nhất trước.
pub fn list_accounts() -> AppResult<Vec<AiAccount>> {
    let accounts = ACCOUNTS.lock().unwrap();
    let mut list: Vec<AiAccount> = accounts.iter().map(to_public).collect();
    list.reverse();
    Ok(list)
}

/// Xóa một account theo id.
pub fn delete_account(id: i64) -> AppResult<()> {
    let mut accounts = ACCOUNTS.lock().unwrap();
    let before = accounts.len();
    accounts.retain(|a| a.id != id);
    if accounts.len() == before {
        return Err(AppError::new("Account not found."));
    }
    Ok(())
}
