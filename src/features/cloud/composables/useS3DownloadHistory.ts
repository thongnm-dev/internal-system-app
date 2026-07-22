import { ref, onMounted } from "vue";
import { s3SearchDownloadHistory, s3ListDownloadStorages } from "@/tauri/commands/s3";
import { friendlyError } from "@/tauri/commands/_base";
import { useToast } from "@/shared/composables/useToast";
import type { DownloadHistorySearchParams, DownloadHistorySearchItem, AwsStorage } from "@/_/types/s3";

export function useS3DownloadHistory() {
  const toast = useToast();

  const storages = ref<AwsStorage[]>([]);
  const results = ref<DownloadHistorySearchItem[]>([]);
  const isSearching = ref(false);

  const params = ref<DownloadHistorySearchParams>({
    fromDate: "",
    toDate: "",
    awsCd: "",
    bugNo: "",
    isMovedAtLocal: false,
    isMovedAtS3: false,
  });

  async function loadStorages() {
    try {
      storages.value = await s3ListDownloadStorages();
    } catch {
      storages.value = [];
    }
  }

  async function search() {
    isSearching.value = true;
    try {
      results.value = await s3SearchDownloadHistory(params.value);
    } catch (e) {
      toast.error(friendlyError(e));
      results.value = [];
    } finally {
      isSearching.value = false;
    }
  }

  function clearSearch() {
    params.value = {
      fromDate: "",
      toDate: "",
      awsCd: "",
      bugNo: "",
      isMovedAtLocal: false,
      isMovedAtS3: false,
    };
    results.value = [];
  }

  onMounted(() => {
    loadStorages();
  });

  return {
    storages,
    results,
    isSearching,
    params,
    search,
    clearSearch,
  };
}
