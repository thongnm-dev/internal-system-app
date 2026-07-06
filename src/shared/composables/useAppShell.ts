import { onMounted, onUnmounted, ref } from "vue";
import { friendlyError, getSystemInfo } from "@/tauri/commands";
import { useNetworkStatus } from "@/shared/composables/useNetworkStatus";
import type { MessageMode } from "@/shared/types/app";
import type { SystemInfo } from "@/shared/types/system";

const BOOTSTRAP_MIN_DURATION_MS = 900;

async function setCurrentWindowResizable(isResizable: boolean) {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().setResizable(isResizable);
  } catch {
    // Browser dev sessions do not expose a Tauri window.
  }
}

export function useAppShell() {
  const systemInfo = ref<SystemInfo>({
    username: "-",
    timestamp: "-",
    ip_address: "-",
    version: "-",
  });
  const message = ref("");
  const messageMode = ref<MessageMode>("info");
  const isSidebarCollapsed = ref(false);
  const isBootstrapping = ref(true);

  const network = useNetworkStatus();

  let pollTimer: number | undefined;

  async function refreshSystemInfo() {
    try {
      const info = await getSystemInfo();
      systemInfo.value = info;
    } catch {
      systemInfo.value = {
        ...systemInfo.value,
        timestamp: new Date().toLocaleString(),
      };
    }
  }

  function showMessage(nextMessage: string, nextMode: MessageMode = "info") {
    message.value = nextMessage;
    messageMode.value = nextMode;
  }

  function showError(error: unknown) {
    showMessage(friendlyError(error), "error");
  }

  function toggleSidebar() {
    isSidebarCollapsed.value = !isSidebarCollapsed.value;
  }

  onMounted(async () => {
    const startedAt = performance.now();

    await setCurrentWindowResizable(false);

    // Kick off the connectivity check immediately so it overlaps the splash.
    const networkReady = network.start();
    await refreshSystemInfo();

    const elapsed = performance.now() - startedAt;
    const remainingDelay = Math.max(0, BOOTSTRAP_MIN_DURATION_MS - elapsed);

    await new Promise((resolve) => window.setTimeout(resolve, remainingDelay));
    // Ensure the initial connectivity decision is ready before revealing the
    // app, so an offline launch shows the error screen without flashing the UI.
    await networkReady;

    isBootstrapping.value = false;
    void setCurrentWindowResizable(true);
    pollTimer = window.setInterval(() => void refreshSystemInfo(), 1000);
  });

  onUnmounted(() => {
    if (pollTimer !== undefined) {
      window.clearInterval(pollTimer);
    }
    void setCurrentWindowResizable(true);
  });

  return {
    isSidebarCollapsed,
    isBootstrapping,
    message,
    messageMode,
    showError,
    showMessage,
    systemInfo,
    toggleSidebar,
  };
}
