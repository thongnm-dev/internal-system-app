//! Probe usage riêng cho provider Claude — tách khỏi
//! [`crate::services::ai_usage_probe`] (dispatcher dùng chung mọi provider).
//!
//! - Subscription (Claude Pro/Max, `account_type = "subscription"`): gọi endpoint
//!   OAuth usage chính thức (giống lệnh `/usage` của Claude Code) để lấy session (5h)
//!   + weekly (7 ngày).
//! - API key (`account_type = "api"`/`"admin"`): gọi `GET /v1/models` (không tốn token)
//!   để xác thực key và đọc header `anthropic-ratelimit-*` nếu có.
//!
//! Provider khác (vd Codex) có probe riêng ngay trong `ai_usage_probe` — khi tách file
//! riêng cho Codex thì đặt tên tương tự (`codex_probe.rs`).

use reqwest::{Client, StatusCode};

use crate::database::ai_account_store::StoredAccount;
use crate::database::ai_profile_store;
use crate::models::ai_usage::ProbeOutcome;
use crate::services::ai_usage_probe::{header_f64, header_string, status_from_percent};
use crate::services::claude_detected;

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
pub(crate) async fn probe_subscription(client: &Client, account: &StoredAccount) -> ProbeOutcome {
    // Token nguồn: account `captured` đọc từ profile (app data dir); còn lại từ Keychain.
    let token = if account.source == "captured" {
        ai_profile_store::read_access_token(account.id)
    } else {
        claude_detected::read_oauth_token(&account.config_dir)
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

    // % tổng hợp = giới hạn nào còn ít hơn — và `reset_at`/`usage_window` phải đi theo
    // ĐÚNG giới hạn đó, chứ không mặc định lấy session_reset_at (bug cũ: countdown hiển
    // thị luôn theo session dù % hiển thị đang bị kéo xuống bởi weekly).
    let (usage_percent, usage_reset_at, usage_window): (Option<f64>, Option<String>, Option<String>) =
        match (session_percent, weekly_percent) {
            (Some(s), Some(w)) if w < s => (Some(w), weekly_reset_at.clone(), Some("weekly".to_string())),
            (Some(s), Some(_)) => (Some(s), session_reset_at.clone(), Some("session".to_string())),
            (Some(s), None) => (Some(s), session_reset_at.clone(), Some("session".to_string())),
            (None, Some(w)) => (Some(w), weekly_reset_at.clone(), Some("weekly".to_string())),
            (None, None) => (None, None, None),
        };

    log::debug!(
        "[probe] id={} email={} — session={:?}% weekly={:?}% usage={:?}% window={:?}",
        account.id, account.email, session_percent, weekly_percent, usage_percent, usage_window,
    );

    ProbeOutcome {
        id: account.id,
        status: usage_percent
            .map(status_from_percent)
            .unwrap_or_else(|| "unknown".to_string()),
        usage_percent,
        reset_at: usage_reset_at,
        usage_source: "billing_api".to_string(),
        session_percent,
        session_reset_at,
        weekly_percent,
        weekly_reset_at,
        usage_window,
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
        usage_window: None,
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

/// Probe account Anthropic (Claude) qua API key. Subscription OAuth thì bỏ qua network
/// (xem [`probe_subscription`]).
pub(crate) async fn probe(client: &Client, account: &StoredAccount) -> ProbeOutcome {
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
            usage_window: None,
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
                usage_window: None,
            }
        }
        // Key hợp lệ nhưng không có header usage → coi là healthy, không rõ % còn lại.
        _ => ProbeOutcome::simple(account.id, "healthy", "unknown"),
    }
}
