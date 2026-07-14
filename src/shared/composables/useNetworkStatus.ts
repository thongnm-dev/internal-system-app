import { readonly, ref } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { checkInternetConnection } from "@/tauri/commands/system";

// Interval between background reachability probes while the app is running.
const POLL_INTERVAL_MS = 15000;

// Module-level singleton state so the startup screen, the app shell and the
// offline banner all observe the same connectivity status.
const isOnline = ref(true);
const isChecking = ref(false);
// Becomes true the first time we confirm connectivity. Used to distinguish
// "never connected since launch" (full-screen error) from "lost connection
// mid-session" (banner while keeping the current screen).
const hasConnectedOnce = ref(false);

let pollTimer: number | undefined;
let started = false;

/**
 * Performs a single reachability probe. Uses the Rust backend when available
 * (real internet check), otherwise falls back to the browser's own signal so
 * the Vite dev server keeps working.
 */
async function runProbe(): Promise<boolean> {
  if (canUseTauriRuntime()) {
    try {
      return await checkInternetConnection();
    } catch {
      return navigator.onLine;
    }
  }
  return navigator.onLine;
}

async function check(): Promise<boolean> {
  isChecking.value = true;
  try {
    const online = await runProbe();
    isOnline.value = online;
    if (online) {
      hasConnectedOnce.value = true;
    }
    return online;
  } finally {
    isChecking.value = false;
  }
}

function handleBrowserOffline() {
  // The OS/interface reports the link is down — reflect it immediately.
  isOnline.value = false;
}

function handleBrowserOnline() {
  // The link is back, but confirm real reachability before clearing the error.
  void check();
}

/**
 * Starts connectivity monitoring exactly once: an initial probe, browser
 * online/offline listeners for instant reaction, and periodic polling to catch
 * silent drops. Returns the promise of the initial check.
 */
function start(): Promise<boolean> {
  if (started) {
    return Promise.resolve(isOnline.value);
  }
  started = true;

  window.addEventListener("online", handleBrowserOnline);
  window.addEventListener("offline", handleBrowserOffline);

  pollTimer = window.setInterval(() => void check(), POLL_INTERVAL_MS);

  return check();
}

export function useNetworkStatus() {
  return {
    isOnline: readonly(isOnline),
    isChecking: readonly(isChecking),
    hasConnectedOnce: readonly(hasConnectedOnce),
    start,
    retry: check,
  };
}
