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
}

impl ProbeOutcome {
    fn simple(id: i64, status: &str, usage_source: &str) -> Self {
        Self {
            id,
            status: status.to_string(),
            usage_percent: None,
            reset_at: None,
            usage_source: usage_source.to_string(),
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

    match account.provider.as_str() {
        "codex" => probe_openai(&client, account).await,
        // Mặc định coi là Claude.
        _ => probe_anthropic(&client, account).await,
    }
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
