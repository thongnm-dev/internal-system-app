import { ref, computed, onMounted } from "vue";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import {
  s3TestConnection,
  s3ListObjects,
  s3DownloadObjects,
  s3UploadFile,
  s3DeleteObjects,
  s3CreateFolder,
} from "@/tauri/commands/s3";
import { open } from "@tauri-apps/plugin-dialog";
import type { S3Object, S3OperationResult } from "@/_/types/s3";
import type { MessageMode } from "@/_/types/app";

export function useS3Browser() {
  const objects = ref<S3Object[]>([]);
  const currentPrefix = ref("");
  const prefixHistory = ref<string[]>([]);
  const bucketName = ref("");
  const isConnected = ref(false);
  const isLoading = ref(false);
  const isTesting = ref(false);
  const isUploading = ref(false);
  const isDownloading = ref(false);
  const isDeleting = ref(false);
  const selectedKeys = ref<Set<string>>(new Set());
  const message = ref("Connecting to S3...");
  const messageMode = ref<MessageMode>("info");

  const breadcrumbs = computed(() => {
    const parts: { label: string; prefix: string }[] = [{ label: bucketName.value || "Bucket", prefix: "" }];
    if (!currentPrefix.value) return parts;

    const segments = currentPrefix.value.split("/").filter(Boolean);
    let accumulated = "";
    for (const seg of segments) {
      accumulated += seg + "/";
      parts.push({ label: seg, prefix: accumulated });
    }
    return parts;
  });

  const selectedCount = computed(() => selectedKeys.value.size);
  const allSelected = computed(
    () => objects.value.length > 0 && selectedKeys.value.size === objects.value.length,
  );

  async function loadAndConnect() {
    if (!canUseTauriRuntime()) return;
    isLoading.value = true;
    try {
      const result = await s3TestConnection();
      isConnected.value = true;
      message.value = result;
      messageMode.value = "info";
      await refresh();
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      isConnected.value = false;
    } finally {
      isLoading.value = false;
    }
  }

  onMounted(() => void loadAndConnect());

  async function testConnection() {
    if (!canUseTauriRuntime()) {
      message.value = "Tauri runtime is not available.";
      messageMode.value = "error";
      return;
    }
    isTesting.value = true;
    try {
      const result = await s3TestConnection();
      message.value = result;
      messageMode.value = "info";
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      isTesting.value = false;
    }
  }

  async function connect() {
    if (!canUseTauriRuntime()) {
      message.value = "Tauri runtime is not available.";
      messageMode.value = "error";
      return;
    }
    isLoading.value = true;
    try {
      await s3TestConnection();
      isConnected.value = true;
      currentPrefix.value = "";
      prefixHistory.value = [];
      message.value = "Connected successfully.";
      messageMode.value = "info";
      await refresh();
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      isConnected.value = false;
    } finally {
      isLoading.value = false;
    }
  }

  async function disconnect() {
    isConnected.value = false;
    objects.value = [];
    currentPrefix.value = "";
    prefixHistory.value = [];
    selectedKeys.value = new Set();
    message.value = "Disconnected.";
    messageMode.value = "info";
  }

  async function refresh() {
    if (!isConnected.value) return;
    isLoading.value = true;
    selectedKeys.value = new Set();
    try {
      const result = await s3ListObjects(currentPrefix.value);
      objects.value = result.objects;
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      isLoading.value = false;
    }
  }

  async function navigateToPrefix(prefix: string) {
    prefixHistory.value.push(currentPrefix.value);
    currentPrefix.value = prefix;
    await refresh();
  }

  async function navigateUp() {
    if (prefixHistory.value.length > 0) {
      currentPrefix.value = prefixHistory.value.pop()!;
      await refresh();
    }
  }

  async function navigateToBreadcrumb(prefix: string) {
    const idx = prefixHistory.value.findIndex((_, i) => {
      const segments = currentPrefix.value.split("/").filter(Boolean);
      const target = prefix.split("/").filter(Boolean);
      return target.length <= segments.length && i <= prefixHistory.value.length;
    });
    currentPrefix.value = prefix;
    if (idx >= 0) {
      prefixHistory.value = prefixHistory.value.slice(0, idx);
    } else {
      prefixHistory.value = [];
    }
    await refresh();
  }

  function toggleSelect(key: string) {
    const newSet = new Set(selectedKeys.value);
    if (newSet.has(key)) {
      newSet.delete(key);
    } else {
      newSet.add(key);
    }
    selectedKeys.value = newSet;
  }

  function toggleSelectAll() {
    if (allSelected.value) {
      selectedKeys.value = new Set();
    } else {
      selectedKeys.value = new Set(objects.value.map((o) => o.key));
    }
  }

  function clearSelection() {
    selectedKeys.value = new Set();
  }

  async function downloadSelected(): Promise<S3OperationResult | null> {
    const keys = Array.from(selectedKeys.value);
    if (keys.length === 0) return null;

    if (!canUseTauriRuntime()) {
      message.value = "Tauri runtime is not available.";
      messageMode.value = "error";
      return null;
    }

    const dir = await open({ directory: true, title: "Select download destination" });
    if (!dir) return null;

    isDownloading.value = true;
    try {
      const result = await s3DownloadObjects(keys, dir as string);
      message.value = result.message;
      messageMode.value = result.success ? "info" : "error";
      return result;
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      return null;
    } finally {
      isDownloading.value = false;
    }
  }

  async function downloadSingle(key: string): Promise<S3OperationResult | null> {
    if (!canUseTauriRuntime()) {
      message.value = "Tauri runtime is not available.";
      messageMode.value = "error";
      return null;
    }

    const dir = await open({ directory: true, title: "Select download destination" });
    if (!dir) return null;

    isDownloading.value = true;
    try {
      const result = await s3DownloadObjects([key], dir as string);
      message.value = result.message;
      messageMode.value = result.success ? "info" : "error";
      return result;
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      return null;
    } finally {
      isDownloading.value = false;
    }
  }

  async function uploadFile(): Promise<S3OperationResult | null> {
    if (!canUseTauriRuntime()) {
      message.value = "Tauri runtime is not available.";
      messageMode.value = "error";
      return null;
    }

    const selected = await open({
      multiple: false,
      title: "Select file to upload",
    });
    if (!selected) return null;

    const localPath = selected as string;
    const fileName = localPath.split(/[/\\]/).pop() || "file";
    const s3Key = currentPrefix.value + fileName;

    isUploading.value = true;
    try {
      const result = await s3UploadFile(localPath, s3Key);
      message.value = result.message;
      messageMode.value = result.success ? "info" : "error";
      if (result.success) await refresh();
      return result;
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      return null;
    } finally {
      isUploading.value = false;
    }
  }

  async function deleteSelected(): Promise<S3OperationResult | null> {
    const keys = Array.from(selectedKeys.value);
    if (keys.length === 0) return null;

    isDeleting.value = true;
    try {
      const result = await s3DeleteObjects(keys);
      message.value = result.message;
      messageMode.value = result.success ? "info" : "error";
      if (result.success) {
        selectedKeys.value = new Set();
        await refresh();
      }
      return result;
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      return null;
    } finally {
      isDeleting.value = false;
    }
  }

  async function deleteSingle(key: string): Promise<S3OperationResult | null> {
    isDeleting.value = true;
    try {
      const result = await s3DeleteObjects([key]);
      message.value = result.message;
      messageMode.value = result.success ? "info" : "error";
      if (result.success) {
        const newSet = new Set(selectedKeys.value);
        newSet.delete(key);
        selectedKeys.value = newSet;
        await refresh();
      }
      return result;
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      return null;
    } finally {
      isDeleting.value = false;
    }
  }

  async function createFolder(name: string): Promise<S3OperationResult | null> {
    if (!name.trim()) return null;
    const prefix = currentPrefix.value + name.trim();
    isLoading.value = true;
    try {
      const result = await s3CreateFolder(prefix);
      message.value = result.message;
      messageMode.value = result.success ? "info" : "error";
      if (result.success) await refresh();
      return result;
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    objects,
    currentPrefix,
    isConnected,
    isLoading,
    isTesting,
    isUploading,
    isDownloading,
    isDeleting,
    selectedKeys,
    selectedCount,
    allSelected,
    message,
    messageMode,
    breadcrumbs,

    testConnection,
    connect,
    disconnect,
    refresh,
    navigateToPrefix,
    navigateUp,
    navigateToBreadcrumb,
    toggleSelect,
    toggleSelectAll,
    clearSelection,
    downloadSelected,
    downloadSingle,
    uploadFile,
    deleteSelected,
    deleteSingle,
    createFolder,
  };
}
