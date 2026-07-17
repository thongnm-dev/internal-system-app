/** Nhà cung cấp AI được hỗ trợ. */
export type AiChatProvider = "gemini" | "groq";

/** Một tin nhắn trong hội thoại gửi lên backend. */
export type AiChatMessage = {
  role: "user" | "assistant";
  content: string;
};

/** Request hoàn thành hội thoại. */
export type AiChatRequest = {
  provider: AiChatProvider;
  model: string;
  messages: AiChatMessage[];
  system?: string;
  api_key?: string;
  max_tokens?: number;
};

/** Phản hồi từ backend. */
export type AiChatResponse = {
  provider: string;
  model: string;
  content: string;
};
