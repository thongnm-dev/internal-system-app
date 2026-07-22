import { ref, onMounted } from "vue";
import { s3SearchUploadHistory, s3ListUploadStorages } from "@/tauri/commands/s3";
import { friendlyError } from "@/tauri/commands/_base";
import { useToast } from "@/shared/composables/useToast";
import type { UploadHistorySearchParams, UploadHistorySearchItem, AwsStorage } from "@/_/types/s3";

export function useS3UploadHistory() {
  const toast = useToast();

  const storages = ref<AwsStorage[]>([]);
  const results = ref<UploadHistorySearchItem[]>([]);
  const isSearching = ref(false);

  const params = ref<UploadHistorySearchParams>({
    fromDate: "",
    toDate: "",
    awsCd: "",
    bugNo: "",
    isMovedAtS3: false,
  });

  async function loadStorages() {
    try {
      storages.value = await s3ListUploadStorages();
    } catch {
      storages.value = [];
    }
  }

  async function search() {
    isSearching.value = true;
    try {
      results.value = await s3SearchUploadHistory(params.value);
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
