import { ref, onMounted } from "vue";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { s3ListUploadStorages, s3ScanUploadFolder, s3UploadFiles, s3ListDeleteOptions, s3DeleteUploadedItems } from "@/tauri/commands/s3";
import { open } from "@tauri-apps/plugin-dialog";
import type { AwsStorage, ScannedFile, S3OperationResult, UploadFileRequest, DeleteUploadedItem } from "@/_/types/s3";
import { useCloudGuard } from "./useCloudGuard";
import { useToast } from "@/shared/composables/useToast";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { useAuthStore } from "@/app/stores/auth";

export function useS3Upload() {
  const guard = useCloudGuard();
  const toast = useToast();
  const loading = useGlobalLoading();
  const authStore = useAuthStore();

  const uploadStorages = ref<AwsStorage[]>([]);
  const isLoading = ref(false);
  const isUploading = ref(false);
  const isDeleting = ref(false);

  const deleteItems = ref<DeleteUploadedItem[]>([]);
  const deleteOptions = ref<AwsStorage[]>([]);
  const showDeleteDialog = ref(false);

  async function loadUploadStorages() {
    if (!canUseTauriRuntime()) return;
    if (!(await guard.ensureOnline())) return;
    isLoading.value = true;
    try {
      uploadStorages.value = await s3ListUploadStorages();
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isLoading.value = false;
    }
  }

  async function scanFolder(): Promise<ScannedFile[]> {
    if (!canUseTauriRuntime()) return [];
    if (!(await guard.ensureOnline())) return [];

    const dir = await open({ directory: true, title: "Chọn thư mục chứa tập tin" });
    if (!dir) return [];

    try {
      return await s3ScanUploadFolder(dir as string);
    } catch (e) {
      toast.error(friendlyError(e));
      return [];
    }
  }

  async function uploadFiles(
    files: UploadFileRequest[],
    storage: AwsStorage,
    createFolderSameName: boolean,
  ): Promise<S3OperationResult | null> {
    if (files.length === 0) return null;
    if (!(await guard.ensureOnline())) return null;
    isUploading.value = true;
    loading.start();
    try {
      const userId = authStore.user?.username || "";
      const result = await s3UploadFiles(files, storage.name, storage.subscribe, createFolderSameName, storage.code, userId);

      if (result.success) {
        toast.success(`Đã thực hiện tải thành công ${result.processed} tập tin lên S3.`);

        if (!createFolderSameName && result.processed > 0) {
          try {
            deleteOptions.value = await s3ListDeleteOptions(storage.code);
          } catch {
            deleteOptions.value = [];
          }

          if (deleteOptions.value.length > 0) {
            const parentNames = [...new Set(files.map((f) => f.parentName))];
            const defaultCode = deleteOptions.value[0].code;
            deleteItems.value = parentNames.map((bugNo) => ({
              awsCd: defaultCode,
              bugNo,
            }));
          }
        }
      } else {
        toast.error(result.message);
      }

      return result;
    } catch (e) {
      toast.error(friendlyError(e));
      return null;
    } finally {
      isUploading.value = false;
      loading.stop();
    }
  }

  async function confirmDelete() {
    if (deleteItems.value.length === 0) return;
    if (!(await guard.ensureOnline())) return;
    showDeleteDialog.value = false;
    isDeleting.value = true;
    loading.start();
    try {
      const result = await s3DeleteUploadedItems(deleteItems.value);
      if (result.success) {
        toast.success(result.message);
      } else {
        toast.error(result.message);
      }
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isDeleting.value = false;
      loading.stop();
      showDeleteDialog.value = false;
      deleteItems.value = [];
      deleteOptions.value = [];
    }
  }

  function dismissDeleteDialog() {
    showDeleteDialog.value = false;
    deleteItems.value = [];
    deleteOptions.value = [];
  }

  function updateDeleteItemCode(bugNo: string, awsCd: string) {
    deleteItems.value = deleteItems.value.map((item) =>
      item.bugNo === bugNo ? { ...item, awsCd } : item,
    );
  }

  onMounted(() => void loadUploadStorages());

  return {
    uploadStorages,
    isLoading,
    isUploading,
    isDeleting,

    deleteItems,
    deleteOptions,
    showDeleteDialog,

    showOfflineDialog: guard.showOfflineDialog,
    offlineMessage: guard.offlineMessage,
    dismissOfflineDialog: guard.dismissOfflineDialog,

    loadUploadStorages,
    scanFolder,
    uploadFiles,
    confirmDelete,
    dismissDeleteDialog,
    updateDeleteItemCode,
  };
}
