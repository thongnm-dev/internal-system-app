import { ref } from "vue";
import { friendlyError } from "@/tauri/commands/_base";
import { aiUsageAddAccount, aiUsageDeleteAccount, aiUsageListAccounts } from "@/tauri/commands/ai-usage";
import { useToast } from "@/shared/composables/useToast";
import type { AiAccount } from "@/_/types/ai-usage";

export function useAiUsage() {
  const toast = useToast();

  const accounts = ref<AiAccount[]>([]);
  const isLoading = ref(false);
  const isSaving = ref(false);

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

  async function addAccount(name: string, apiKey: string): Promise<boolean> {
    isSaving.value = true;
    try {
      const account = await aiUsageAddAccount({ name, api_key: apiKey });
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

  async function deleteAccount(id: number) {
    try {
      await aiUsageDeleteAccount(id);
      toast.success("Account deleted.");
      await loadAccounts();
    } catch (error) {
      toast.error(friendlyError(error));
    }
  }

  return {
    accounts,
    isLoading,
    isSaving,
    loadAccounts,
    addAccount,
    deleteAccount,
  };
}
