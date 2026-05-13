import { invoke } from "@tauri-apps/api/core";
import { tauriRuntimeMessage } from "../config/appConfig";

type TauriWindow = Window & {
  __TAURI_INTERNALS__?: {
    invoke?: unknown;
  };
};

export function canUseTauriRuntime(): boolean {
  return typeof window !== "undefined" && typeof (window as TauriWindow).__TAURI_INTERNALS__?.invoke === "function";
}

export async function safeInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!canUseTauriRuntime()) {
    throw new Error(tauriRuntimeMessage);
  }

  return invoke<T>(command, args);
}

export function friendlyError(error: unknown): string {
  const text = String(error);
  if (text.includes("__TAURI_INTERNALS__") || text.includes("reading 'invoke'")) {
    return tauriRuntimeMessage;
  }

  return text;
}
