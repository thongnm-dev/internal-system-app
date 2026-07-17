import { ref } from "vue";
import { useNetworkStatus } from "@/shared/composables/useNetworkStatus";

const OFFLINE_MESSAGE = "Lỗi không thể kết nối mạng. Vui lòng kiểm tra kết nối internet!";

export function useCloudGuard() {
  const { retry } = useNetworkStatus();
  const showOfflineDialog = ref(false);

  async function ensureOnline(): Promise<boolean> {
    const online = await retry();
    if (!online) {
      showOfflineDialog.value = true;
      return false;
    }
    return true;
  }

  function dismissOfflineDialog() {
    showOfflineDialog.value = false;
  }

  return {
    showOfflineDialog,
    offlineMessage: OFFLINE_MESSAGE,
    ensureOnline,
    dismissOfflineDialog,
  };
}
