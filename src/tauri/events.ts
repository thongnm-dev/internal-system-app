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

/** Event backend bắn khi file watcher phát hiện thay đổi trong working tree của repo Git đang theo dõi. */
export const GIT_REPO_CHANGED_EVENT = "git-repo-changed";

/**
 * Lắng nghe event `git-repo-changed` (file watcher nền cho repo Git đang mở).
 * Payload là đường dẫn repo đã thay đổi. Trả về hàm huỷ đăng ký; no-op nếu
 * không chạy trong Tauri runtime.
 */
export async function onGitRepoChanged(handler: (path: string) => void): Promise<UnlistenFn> {
  if (!canUseTauriRuntime()) {
    return () => {};
  }
  return listen<string>(GIT_REPO_CHANGED_EVENT, (event) => handler(event.payload));
}
