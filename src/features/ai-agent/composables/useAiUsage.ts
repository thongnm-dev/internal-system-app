import { onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { friendlyError } from "@/tauri/commands/_base";
import {
  aiUsageAddAccount,
  aiUsageDeleteAccount,
  aiUsageGetSettings,
  aiUsageGetToken,
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
  UpdateAiAccountRequest,
} from "@/_/types/ai-usage";

export function useAiUsage() {
  const toast = useToast();

  const accounts = ref<AiAccount[]>([]);
  const settings = ref<AiUsageSettings>({ switch_threshold_percent: 10, poll_interval_secs: 60 });
  const isLoading = ref(false);
  const isSaving = ref(false);
  const isRefreshing = ref(false);

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
    start,
    loadAccounts,
    addAccount,
    updateAccount,
    deleteAccount,
    setActive,
    copyToken,
    reportExhausted,
    refresh,
    saveSettings,
  };
}
