import { ref, computed, onMounted, onUnmounted } from "vue";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import {
  s3ListDownloadStorages,
  s3CheckDownloadAvailable,
  s3GetDownloadList,
  s3DownloadByStorage,
  s3MoveObjects,
  s3DeleteByStorage,
  s3GetDownloadHistory,
  s3UpdateDownloadMovedLocal,
} from "@/tauri/commands/s3";
import { open } from "@tauri-apps/plugin-dialog";
import type { AwsStorage, DownloadAvailability, DownloadHistoryItem } from "@/_/types/s3";
import { useCloudGuard } from "./useCloudGuard";
import { useToast } from "@/shared/composables/useToast";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { useAuthStore } from "@/app/stores/auth";

const POLL_INTERVAL = 15 * 60 * 1000;

export function useS3Download() {
  const guard = useCloudGuard();
  const toast = useToast();
  const loading = useGlobalLoading();
  const authStore = useAuthStore();

  const downloadStorages = ref<AwsStorage[]>([]);
  const downloadable = ref<Record<string, DownloadAvailability>>({});
  const isLoading = ref(false);
  const isReloading = ref(false);
  const downloadHistory = ref<DownloadHistoryItem[]>([]);

  let pollTimer: ReturnType<typeof setInterval> | null = null;

  const hasDownloadable = computed(() => {
    if (Object.keys(downloadable.value).length === 0) return false;
    return downloadStorages.value.some(
      (s) => downloadable.value[s.code]?.downloadAvailable,
    );
  });

  const downloadableStorages = computed(() =>
    downloadStorages.value.filter(
      (s) => downloadable.value[s.code]?.downloadAvailable,
    ),
  );

  async function loadDownloadStorages() {
    if (!canUseTauriRuntime()) return;
    if (!(await guard.ensureOnline())) return;
    isLoading.value = true;
    try {
      downloadStorages.value = await s3ListDownloadStorages();
      if (downloadStorages.value.length > 0) {
        await checkAvailability();
      }
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isLoading.value = false;
    }
  }

  async function checkAvailability() {
    if (downloadStorages.value.length === 0) return;
    try {
      const codes = downloadStorages.value.map((s) => s.code);
      downloadable.value = await s3CheckDownloadAvailable(codes);
    } catch (e) {
      toast.error(friendlyError(e));
    }
  }

  async function refresh() {
    if (!(await guard.ensureOnline())) return;
    isReloading.value = true;
    try {
      downloadable.value = {};
      await checkAvailability();
    } finally {
      isReloading.value = false;
    }
  }

  async function getDownloadList(code: string): Promise<string[]> {
    if (!canUseTauriRuntime()) return [];
    try {
      return await s3GetDownloadList(code);
    } catch (e) {
      toast.error(friendlyError(e));
      return [];
    }
  }

  async function selectFolder(): Promise<string | null> {
    if (!canUseTauriRuntime()) return null;
    const dir = await open({ directory: true, title: "Chọn thư mục lưu tập tin" });
    return dir as string | null;
  }

  async function downloadFiles(
    code: string,
    bugList: string[],
    localPath: string,
  ): Promise<{ syncPath: string; historyId: number | null } | null> {
    if (bugList.length === 0) return null;
    if (!(await guard.ensureOnline())) return null;
    loading.start();
    try {
      const userId = authStore.user?.username || "";
      const result = await s3DownloadByStorage(code, bugList, localPath, userId);
      if (result.success) {
        toast.success(result.message);
        await loadHistory();
        const matched = downloadHistory.value.find(
          (h) => h.syncPath === result.syncPath && !h.isMovedAtLocal,
        );
        return { syncPath: result.syncPath, historyId: matched?.id ?? null };
      } else {
        toast.error(result.message);
        return null;
      }
    } catch (e) {
      toast.error(friendlyError(e));
      return null;
    } finally {
      loading.stop();
    }
  }

  async function moveObjects(code: string, items: string[]) {
    if (items.length === 0) return;
    if (!(await guard.ensureOnline())) return;
    loading.start();
    try {
      const result = await s3MoveObjects(code, items);
      if (result.success) {
        toast.success(result.message);
      } else {
        toast.error(result.message);
      }
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      loading.stop();
    }
  }

  async function deleteObjects(code: string, items: string[]) {
    if (items.length === 0) return;
    if (!(await guard.ensureOnline())) return;
    loading.start();
    try {
      const result = await s3DeleteByStorage(code, items);
      if (result.success) {
        toast.success(result.message);
      } else {
        toast.error(result.message);
      }
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      loading.stop();
    }
  }

  async function updateMovedLocal(id: number, pathCopied: string) {
    try {
      await s3UpdateDownloadMovedLocal(id, pathCopied);
      await loadHistory();
    } catch (e) {
      toast.error(friendlyError(e));
    }
  }

  async function loadHistory() {
    if (!canUseTauriRuntime()) return;
    try {
      const userId = authStore.user?.username || "";
      if (userId) {
        downloadHistory.value = await s3GetDownloadHistory(userId);
      }
    } catch {
      // silent — history is non-critical
    }
  }

  function startPolling() {
    stopPolling();
    pollTimer = setInterval(() => {
      checkAvailability();
    }, POLL_INTERVAL);
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  onMounted(() => {
    loadDownloadStorages();
    loadHistory();
    startPolling();
  });

  onUnmounted(() => {
    stopPolling();
  });

  return {
    downloadStorages,
    downloadable,
    isLoading,
    isReloading,
    hasDownloadable,
    downloadableStorages,
    downloadHistory,

    showOfflineDialog: guard.showOfflineDialog,
    offlineMessage: guard.offlineMessage,
    dismissOfflineDialog: guard.dismissOfflineDialog,
    ensureOnline: guard.ensureOnline,

    refresh,
    getDownloadList,
    selectFolder,
    downloadFiles,
    moveObjects,
    deleteObjects,
    checkAvailability,
    loadHistory,
    updateMovedLocal,
  };
}
