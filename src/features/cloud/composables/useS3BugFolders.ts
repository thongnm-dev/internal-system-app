import { ref, computed, onMounted, onUnmounted } from "vue";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { s3ListBugFolderTabs } from "@/tauri/commands/s3";
import type { BugFolderTab } from "@/_/types/s3";
import { useToast } from "@/shared/composables/useToast";

const POLL_INTERVAL = 5 * 60 * 1000;

export function useS3BugFolders() {
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

    refresh,
  };
}
