//! Model cho module AI Chat (gọi API các nhà cung cấp LLM).

use serde::{Deserialize, Serialize};

/// Một tin nhắn trong hội thoại. `role` là `user` hoặc `assistant`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiChatMessage {
    pub role: String,
    pub content: String,
}

/// Request gửi hội thoại lên một nhà cung cấp AI.
#[derive(Debug, Deserialize)]
pub struct AiChatRequest {
    /// Nhà cung cấp: `gemini` | `groq`.
    pub provider: String,
    /// Model cụ thể của nhà cung cấp (ví dụ `claude-opus-4-8`).
    pub model: String,
    /// Lịch sử hội thoại theo thứ tự thời gian.
    pub messages: Vec<AiChatMessage>,
    /// System prompt (tùy chọn).
    #[serde(default)]
    pub system: Option<String>,
    /// API key ghi đè (tùy chọn) — nếu rỗng sẽ lấy từ DB hoặc config.ini.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Giới hạn token đầu ra (tùy chọn).
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

/// Phản hồi trả về frontend.
#[derive(Debug, Serialize)]
pub struct AiChatResponse {
    pub provider: String,
    pub model: String,
    pub content: String,
}
