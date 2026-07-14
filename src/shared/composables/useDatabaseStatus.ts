import { readonly, ref } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { checkDatabaseStatus, saveDatabaseConfig } from "@/tauri/commands/database-config";
import type { SaveDatabaseConfigRequest } from "@/_/types/database-config";

// Module-level singleton state so the startup flow, the config screen and the
// app shell all observe the same database status.
const isConfigured = ref(false);
const isConnected = ref(false);
const isChecking = ref(false);
const hasChecked = ref(false);
const statusMessage = ref("");
// True when the user asks to edit the config from the connection-error screen,
// so the config form is shown even though a (broken) config already exists.
const wantsReconfigure = ref(false);

/**
 * Runs the database configuration/connectivity check. In browser dev (no Tauri
 * runtime) the database is assumed ready so the Vite dev server keeps working.
 */
async function check(): Promise<boolean> {
  if (!canUseTauriRuntime()) {
    isConfigured.value = true;
    isConnected.value = true;
    hasChecked.value = true;
    return true;
  }

  isChecking.value = true;
  try {
    const status = await checkDatabaseStatus();
    isConfigured.value = status.configured;
    isConnected.value = status.connected;
    statusMessage.value = status.message;
    return status.connected;
  } catch {
    // Treat an unexpected failure as "needs configuration" so the user gets the
    // config screen rather than a hard crash.
    isConfigured.value = false;
    isConnected.value = false;
    return false;
  } finally {
    hasChecked.value = true;
    isChecking.value = false;
  }
}

type DatabaseStatusResult = {
  configured: boolean;
  connected: boolean;
  message: string;
};

/** Apply a status returned by the backend to the shared reactive state. */
function applyStatus(status: DatabaseStatusResult): void {
  isConfigured.value = status.configured;
  isConnected.value = status.connected;
  statusMessage.value = status.message;
  hasChecked.value = true;
  if (status.connected) {
    wantsReconfigure.value = false;
  }
}

/** From the connection-error screen: open the config form to fix the settings. */
function requestReconfigure(): void {
  wantsReconfigure.value = true;
}

/** From the config form: return to the connection-error screen. */
function cancelReconfigure(): void {
  wantsReconfigure.value = false;
}

/**
 * Persists a new database configuration through the backend and returns the
 * resulting status WITHOUT applying it. The caller decides when to apply it
 * (via `applyStatus`) — e.g. after the user acknowledges a confirm dialog — so
 * the app does not swap away before the confirmation is shown.
 */
async function save(request: SaveDatabaseConfigRequest): Promise<DatabaseStatusResult> {
  return await saveDatabaseConfig(request);
}

export function useDatabaseStatus() {
  return {
    isConfigured: readonly(isConfigured),
    isConnected: readonly(isConnected),
    isChecking: readonly(isChecking),
    hasChecked: readonly(hasChecked),
    statusMessage: readonly(statusMessage),
    wantsReconfigure: readonly(wantsReconfigure),
    applyStatus,
    cancelReconfigure,
    check,
    requestReconfigure,
    save,
  };
}
