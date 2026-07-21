//! Business logic cho AI Chat — gọi API của các nhà cung cấp LLM.
//!
//! Rust không có SDK chính thức cho các nhà cung cấp này nên toàn bộ gọi qua
//! HTTP thô bằng `reqwest`. API key được resolve theo thứ tự ưu tiên:
//! 1. Trường `api_key` trong request (ghi đè trực tiếp).
//! 2. Section `[ai]` trong `config.ini`.

use std::time::Duration;

use ini::Ini;
use reqwest::Client;
use serde_json::{json, Value};

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::ai_chat::{AiChatRequest, AiChatResponse};
use crate::utils::app_config;
/// Token đầu ra mặc định nếu request không chỉ định.
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Điểm vào chính: nhận request, gọi đúng nhà cung cấp, trả về nội dung trả lời.
pub async fn complete(request: AiChatRequest) -> AppResult<AiChatResponse> {
    if request.messages.is_empty() {
        return Err(AppError::new("Hội thoại trống — không có tin nhắn để gửi."));
    }

    let provider = request.provider.trim().to_lowercase();
    let api_key = resolve_api_key(&provider, request.api_key.as_deref())?;
    let max_tokens = request.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
    let client = build_client()?;

    let content = match provider.as_str() {
        "gemini" => gemini(&client, &api_key, &request, max_tokens).await?,
        "groq" => {
            openai_compatible(&client, "https://api.groq.com/openai/v1/chat/completions", &api_key, &request, max_tokens).await?
        }
        other => {
            return Err(AppError::new(format!(
                "Nhà cung cấp không được hỗ trợ: '{other}'."
            )))
        }
    };

    Ok(AiChatResponse {
        provider,
        model: request.model.clone(),
        content,
    })
}

fn build_client() -> AppResult<Client> {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::new(format!("Không tạo được HTTP client: {e}")))
}

/// Nhãn key_label trong config.ini tương ứng từng provider.
fn key_label(provider: &str) -> AppResult<&'static str> {
    match provider {
        "gemini" => Ok("GEMINI_API_KEY"),
        "groq" => Ok("GROQ_API_KEY"),
        other => Err(AppError::new(format!(
            "Nhà cung cấp không được hỗ trợ: '{other}'."
        ))),
    }
}

/// Resolve API key theo thứ tự: request override → config.ini.
fn resolve_api_key(provider: &str, override_key: Option<&str>) -> AppResult<String> {
    if let Some(key) = override_key {
        let key = key.trim();
        if !key.is_empty() {
            return Ok(key.to_string());
        }
    }

    let label = key_label(provider)?;

    if let Some(key) = api_key_from_ini(label) {
        let key = key.trim().to_string();
        if !key.is_empty() {
            return Ok(key);
        }
    }

    Err(AppError::new(format!(
        "Chưa cấu hình API key cho {provider}. Thêm '{label}' vào section [ai] trong config.ini hoặc nhập trực tiếp."
    )))
}

fn api_key_from_ini(label: &str) -> Option<String> {
    let path = app_config::config_path();
    let ini = Ini::load_from_file(&path).ok()?;
    let section = ini.section(Some("ai"))?;
    section.get(label).map(|s| s.to_string())
}

/// Trích lỗi có nội dung từ response không thành công.
async fn error_body(provider: &str, status: reqwest::StatusCode, resp: reqwest::Response) -> AppError {
    let body = resp.text().await.unwrap_or_default();
    AppError::new(format!("{provider} API lỗi {}: {body}", status.as_u16()))
}

/// OpenAI / xAI (Grok) — cùng chuẩn `chat/completions`.
async fn openai_compatible(
    client: &Client,
    url: &str,
    api_key: &str,
    request: &AiChatRequest,
    max_tokens: u32,
) -> AppResult<String> {
    let mut messages: Vec<Value> = Vec::new();
    if let Some(system) = request.system.as_deref() {
        if !system.trim().is_empty() {
            messages.push(json!({ "role": "system", "content": system }));
        }
    }
    for m in &request.messages {
        messages.push(json!({ "role": m.role, "content": m.content }));
    }

    let body = json!({
        "model": request.model,
        "messages": messages,
        "max_tokens": max_tokens,
    });

    let resp = client
        .post(url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::new(format!("Gọi API thất bại: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(error_body(&request.provider, status, resp).await);
    }

    let value: Value = resp
        .json()
        .await
        .map_err(|e| AppError::new(format!("Không phân tích được phản hồi: {e}")))?;

    value["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::new("Phản hồi không có nội dung."))
}

/// Google Gemini — endpoint `generateContent`, header `x-goog-api-key`.
async fn gemini(
    client: &Client,
    api_key: &str,
    request: &AiChatRequest,
    max_tokens: u32,
) -> AppResult<String> {
    // Gemini dùng role "user"/"model" (assistant → model).
    let contents: Vec<Value> = request
        .messages
        .iter()
        .map(|m| {
            let role = if m.role == "assistant" { "model" } else { "user" };
            json!({ "role": role, "parts": [{ "text": m.content }] })
        })
        .collect();

    let mut body = json!({
        "contents": contents,
        "generationConfig": { "maxOutputTokens": max_tokens },
    });
    if let Some(system) = request.system.as_deref() {
        if !system.trim().is_empty() {
            body["systemInstruction"] = json!({ "parts": [{ "text": system }] });
        }
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        request.model
    );

    let resp = client
        .post(&url)
        .header("x-goog-api-key", api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::new(format!("Gọi API thất bại: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(error_body("Gemini", status, resp).await);
    }

    let value: Value = resp
        .json()
        .await
        .map_err(|e| AppError::new(format!("Không phân tích được phản hồi: {e}")))?;

    let text = value["candidates"][0]["content"]["parts"]
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter_map(|p| p["text"].as_str())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    if text.is_empty() {
        return Err(AppError::new("Phản hồi không có nội dung."));
    }
    Ok(text)
}
