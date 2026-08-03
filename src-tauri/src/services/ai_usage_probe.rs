//! Probe tình trạng usage cho từng account AI.
//!
//! Không có API "phần trăm còn lại" chính thức cho mọi loại account, nên module
//! này áp dụng chiến lược theo `account_type`/`provider` và **degrade gracefully**:
//!
//! - `claude` + `api`/`admin` → gọi `GET /v1/models` (không tốn token) để xác thực
//!   key và đọc header `anthropic-ratelimit-*` nếu có → `usage_source = ratelimit_header`.
//! - `codex` (OpenAI) → gọi `GET /v1/models` và đọc header `x-ratelimit-*`.
//! - `claude` + `oauth` (subscription) → không có API usage chính thức, không probe
//!   qua mạng (tránh đụng luồng auth subscription) → giữ số liệu cũ, `unknown`.
//!
//! Mọi lỗi mạng/timeout → `status = "error"`, không ghi đè `usage_percent` cũ.

use std::time::Duration;

use reqwest::{Client, StatusCode};

use crate::database::ai_account_store::StoredAccount;
use crate::database::ai_profile_store;
use crate::services::ai_usage_detect;

/// Timeout cho mỗi lần probe (giây).
const PROBE_TIMEOUT_SECS: u64 = 8;

/// Kết quả probe cho một account. `usage_percent`/`reset_at` là `None` nghĩa là
/// "không có số liệu mới" → service giữ nguyên giá trị cũ.
pub struct ProbeOutcome {
    pub id: i64,
    pub status: String,
    pub usage_percent: Option<f64>,
    pub reset_at: Option<String>,
    pub usage_source: String,
    /// Session hiện tại (5h) — phần trăm CÒN LẠI. `None` = giữ giá trị cũ.
    pub session_percent: Option<f64>,
    pub session_reset_at: Option<String>,
    /// Weekly limit (7 ngày) — phần trăm CÒN LẠI. `None` = giữ giá trị cũ.
    pub weekly_percent: Option<f64>,
    pub weekly_reset_at: Option<String>,
}

impl ProbeOutcome {
    fn simple(id: i64, status: &str, usage_source: &str) -> Self {
        Self {
            id,
            status: status.to_string(),
            usage_percent: None,
            reset_at: None,
            usage_source: usage_source.to_string(),
            session_percent: None,
            session_reset_at: None,
            weekly_percent: None,
            weekly_reset_at: None,
        }
    }
}

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
        return probe_subscription(&client, account).await;
    }

    match account.provider.as_str() {
        "codex" => probe_openai(&client, account).await,
        // Mặc định coi là Claude.
        _ => probe_anthropic(&client, account).await,
    }
}

// ─────────────────────────── Subscription (OAuth usage) ───────────────────────────

/// URL endpoint usage của Claude (dùng bởi lệnh `/usage`).
const CLAUDE_OAUTH_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

/// Một cửa sổ giới hạn (`five_hour` / `seven_day`) trong response usage.
#[derive(Debug, Clone, serde::Deserialize)]
struct UsageWindow {
    /// Phần trăm ĐÃ DÙNG (0–100).
    #[serde(default)]
    utilization: f64,
    /// Thời điểm reset (RFC3339), rỗng nếu không có.
    #[serde(default)]
    resets_at: Option<String>,
}

/// Response từ `/api/oauth/usage`. Trước đây có `five_hour`/`seven_day` ở top-level;
/// từ khi Anthropic thêm mode (standard / extended_thinking / …) các window có thể nằm
/// dưới key mode, ví dụ `{ "standard": { "five_hour": … } }`. Parse cả hai layout.
#[derive(Debug, serde::Deserialize)]
struct OauthUsageResponse {
    five_hour: Option<UsageWindow>,
    seven_day: Option<UsageWindow>,
}

/// Nhóm window cho một chế độ (standard, extended_thinking, …).
#[derive(Debug, serde::Deserialize)]
struct ModeWindows {
    five_hour: Option<UsageWindow>,
    seven_day: Option<UsageWindow>,
}

/// Trích xuất cặp (five_hour, seven_day) từ JSON response, hỗ trợ cả layout cũ
/// (flat) lẫn layout mới (mode-based). Với layout mới, lấy window có utilization
/// CAO NHẤT giữa các mode (phản ánh giới hạn nào bị ép chặt nhất).
fn extract_usage_windows(body: &str) -> (Option<UsageWindow>, Option<UsageWindow>) {
    let val: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };

    // 1) Thử layout cũ (top-level five_hour / seven_day).
    if let Ok(flat) = serde_json::from_value::<OauthUsageResponse>(val.clone()) {
        if flat.five_hour.is_some() || flat.seven_day.is_some() {
            return (flat.five_hour, flat.seven_day);
        }
    }

    // 2) Layout mode-based: duyệt mọi key ở top-level là object chứa five_hour/seven_day.
    //    Lấy window có utilization cao nhất (= sát giới hạn nhất).
    let obj = match val.as_object() {
        Some(o) => o,
        None => return (None, None),
    };

    let mut best_five: Option<UsageWindow> = None;
    let mut best_seven: Option<UsageWindow> = None;

    for (_mode_key, mode_val) in obj {
        if let Ok(mw) = serde_json::from_value::<ModeWindows>(mode_val.clone()) {
            if let Some(w) = mw.five_hour {
                best_five = Some(match best_five {
                    Some(prev) if prev.utilization >= w.utilization => prev,
                    _ => w,
                });
            }
            if let Some(w) = mw.seven_day {
                best_seven = Some(match best_seven {
                    Some(prev) if prev.utilization >= w.utilization => prev,
                    _ => w,
                });
            }
        }
    }

    (best_five, best_seven)
}

/// Probe account subscription qua OAuth usage endpoint.
///
/// Đọc access token từ Keychain (theo `config_dir`), gọi endpoint usage và ánh xạ:
/// - `five_hour` → session hiện tại (5h);
/// - `seven_day` → weekly limit (7 ngày).
///
/// `usage_percent` tổng hợp = min(session, weekly) còn lại — để status/auto-switch
/// phản ánh giới hạn nào sắp chạm trước. Không đọc được token / lỗi mạng → giữ số
/// liệu cũ (`unknown`/`manual`), không phá vỡ trạng thái hiện có.
async fn probe_subscription(client: &Client, account: &StoredAccount) -> ProbeOutcome {
    // Token nguồn: account `captured` đọc từ profile (app data dir); còn lại từ Keychain.
    let token = if account.source == "captured" {
        ai_profile_store::read_access_token(account.id)
    } else {
        ai_usage_detect::read_oauth_token(&account.config_dir)
    };
    let Some(token) = token else {
        log::debug!(
            "[probe] id={} email={} — no token found (config_dir={})",
            account.id, account.email, account.config_dir,
        );
        return ProbeOutcome::simple(account.id, "unknown", "manual");
    };

    let response = client
        .get(CLAUDE_OAUTH_USAGE_URL)
        .bearer_auth(&token)
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await;

    let response = match response {
        Ok(resp) => resp,
        Err(e) => {
            log::debug!("[probe] id={} email={} — network error: {e}", account.id, account.email);
            return keep_previous(account);
        }
    };

    let status = response.status();
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        log::debug!("[probe] id={} email={} — auth error {status}", account.id, account.email);
        return ProbeOutcome::simple(account.id, "error", "manual");
    }
    if !status.is_success() {
        log::debug!("[probe] id={} email={} — HTTP {status}, keeping previous", account.id, account.email);
        return keep_previous(account);
    }

    let body = match response.text().await {
        Ok(text) => text,
        Err(_) => return keep_previous(account),
    };
    log::debug!(
        "[probe] id={} email={} — raw response: {}",
        account.id, account.email,
        if body.len() > 2000 { &body[..2000] } else { &body },
    );

    let (five_hour, seven_day) = extract_usage_windows(&body);

    let (session_percent, session_reset_at) = match five_hour.map(window_remaining) {
        Some((p, r)) => (Some(p), r),
        None => (None, None),
    };
    let (weekly_percent, weekly_reset_at) = match seven_day.map(window_remaining) {
        Some((p, r)) => (Some(p), r),
        None => (None, None),
    };

    // % tổng hợp = giới hạn nào còn ít hơn.
    let combined = [session_percent, weekly_percent]
        .into_iter()
        .flatten()
        .fold(None, |acc: Option<f64>, p| Some(acc.map_or(p, |a| a.min(p))));

    log::debug!(
        "[probe] id={} email={} — session={:?}% weekly={:?}% combined={:?}%",
        account.id, account.email, session_percent, weekly_percent, combined,
    );

    ProbeOutcome {
        id: account.id,
        status: combined
            .map(status_from_percent)
            .unwrap_or_else(|| "unknown".to_string()),
        usage_percent: combined,
        reset_at: session_reset_at.clone(),
        usage_source: "billing_api".to_string(),
        session_percent,
        session_reset_at,
        weekly_percent,
        weekly_reset_at,
    }
}

/// Giữ nguyên số liệu usage cũ của account (dùng khi 429/lỗi tạm thời) — tất cả
/// trường `None` nên tầng service không ghi đè, chỉ cập nhật `last_checked_at`.
fn keep_previous(account: &StoredAccount) -> ProbeOutcome {
    ProbeOutcome {
        id: account.id,
        status: account.status.clone(),
        usage_percent: None,
        reset_at: None,
        usage_source: account.usage_source.clone(),
        session_percent: None,
        session_reset_at: None,
        weekly_percent: None,
        weekly_reset_at: None,
    }
}

/// Từ một cửa sổ usage → (phần trăm còn lại, thời điểm reset dạng local).
fn window_remaining(window: UsageWindow) -> (f64, Option<String>) {
    let remaining = (100.0 - window.utilization).clamp(0.0, 100.0);
    let reset_at = window
        .resets_at
        .as_deref()
        .and_then(format_rfc3339_local)
        .filter(|s| !s.is_empty());
    (remaining, reset_at)
}

/// Chuyển timestamp RFC3339 → `YYYY-MM-DD HH:MM:SS` theo timezone local.
fn format_rfc3339_local(raw: &str) -> Option<String> {
    chrono::DateTime::parse_from_rfc3339(raw.trim())
        .ok()
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
}

/// Probe account Anthropic (Claude). Subscription OAuth thì bỏ qua network.
async fn probe_anthropic(client: &Client, account: &StoredAccount) -> ProbeOutcome {
    if account.account_type == "oauth" {
        // Subscription: không có endpoint usage chính thức, dựa error_signal/manual.
        return ProbeOutcome::simple(account.id, "unknown", "manual");
    }

    let response = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", &account.api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await;

    let response = match response {
        Ok(resp) => resp,
        Err(_) => return ProbeOutcome::simple(account.id, "error", "unknown"),
    };

    let status = response.status();
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return ProbeOutcome::simple(account.id, "error", "unknown");
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        let reset_at = header_string(&response, "anthropic-ratelimit-requests-reset");
        return ProbeOutcome {
            id: account.id,
            status: "exhausted".to_string(),
            usage_percent: Some(0.0),
            reset_at,
            usage_source: "ratelimit_header".to_string(),
            session_percent: None,
            session_reset_at: None,
            weekly_percent: None,
            weekly_reset_at: None,
        };
    }

    // 2xx: thử đọc rate-limit header (nếu Anthropic đính kèm).
    let limit = header_f64(&response, "anthropic-ratelimit-requests-limit");
    let remaining = header_f64(&response, "anthropic-ratelimit-requests-remaining");
    let reset_at = header_string(&response, "anthropic-ratelimit-requests-reset");

    match (limit, remaining) {
        (Some(limit), Some(remaining)) if limit > 0.0 => {
            let percent = ((remaining / limit) * 100.0).clamp(0.0, 100.0);
            ProbeOutcome {
                id: account.id,
                status: status_from_percent(percent),
                usage_percent: Some(percent),
                reset_at,
                usage_source: "ratelimit_header".to_string(),
                session_percent: None,
                session_reset_at: None,
                weekly_percent: None,
                weekly_reset_at: None,
            }
        }
        // Key hợp lệ nhưng không có header usage → coi là healthy, không rõ % còn lại.
        _ => ProbeOutcome::simple(account.id, "healthy", "unknown"),
    }
}

/// Probe account OpenAI (Codex) qua header `x-ratelimit-*`.
async fn probe_openai(client: &Client, account: &StoredAccount) -> ProbeOutcome {
    let response = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(&account.api_key)
        .send()
        .await;

    let response = match response {
        Ok(resp) => resp,
        Err(_) => return ProbeOutcome::simple(account.id, "error", "unknown"),
    };

    let status = response.status();
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return ProbeOutcome::simple(account.id, "error", "unknown");
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        let reset_at = header_string(&response, "x-ratelimit-reset-requests");
        return ProbeOutcome {
            id: account.id,
            status: "exhausted".to_string(),
            usage_percent: Some(0.0),
            reset_at,
            usage_source: "ratelimit_header".to_string(),
            session_percent: None,
            session_reset_at: None,
            weekly_percent: None,
            weekly_reset_at: None,
        };
    }

    let limit = header_f64(&response, "x-ratelimit-limit-requests");
    let remaining = header_f64(&response, "x-ratelimit-remaining-requests");
    let reset_at = header_string(&response, "x-ratelimit-reset-requests");

    match (limit, remaining) {
        (Some(limit), Some(remaining)) if limit > 0.0 => {
            let percent = ((remaining / limit) * 100.0).clamp(0.0, 100.0);
            ProbeOutcome {
                id: account.id,
                status: status_from_percent(percent),
                usage_percent: Some(percent),
                reset_at,
                usage_source: "ratelimit_header".to_string(),
                session_percent: None,
                session_reset_at: None,
                weekly_percent: None,
                weekly_reset_at: None,
            }
        }
        _ => ProbeOutcome::simple(account.id, "healthy", "unknown"),
    }
}

/// Ánh xạ phần trăm còn lại → trạng thái thô (chưa xét ngưỡng cấu hình).
fn status_from_percent(percent: f64) -> String {
    if percent <= 0.0 {
        "exhausted".to_string()
    } else if percent <= 15.0 {
        "low".to_string()
    } else {
        "healthy".to_string()
    }
}

/// Đọc một header và parse thành `f64`.
fn header_f64(response: &reqwest::Response, name: &str) -> Option<f64> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|text| text.trim().parse::<f64>().ok())
}

/// Đọc một header dạng chuỗi.
fn header_string(response: &reqwest::Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}
