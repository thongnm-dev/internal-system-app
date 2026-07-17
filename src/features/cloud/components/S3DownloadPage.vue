<script setup lang="ts">
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import ProgressSpinner from "primevue/progressspinner";
import S3DownloadCard from "./S3DownloadCard.vue";
import { useS3Download } from "../composables/useS3Download";

const {
  isLoading,
  isReloading,
  hasDownloadable,
  downloadableStorages,
  showOfflineDialog,
  offlineMessage,
  dismissOfflineDialog,
  ensureOnline,
  refresh,
  getDownloadList,
  selectFolder,
  downloadFiles,
  moveObjects,
  deleteObjects,
  checkAvailability,
} = useS3Download();

async function handleRefresh() {
  await refresh();
}
</script>

<template>
  <div class="flex h-full flex-col gap-4">
    <!-- Loading -->
    <div v-if="isLoading" class="flex h-full items-center justify-center">
      <ProgressSpinner style="width: 40px; height: 40px" />
    </div>

    <!-- Downloadable items -->
    <template v-else-if="hasDownloadable">
      <div class="flex flex-col gap-3">
        <S3DownloadCard
          v-for="storage in downloadableStorages"
          :key="storage.code"
          :aws-storage="storage"
          :ensure-online="ensureOnline"
          :get-download-list="getDownloadList"
          :select-folder="selectFolder"
          :download-files="downloadFiles"
          :move-objects="moveObjects"
          :delete-objects="deleteObjects"
          @refreshed="checkAvailability"
        />
      </div>
    </template>

    <!-- Empty state -->
    <template v-else>
      <div
        class="flex h-full flex-col items-center justify-center rounded-lg bg-surface-0 text-lg text-surface-500 dark:bg-surface-900"
      >
        <i class="pi pi-cloud-download mb-4 text-5xl text-surface-300" />
        <span class="animate-bounce py-4 text-sm text-red-500">
          Không có tập tin nào để tải về...
        </span>
        <Button
          label="Làm mới trạng thái"
          icon="pi pi-refresh"
          severity="secondary"
          outlined
          :loading="isReloading"
          @click="handleRefresh"
        />
      </div>
    </template>

    <!-- Offline Dialog -->
    <Dialog
      v-model:visible="showOfflineDialog"
      header="Lỗi kết nối"
      :modal="true"
      :closable="true"
      :style="{ width: '28rem' }"
    >
      <div class="flex items-center gap-3">
        <i class="pi pi-wifi text-3xl text-red-500" />
        <span class="text-sm text-surface-600 dark:text-surface-400">{{ offlineMessage }}</span>
      </div>
      <template #footer>
        <Button label="Đóng" @click="dismissOfflineDialog()" />
      </template>
    </Dialog>
  </div>
</template>
