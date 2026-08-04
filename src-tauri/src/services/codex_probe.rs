//! Probe usage riêng cho provider Codex (OpenAI) — tách khỏi
//! [`crate::services::ai_usage_probe`] (dispatcher dùng chung mọi provider).
//!
//! Không có API "phần trăm còn lại" chính thức, nên probe qua header
//! `x-ratelimit-*` khi gọi `GET /v1/models` (không tốn token). Xem
//! [`crate::services::claude_probe`] cho phía Claude.

use reqwest::{Client, StatusCode};

use crate::database::ai_account_store::StoredAccount;
use crate::services::ai_usage_probe::{header_f64, header_string, status_from_percent, ProbeOutcome};

/// Probe account OpenAI (Codex) qua header `x-ratelimit-*`.
pub(crate) async fn probe(client: &Client, account: &StoredAccount) -> ProbeOutcome {
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
