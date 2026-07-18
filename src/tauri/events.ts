import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { canUseTauriRuntime } from "./commands/_base";

/** Event backend bắn khi số liệu usage / active account thay đổi. */
export const AI_USAGE_UPDATED_EVENT = "ai-usage-updated";

/**
 * Lắng nghe event `ai-usage-updated` từ backend (poll nền).
 * Trả về hàm huỷ đăng ký; no-op nếu không chạy trong Tauri runtime.
 */
export async function onAiUsageUpdated(handler: () => void): Promise<UnlistenFn> {
  if (!canUseTauriRuntime()) {
    return () => {};
  }
  return listen(AI_USAGE_UPDATED_EVENT, () => handler());
}
