import { onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { friendlyError } from "@/tauri/commands/_base";
import { getSystemInfo } from "@/tauri/commands/system";
import { onS3NewDocuments } from "@/tauri/events";
import { useToast } from "@/shared/composables/useToast";
import { useNetworkStatus } from "@/shared/composables/useNetworkStatus";
import { useDatabaseStatus } from "@/shared/composables/useDatabaseStatus";
import { useAuthStore } from "@/app/stores/auth";
import { useMenuStore } from "@/app/stores/menu";
import { loginRoute } from "@/app/router/routes";
import type { MenuKey, MessageMode } from "@/_/types/app";
import type { SystemInfo } from "@/_/types/system";

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

  const router = useRouter();
  const auth = useAuthStore();
  const menu = useMenuStore();
  const network = useNetworkStatus();
  const database = useDatabaseStatus();
  const toast = useToast();

  let pollTimer: number | undefined;
  let unlistenS3NewDocs: UnlistenFn | undefined;

  // Re-verify the database whenever connectivity is (re)established while it is
  // not yet confirmed connected — e.g. the app was launched offline and the
  // user has just retried successfully, so the bootstrap DB check was skipped.
  watch(
    () => network.isOnline.value,
    (online, wasOnline) => {
      if (online && !wasOnline && !database.isConnected.value && !database.isChecking.value) {
        void database.check();
      }
    },
  );

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
    const online = await networkReady;

    // After the internet check, verify the database configuration. When online
    // (the DB check screen only follows a successful connectivity check), probe
    // the database so a missing/invalid config shows the config screen instead
    // of flashing the app.
    if (online) {
      await database.check();
    }

    // Load the remembered user's effective menu permissions before revealing
    // the shell so the sidebar renders already filtered (no flash of denied
    // menus) and the router guard has data to act on.
    if (auth.isAuthenticated && auth.user && database.isConnected.value) {
      await menu.load(auth.user.user_id);

      // A remembered session may have deep-linked into a now-denied menu; the
      // beforeEach guard won't fire without a navigation, so redirect here.
      const currentKey = router.currentRoute.value.meta.key as MenuKey | undefined;
      if (currentKey && !menu.canAccess(currentKey)) {
        router.replace("/overview");
      }
    }

    isBootstrapping.value = false;

    if (!auth.isAuthenticated) {
      router.push(loginRoute.path);
    }

    void setCurrentWindowResizable(true);
    pollTimer = window.setInterval(() => void refreshSystemInfo(), 1000);

    // Song song với notification native, hiện toast in-app khi poll nền phát
    // hiện tài liệu S3 mới (chỉ có tác dụng khi user đang mở app).
    unlistenS3NewDocs = await onS3NewDocuments((payload) => {
      const detail = payload.storages.map((s) => s.name).join(", ");
      toast.info(`Có ${payload.total} tài liệu mới trên S3${detail ? ` — ${detail}` : ""}`);
    });
  });

  onUnmounted(() => {
    if (pollTimer !== undefined) {
      window.clearInterval(pollTimer);
    }
    unlistenS3NewDocs?.();
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
