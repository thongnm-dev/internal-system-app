//! Probe tình trạng usage cho từng account AI — dispatcher dùng chung mọi provider.
//!
//! Không có API "phần trăm còn lại" chính thức cho mọi loại account, nên module
//! này áp dụng chiến lược theo `account_type`/`provider` và **degrade gracefully**:
//!
//! - `claude` + `api`/`admin`, hoặc subscription (OAuth usage) → xem [`claude_probe`].
//! - `codex` (OpenAI) → xem [`codex_probe`].
//!
//! Mọi lỗi mạng/timeout → `status = "error"`, không ghi đè `usage_percent` cũ.

use std::time::Duration;

use reqwest::Client;

use crate::database::ai_account_store::StoredAccount;
use crate::models::ai_usage::ProbeOutcome;
use crate::services::claude_probe;
use crate::services::codex_probe;

/// Timeout cho mỗi lần probe (giây).
const PROBE_TIMEOUT_SECS: u64 = 8;

/// Probe một account, trả về kết quả (không panic).
pub async fn probe(account: &StoredAccount) -> ProbeOutcome {
    let client = match Client::builder()
        .timeout(Duration::from_secs(PROBE_TIMEOUT_SECS))
        .build()
    {
        Ok(client) => client,
        Err(_) => return ProbeOutcome::simple(account.id, "error", "unknown"),
    };

    // Subscription (Claude Pro/Max): dùng OAuth token trong Keychain gọi endpoint
    // usage chính thức (giống lệnh `/usage` của Claude Code) để lấy session + weekly.
    if account.account_type == "subscription" {
        return claude_probe::probe_subscription(&client, account).await;
    }

    match account.provider.as_str() {
        "codex" => codex_probe::probe(&client, account).await,
        // Mặc định coi là Claude.
        _ => claude_probe::probe(&client, account).await,
    }
}

/// Ánh xạ phần trăm còn lại → trạng thái thô (chưa xét ngưỡng cấu hình).
/// `pub(crate)` — dùng chung bởi probe riêng theo provider (vd [`claude_probe`]).
pub(crate) fn status_from_percent(percent: f64) -> String {
    if percent <= 0.0 {
        "exhausted".to_string()
    } else if percent <= 15.0 {
        "low".to_string()
    } else {
        "healthy".to_string()
    }
}

/// Đọc một header và parse thành `f64`. `pub(crate)` — xem [`status_from_percent`].
pub(crate) fn header_f64(response: &reqwest::Response, name: &str) -> Option<f64> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|text| text.trim().parse::<f64>().ok())
}

/// Đọc một header dạng chuỗi. `pub(crate)` — xem [`status_from_percent`].
pub(crate) fn header_string(response: &reqwest::Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}
