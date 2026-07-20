<script setup lang="ts">
import { ref } from "vue";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import ProgressSpinner from "primevue/progressspinner";
import S3DownloadCard from "./S3DownloadCard.vue";
import S3ConfigError from "./S3ConfigError.vue";
import { useS3Download } from "../composables/useS3Download";
import { useS3ConfigGuard } from "../composables/useS3ConfigGuard";
import { explorerReadDir, explorerCopyBugs } from "@/tauri/commands/explorer";
import type { FileEntry } from "@/tauri/commands/explorer";
import { open } from "@tauri-apps/plugin-dialog";
import { useToast } from "@/shared/composables/useToast";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { friendlyError } from "@/tauri/commands/_base";

const s3Guard = useS3ConfigGuard();
s3Guard.checkConfig();

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

const toast = useToast();
const loading = useGlobalLoading();

const COPY_DEST_KEY = "copy_dest_state";

const lastDownloadPath = ref("");
const showCopyDialog = ref(false);
const copyEntries = ref<FileEntry[]>([]);
const copyDestPath = ref("");
const isCopying = ref(false);

function handleDownloaded(path: string) {
  lastDownloadPath.value = path;
}

function loadSavedCopyDest() {
  try {
    const saved = localStorage.getItem(COPY_DEST_KEY);
    if (saved) copyDestPath.value = saved;
  } catch {
    // ignore
  }
}

function saveCopyDest() {
  try {
    localStorage.setItem(COPY_DEST_KEY, copyDestPath.value);
  } catch {
    // ignore
  }
}

async function openCopyDialog() {
  loadSavedCopyDest();
  try {
    const result = await explorerReadDir(lastDownloadPath.value);
    copyEntries.value = result.entries;
  } catch {
    copyEntries.value = [];
  }
  showCopyDialog.value = true;
}

async function chooseCopyDest() {
  const dir = await open({ directory: true, title: "Chọn thư mục đích" });
  if (dir) {
    copyDestPath.value = dir as string;
    saveCopyDest();
  }
}

async function handleCopy() {
  if (!copyDestPath.value) return;
  isCopying.value = true;
  loading.start();
  try {
    const msg = await explorerCopyBugs(lastDownloadPath.value, copyDestPath.value);
    toast.success(msg);
    showCopyDialog.value = false;
  } catch (e) {
    toast.error(friendlyError(e));
  } finally {
    isCopying.value = false;
    loading.stop();
  }
}

async function handleRefresh() {
  await refresh();
}
</script>

<template>
  <S3ConfigError
    v-if="s3Guard.configError.value"
    :error="s3Guard.configError.value"
    :is-checking="s3Guard.configChecking.value"
    @retry="s3Guard.checkConfig()"
  />

  <div v-else class="flex h-full flex-col gap-4">
    <!-- Loading -->
    <div v-if="isLoading" class="flex h-full items-center justify-center">
      <ProgressSpinner style="width: 40px; height: 40px" />
    </div>

    <!-- Downloadable items -->
    <template v-else-if="hasDownloadable">
      <div class="flex flex-1 flex-col gap-3 overflow-y-auto">
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
          @downloaded="handleDownloaded"
        />
      </div>

      <!-- Sticky panel after download -->
      <div
        v-if="lastDownloadPath"
        class="sticky bottom-0 flex items-center gap-2 rounded-lg border border-surface-200 bg-surface-0 px-4 py-3 shadow-md dark:border-surface-700 dark:bg-surface-900"
      >
        <i class="pi pi-folder-open text-lg text-green-500" />
        <InputText
          :model-value="lastDownloadPath"
          class="flex-1 font-mono text-sm"
          readonly
        />
        <Button
          label="Copy"
          icon="pi pi-copy"
          severity="info"
          size="small"
          @click="openCopyDialog"
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

    <!-- Copy Dialog -->
    <Dialog
      v-model:visible="showCopyDialog"
      header="Bố trí tập tin"
      :modal="true"
      :style="{ width: '40rem' }"
      :closable="true"
    >
      <div class="flex flex-col gap-4">
        <!-- File list from download path -->
        <div>
          <h4 class="mb-2 text-sm font-semibold text-surface-700 dark:text-surface-200">
            Danh sách file đã tải:
          </h4>
          <div
            class="max-h-52 overflow-y-auto rounded-lg border border-surface-200 bg-surface-50 dark:border-surface-700 dark:bg-surface-800"
          >
            <div
              v-for="entry in copyEntries"
              :key="entry.path"
              class="flex items-center gap-2 border-b border-surface-100 px-3 py-2 last:border-b-0 dark:border-surface-700"
            >
              <i
                :class="[
                  'pi text-sm',
                  entry.is_dir
                    ? 'pi-folder text-orange-500'
                    : 'pi-file text-surface-400',
                ]"
              />
              <span class="truncate text-sm text-surface-700 dark:text-surface-300">
                {{ entry.name }}
              </span>
            </div>
            <div
              v-if="copyEntries.length === 0"
              class="px-3 py-4 text-center text-sm text-surface-400"
            >
              Thư mục trống
            </div>
          </div>
        </div>

        <!-- Destination path picker -->
        <div>
          <h4 class="mb-2 text-sm font-semibold text-surface-700 dark:text-surface-200">
            Đường dẫn đích nơi lưu:
          </h4>
          <div class="flex items-center gap-2">
            <InputText
              v-model="copyDestPath"
              class="flex-1 font-mono text-sm"
              placeholder="Chưa chọn thư mục đích"
              readonly
            />
            <Button
              icon="pi pi-folder-open"
              severity="secondary"
              @click="chooseCopyDest"
            />
          </div>
        </div>
      </div>
      <template #footer>
        <Button
          label="Đóng"
          icon="pi pi-times"
          severity="secondary"
          @click="showCopyDialog = false"
        />
        <Button
          label="Copy"
          icon="pi pi-copy"
          :disabled="!copyDestPath"
          :loading="isCopying"
          @click="handleCopy"
        />
      </template>
    </Dialog>
  </div>
</template>
