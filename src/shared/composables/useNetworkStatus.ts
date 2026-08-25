import { readonly, ref } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { checkInternetConnection } from "@/tauri/commands/system";

// Interval between background reachability probes while the app is running.
const POLL_INTERVAL_MS = 15000;
// Collapses bursts of native online/offline events (common during flaky
// WiFi/VPN reconnects) into a single reaction instead of racing.
const BROWSER_EVENT_DEBOUNCE_MS = 400;

// Module-level singleton state so the startup screen, the app shell and the
// offline banner all observe the same connectivity status.
const isOnline = ref(true);
const isChecking = ref(false);
// Becomes true the first time we confirm connectivity. Used to distinguish
// "never connected since launch" (full-screen error) from "lost connection
// mid-session" (banner while keeping the current screen).
const hasConnectedOnce = ref(false);

const AUTO_CHECK_KEY = "msh.network.autoCheck";

function loadAutoCheck(): boolean {
  try {
    const saved = window.localStorage.getItem(AUTO_CHECK_KEY);
    return saved === null ? true : saved === "true";
  } catch {
    return true;
  }
}

// User-facing toggle (settings UI) for whether connectivity is watched in the
// background (poll timer + browser online/offline listeners). Disabling it
// does not affect manual/one-off checks via retry().
const autoCheckEnabled = ref(loadAutoCheck());

let pollTimer: number | undefined;
let started = false;
let autoCheckAttached = false;
// Shared promise for whatever probe is currently in flight. Any caller that
// invokes check() while one is already running awaits this same result
// instead of firing a second concurrent probe — otherwise overlapping probes
// can resolve out of order and the slower-but-stale one overwrites isOnline
// with an outdated result, making the banner flicker.
let inFlightCheck: Promise<boolean> | null = null;
let onlineDebounceTimer: number | undefined;
let offlineDebounceTimer: number | undefined;

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

function check(): Promise<boolean> {
  if (inFlightCheck) {
    return inFlightCheck;
  }

  isChecking.value = true;
  inFlightCheck = (async () => {
    try {
      const online = await runProbe();
      isOnline.value = online;
      if (online) {
        hasConnectedOnce.value = true;
      }
      return online;
    } finally {
      isChecking.value = false;
      inFlightCheck = null;
    }
  })();

  return inFlightCheck;
}

function handleBrowserOffline() {
  // The OS/interface reports the link is down. Debounced: flaky WiFi/VPN
  // reconnects can fire online/offline in quick bursts, so wait for the
  // signal to settle before reacting.
  if (onlineDebounceTimer !== undefined) {
    window.clearTimeout(onlineDebounceTimer);
    onlineDebounceTimer = undefined;
  }
  if (offlineDebounceTimer !== undefined) {
    window.clearTimeout(offlineDebounceTimer);
  }
  offlineDebounceTimer = window.setTimeout(() => {
    isOnline.value = false;
  }, BROWSER_EVENT_DEBOUNCE_MS);
}

function handleBrowserOnline() {
  // The link is back, but confirm real reachability before clearing the
  // error. Debounced for the same reason as handleBrowserOffline.
  if (offlineDebounceTimer !== undefined) {
    window.clearTimeout(offlineDebounceTimer);
    offlineDebounceTimer = undefined;
  }
  if (onlineDebounceTimer !== undefined) {
    window.clearTimeout(onlineDebounceTimer);
  }
  onlineDebounceTimer = window.setTimeout(() => {
    void check();
  }, BROWSER_EVENT_DEBOUNCE_MS);
}

function attachAutoCheck() {
  if (autoCheckAttached) return;
  autoCheckAttached = true;
  window.addEventListener("online", handleBrowserOnline);
  window.addEventListener("offline", handleBrowserOffline);
  pollTimer = window.setInterval(() => void check(), POLL_INTERVAL_MS);
}

function detachAutoCheck() {
  if (!autoCheckAttached) return;
  autoCheckAttached = false;
  window.removeEventListener("online", handleBrowserOnline);
  window.removeEventListener("offline", handleBrowserOffline);
  if (pollTimer !== undefined) {
    window.clearInterval(pollTimer);
    pollTimer = undefined;
  }
}

/**
 * Enables/disables background connectivity watching (poll timer + browser
 * online/offline listeners) and persists the choice. Manual checks via
 * retry() keep working either way.
 */
function setAutoCheck(enabled: boolean): void {
  autoCheckEnabled.value = enabled;
  try {
    window.localStorage.setItem(AUTO_CHECK_KEY, String(enabled));
  } catch {
    // Private browsing / storage quota — the in-memory setting still applies
    // for the rest of the session.
  }

  if (!started) return;

  if (enabled) {
    attachAutoCheck();
    void check();
  } else {
    detachAutoCheck();
  }
}

/**
 * Starts connectivity monitoring exactly once: an initial probe, plus (when
 * auto-check is enabled) browser online/offline listeners for instant
 * reaction and periodic polling to catch silent drops. Returns the promise of
 * the initial check.
 */
function start(): Promise<boolean> {
  if (started) {
    return Promise.resolve(isOnline.value);
  }
  started = true;

  if (autoCheckEnabled.value) {
    attachAutoCheck();
  }

  return check();
}

export function useNetworkStatus() {
  return {
    isOnline: readonly(isOnline),
    isChecking: readonly(isChecking),
    hasConnectedOnce: readonly(hasConnectedOnce),
    autoCheckEnabled: readonly(autoCheckEnabled),
    start,
    retry: check,
    setAutoCheck,
  };
}
