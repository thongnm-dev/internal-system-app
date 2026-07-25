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

/** Event backend bắn mỗi khi một phiên terminal có chunk output mới. */
export const TERMINAL_OUTPUT_EVENT = "terminal-output";

/** Payload của event `terminal-output`. `data` là chuỗi base64 của byte thô từ PTY. */
export interface TerminalOutputPayload {
  id: string;
  data: string;
}

/**
 * Lắng nghe output từ tất cả phiên terminal. Handler tự lọc theo `id` phiên.
 * Trả về hàm huỷ đăng ký; no-op nếu không chạy trong Tauri runtime.
 */
export async function onTerminalOutput(
  handler: (payload: TerminalOutputPayload) => void,
): Promise<UnlistenFn> {
  if (!canUseTauriRuntime()) {
    return () => {};
  }
  return listen<TerminalOutputPayload>(TERMINAL_OUTPUT_EVENT, (event) => handler(event.payload));
}

/** Event backend bắn khi tiến trình shell của một phiên terminal kết thúc. */
export const TERMINAL_EXIT_EVENT = "terminal-exit";

/** Payload của event `terminal-exit`. */
export interface TerminalExitPayload {
  id: string;
  code: number | null;
}

/**
 * Lắng nghe sự kiện kết thúc của các phiên terminal.
 * Trả về hàm huỷ đăng ký; no-op nếu không chạy trong Tauri runtime.
 */
export async function onTerminalExit(
  handler: (payload: TerminalExitPayload) => void,
): Promise<UnlistenFn> {
  if (!canUseTauriRuntime()) {
    return () => {};
  }
  return listen<TerminalExitPayload>(TERMINAL_EXIT_EVENT, (event) => handler(event.payload));
}

/** Event backend bắn khi poll nền phát hiện tài liệu S3 mới. */
export const S3_NEW_DOCUMENTS_EVENT = "s3-new-documents";

/** Payload của event `s3-new-documents`. */
export interface S3NewDocumentsPayload {
  total: number;
  storages: { name: string; items: string[] }[];
}

/**
 * Lắng nghe event `s3-new-documents` từ backend (poll nền toàn app).
 * Trả về hàm huỷ đăng ký; no-op nếu không chạy trong Tauri runtime.
 */
export async function onS3NewDocuments(
  handler: (payload: S3NewDocumentsPayload) => void,
): Promise<UnlistenFn> {
  if (!canUseTauriRuntime()) {
    return () => {};
  }
  return listen<S3NewDocumentsPayload>(S3_NEW_DOCUMENTS_EVENT, (event) => handler(event.payload));
}
