import { safeInvoke } from "./_base";
import type { AiChatRequest, AiChatResponse } from "@/_/types/ai-chat";

/** Gửi hội thoại lên nhà cung cấp AI và nhận phản hồi. */
export function aiChatComplete(request: AiChatRequest) {
  return safeInvoke<AiChatResponse>("ai_chat_complete", { request });
}
