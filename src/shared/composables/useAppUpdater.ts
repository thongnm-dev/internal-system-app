import { computed, ref, shallowRef } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { canUseTauriRuntime } from "@/tauri/commands/_base";

const CHECK_INTERVAL_MS = 30 * 60 * 1000;

export type UpdaterStatus =
  | "idle"
  | "checking"
  | "downloading"
  | "ready"
  | "installing"
  | "error";

// ── Singleton state — shared across all callers, survives component remounts ──

const status = ref<UpdaterStatus>("idle");
const version = ref<string | null>(null);
const errorMessage = ref<string | null>(null);
const downloadedBytes = ref(0);
const totalBytes = ref(0);
const pending = shallowRef<Update | null>(null);

let intervalId: ReturnType<typeof setInterval> | null = null;
let pollingStarted = false;

const downloadPercent = computed(() => {
  if (totalBytes.value <= 0) return null;
  const pct = Math.round((downloadedBytes.value / totalBytes.value) * 100);
  return Math.min(100, Math.max(0, pct));
});

const isActive = computed(
  () =>
    status.value === "checking" ||
    status.value === "downloading" ||
    status.value === "ready" ||
    status.value === "installing",
);

async function downloadUpdate(update: Update): Promise<void> {
  status.value = "downloading";
  downloadedBytes.value = 0;
  totalBytes.value = 0;

  await update.download((event) => {
    switch (event.event) {
      case "Started":
        totalBytes.value = event.data.contentLength ?? 0;
        downloadedBytes.value = 0;
        break;
      case "Progress":
        downloadedBytes.value += event.data.chunkLength;
        break;
      case "Finished":
        if (totalBytes.value > 0) {
          downloadedBytes.value = totalBytes.value;
        }
        break;
    }
  });

  pending.value = update;
  status.value = "ready";
}

async function checkAndDownload(): Promise<void> {
  if (!canUseTauriRuntime()) return;
  if (isActive.value) return;

  status.value = "checking";
  errorMessage.value = null;

  try {
    const update = await check();
    if (!update) {
      status.value = "idle";
      version.value = null;
      return;
    }
    version.value = update.version;
    await downloadUpdate(update);
  } catch (error) {
    errorMessage.value = String(error);
    status.value = "error";
  }
}

async function checkNow(): Promise<void> {
  if (!canUseTauriRuntime()) return;
  if (isActive.value) return;

  pending.value = null;
  errorMessage.value = null;
  status.value = "idle";

  await checkAndDownload();
}

async function install(): Promise<void> {
  const update = pending.value;
  if (!update || status.value !== "ready") return;

  status.value = "installing";
  try {
    await update.install();
    await relaunch();
  } catch (error) {
    errorMessage.value = String(error);
    status.value = "error";
  }
}

function startPolling(): void {
  if (pollingStarted) return;
  pollingStarted = true;

  void checkAndDownload();
  intervalId = setInterval(() => {
    if (status.value !== "ready" && status.value !== "installing") {
      void checkAndDownload();
    }
  }, CHECK_INTERVAL_MS);
}

export function useAppUpdater() {
  return {
    status,
    version,
    errorMessage,
    downloadPercent,
    downloadedBytes,
    totalBytes,
    isActive,
    isTauri: canUseTauriRuntime(),
    checkAndDownload,
    checkNow,
    startPolling,
    install,
  };
}
