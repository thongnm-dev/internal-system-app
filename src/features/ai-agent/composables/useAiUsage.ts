import { onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { friendlyError } from "@/tauri/commands/_base";
import {
  aiUsageAddAccount,
  aiUsageDeleteAccount,
  aiUsageDetectLocal,
  aiUsageGetSettings,
  aiUsageGetToken,
  aiUsageImportDetected,
  aiUsageListAccounts,
  aiUsageRefresh,
  aiUsageReportSignal,
  aiUsageSaveSettings,
  aiUsageSetActive,
  aiUsageUpdateAccount,
} from "@/tauri/commands/ai-usage";
import { onAiUsageUpdated } from "@/tauri/events";
import { useToast } from "@/shared/composables/useToast";
import type {
  AddAiAccountRequest,
  AiAccount,
  AiUsageSettings,
  DetectedLogin,
  UpdateAiAccountRequest,
} from "@/_/types/ai-usage";

export function useAiUsage() {
  const toast = useToast();

  const accounts = ref<AiAccount[]>([]);
  const settings = ref<AiUsageSettings>({
    switch_threshold_percent: 10,
    poll_interval_secs: 60,
    work_dir: "",
  });
  const isLoading = ref(false);
  const isSaving = ref(false);
  const isRefreshing = ref(false);
  const isDetecting = ref(false);
  const detected = ref<DetectedLogin[]>([]);

  let unlisten: UnlistenFn | null = null;

  async function loadAccounts() {
    isLoading.value = true;
    try {
      accounts.value = await aiUsageListAccounts();
    } catch (error) {
      toast.error(friendlyError(error));
    } finally {
      isLoading.value = false;
    }
  }

  async function loadSettings() {
    try {
      settings.value = await aiUsageGetSettings();
    } catch (error) {
      toast.error(friendlyError(error));
    }
  }

  async function addAccount(request: AddAiAccountRequest): Promise<boolean> {
    isSaving.value = true;
    try {
      const account = await aiUsageAddAccount(request);
      toast.success(`Account "${account.name}" added.`);
      await loadAccounts();
      return true;
    } catch (error) {
      toast.error(friendlyError(error));
      return false;
    } finally {
      isSaving.value = false;
    }
  }

  async function updateAccount(request: UpdateAiAccountRequest): Promise<boolean> {
    try {
      await aiUsageUpdateAccount(request);
      await loadAccounts();
      return true;
    } catch (error) {
      toast.error(friendlyError(error));
      return false;
    }
  }

  async function deleteAccount(id: number) {
    try {
      await aiUsageDeleteAccount(id);
      toast.success("Account deleted.");
      await loadAccounts();
    } catch (error) {
      toast.error(friendlyError(error));
    }
  }

  async function setActive(id: number) {
    try {
      await aiUsageSetActive(id);
      await loadAccounts();
    } catch (error) {
      toast.error(friendlyError(error));
    }
  }

  async function copyToken(id: number) {
    try {
      const token = await aiUsageGetToken(id);
      await navigator.clipboard.writeText(token);
      toast.success("Token copied to clipboard.");
    } catch (error) {
      toast.error(friendlyError(error));
    }
  }

  async function reportExhausted(id: number) {
    try {
      await aiUsageReportSignal({ id, exhausted: true });
      await loadAccounts();
    } catch (error) {
      toast.error(friendlyError(error));
    }
  }

  /** Dò các login Claude đã tồn tại trên máy (không thêm gì). */
  async function detectLocal(): Promise<boolean> {
    isDetecting.value = true;
    try {
      detected.value = await aiUsageDetectLocal();
      return true;
    } catch (error) {
      toast.error(friendlyError(error));
      return false;
    } finally {
      isDetecting.value = false;
    }
  }

  /** Dò + tự thêm những login chưa có vào danh sách account. */
  async function importDetected() {
    isDetecting.value = true;
    try {
      const before = accounts.value.length;
      accounts.value = await aiUsageImportDetected();
      const added = accounts.value.length - before;
      toast.success(added > 0 ? `Đã thêm ${added} account từ login local.` : "Không có login mới để thêm.");
      await detectLocal();
    } catch (error) {
      toast.error(friendlyError(error));
    } finally {
      isDetecting.value = false;
    }
  }

  async function refresh() {
    isRefreshing.value = true;
    try {
      accounts.value = await aiUsageRefresh();
    } catch (error) {
      toast.error(friendlyError(error));
    } finally {
      isRefreshing.value = false;
    }
  }

  async function saveSettings(next: AiUsageSettings): Promise<boolean> {
    try {
      await aiUsageSaveSettings(next);
      settings.value = next;
      toast.success("Settings saved.");
      return true;
    } catch (error) {
      toast.error(friendlyError(error));
      return false;
    }
  }

  async function start() {
    await Promise.all([loadAccounts(), loadSettings()]);
    // Cập nhật danh sách mỗi khi poll nền bắn event.
    unlisten = await onAiUsageUpdated(() => {
      void loadAccounts();
    });
  }

  onUnmounted(() => {
    unlisten?.();
    unlisten = null;
  });

  return {
    accounts,
    settings,
    isLoading,
    isSaving,
    isRefreshing,
    isDetecting,
    detected,
    start,
    loadAccounts,
    addAccount,
    updateAccount,
    deleteAccount,
    setActive,
    copyToken,
    reportExhausted,
    detectLocal,
    importDetected,
    refresh,
    saveSettings,
  };
}
