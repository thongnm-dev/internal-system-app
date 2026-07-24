import { ref, computed, onMounted, onUnmounted } from "vue";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { s3ListBugFolderTabs } from "@/tauri/commands/s3";
import type { BugFolderTab } from "@/_/types/s3";
import { useCloudGuard } from "./useCloudGuard";
import { useToast } from "@/shared/composables/useToast";

const POLL_INTERVAL = 5 * 60 * 1000;

export function useS3BugFolders() {
  const guard = useCloudGuard();
  const toast = useToast();

  const tabs = ref<BugFolderTab[]>([]);
  const isLoading = ref(false);
  const isRefreshing = ref(false);

  let pollTimer: ReturnType<typeof setInterval> | null = null;

  const tabsWithItems = computed(() =>
    tabs.value.filter((t) => t.items.length > 0),
  );

  const totalBugCount = computed(() =>
    tabs.value.reduce((sum, t) => sum + t.items.length, 0),
  );

  async function loadAll() {
    if (!canUseTauriRuntime()) return;
    if (!(await guard.ensureOnline())) return;
    isLoading.value = true;
    try {
      tabs.value = await s3ListBugFolderTabs();
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isLoading.value = false;
    }
  }

  async function refresh() {
    if (!(await guard.ensureOnline())) return;
    isRefreshing.value = true;
    try {
      tabs.value = await s3ListBugFolderTabs();
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isRefreshing.value = false;
    }
  }

  function startPolling() {
    stopPolling();
    pollTimer = setInterval(() => {
      refresh();
    }, POLL_INTERVAL);
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  onMounted(() => {
    loadAll();
    startPolling();
  });

  onUnmounted(() => {
    stopPolling();
  });

  return {
    tabs,
    tabsWithItems,
    totalBugCount,
    isLoading,
    isRefreshing,

    showOfflineDialog: guard.showOfflineDialog,
    offlineMessage: guard.offlineMessage,
    dismissOfflineDialog: guard.dismissOfflineDialog,

    refresh,
  };
}
