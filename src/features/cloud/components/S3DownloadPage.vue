<script setup lang="ts">
import { computed, ref } from "vue";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import InputText from "primevue/inputtext";
import InputGroup from "primevue/inputgroup";
import ProgressSpinner from "primevue/progressspinner";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import S3DownloadCard from "./S3DownloadCard.vue";
import S3ConfigError from "./S3ConfigError.vue";
import S3BugFoldersDialog from "./S3BugFoldersDialog.vue";
import { useS3Download } from "../composables/useS3Download";
import { useS3ConfigGuard } from "../composables/useS3ConfigGuard";
import { explorerReadDir, explorerCopyBugs, explorerOpen } from "@/tauri/commands/explorer";
import type { FileEntry } from "@/tauri/commands/explorer";
import { open } from "@tauri-apps/plugin-dialog";
import { useToast } from "@/shared/composables/useToast";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { friendlyError } from "@/tauri/commands/_base";
import s3DownloadImg from "@/assets/s3-download2.webp";

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
  downloadHistory,
  updateMovedLocal,
} = useS3Download();

const toast = useToast();
const loading = useGlobalLoading();

const COPY_DEST_KEY = "copy_dest_state";

const lastDownloadPath = ref("");
const lastDownloadHistoryId = ref<number | null>(null);
const showS3ConfirmDialog = ref(false);
const showBugFoldersDialog = ref(false);
const showCopyDialog = ref(false);
const copyEntries = ref<FileEntry[]>([]);
const selectedCopyNames = ref<Set<string>>(new Set());
const copyDestPath = ref("");
const isCopying = ref(false);
const copyHistoryId = ref<number | null>(null);
const copySourcePath = ref("");

const copyFolderEntries = computed(() => copyEntries.value.filter((e) => e.is_dir));
const allCopyEntriesSelected = computed(
  () =>
    copyFolderEntries.value.length > 0 &&
    copyFolderEntries.value.every((e) => selectedCopyNames.value.has(e.name)),
);

function isCopyEntrySelected(entry: FileEntry): boolean {
  return selectedCopyNames.value.has(entry.name);
}

function toggleCopyEntrySelected(entry: FileEntry): void {
  const next = new Set(selectedCopyNames.value);
  if (next.has(entry.name)) {
    next.delete(entry.name);
  } else {
    next.add(entry.name);
  }
  selectedCopyNames.value = next;
}

function toggleSelectAllCopyEntries(): void {
  selectedCopyNames.value = allCopyEntriesSelected.value
    ? new Set()
    : new Set(copyFolderEntries.value.map((e) => e.name));
}

function handleDownloaded(path: string, historyId: number | null) {
  lastDownloadPath.value = path;
  lastDownloadHistoryId.value = historyId;
}

function handleMoved() {
  showS3ConfirmDialog.value = true;
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
  copyHistoryId.value = lastDownloadHistoryId.value;
  copySourcePath.value = lastDownloadPath.value;
  loadSavedCopyDest();
  try {
    const result = await explorerReadDir(lastDownloadPath.value);
    copyEntries.value = result.entries;
  } catch {
    copyEntries.value = [];
  }
  selectedCopyNames.value = new Set(copyFolderEntries.value.map((e) => e.name));
  showCopyDialog.value = true;
}

async function openCopyDialogForHistory(id: number, syncPath: string) {
  copyHistoryId.value = id;
  copySourcePath.value = syncPath;
  loadSavedCopyDest();
  try {
    const result = await explorerReadDir(syncPath);
    copyEntries.value = result.entries;
  } catch {
    copyEntries.value = [];
  }
  selectedCopyNames.value = new Set(copyFolderEntries.value.map((e) => e.name));
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
  if (!copyDestPath.value || !copySourcePath.value || selectedCopyNames.value.size === 0) return;
  isCopying.value = true;
  loading.start();
  try {
    const msg = await explorerCopyBugs(
      copySourcePath.value,
      copyDestPath.value,
      Array.from(selectedCopyNames.value),
    );
    toast.success(msg);
    if (copyHistoryId.value !== null) {
      await updateMovedLocal(copyHistoryId.value, copyDestPath.value);
    }
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

function formatDate(ymd: string): string {
  if (ymd.length !== 8) return ymd;
  return `${ymd.slice(0, 4)}/${ymd.slice(4, 6)}/${ymd.slice(6, 8)}`;
}

function formatTime(hms: string): string {
  if (hms.length >= 4) {
    return `${hms.slice(0, 2)}:${hms.slice(2, 4)}`;
  }
  return hms;
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
          @downloaded="(path, historyId) => handleDownloaded(path, historyId)"
          @moved="handleMoved"
        />
      </div>

      <!-- Sticky panel after download -->
      <div
        v-if="lastDownloadPath"
        class="sticky bottom-0 flex items-center gap-2 rounded-lg border border-divider bg-panel px-4 py-3 shadow-md"
      >
        <i class="pi pi-folder-open text-lg text-success" />
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
        <Button
          v-tooltip.top="'Show in folder'"
          icon="pi pi-external-link"
          severity="secondary"
          size="small"
          text
          @click="explorerOpen(lastDownloadPath)"
        />
      </div>
    </template>

    <!-- Empty state -->
    <template v-else>
      <div
        class="flex h-full flex-col items-center justify-center rounded-lg bg-panel text-lg text-muted"
      >
        <img
          :src="s3DownloadImg"
          alt="No files to download"
          class="mb-2 h-60 w-60 rounded-full object-contain"
        />
        <span class="animate-bounce py-4 text-sm text-danger">
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

    <!-- Download History -->
    <div
      v-if="!isLoading && downloadHistory.length > 0"
      class="rounded-lg bg-panel shadow"
    >
      <div class="flex items-center gap-2 border-b border-divider px-4 py-3">
        <i class="pi pi-history text-lg text-info" />
        <span class="text-sm font-semibold text-ink">
          Lịch sử tải về ({{ downloadHistory.length }})
        </span>
      </div>
      <DataTable
        :value="downloadHistory"
        scrollable
        scroll-height="260px"
        size="small"
        striped-rows
        class="text-sm"
      >
        <Column header="Ngày" :style="{ width: '100px' }">
          <template #body="{ data }">
            {{ formatDate(data.downloadYmd) }}
          </template>
        </Column>
        <Column header="Giờ" :style="{ width: '60px' }">
          <template #body="{ data }">
            {{ formatTime(data.downloadHms) }}
          </template>
        </Column>
        <Column header="Nơi lưu trữ">
          <template #body="{ data }">
            {{ data.awsNameAlias || data.awsName }}
          </template>
        </Column>
        <Column header="Số lượng" :style="{ width: '80px' }">
          <template #body="{ data }">
            <span class="font-semibold text-info">
              {{ data.downloadCount }}
            </span>
          </template>
        </Column>
        <Column header="Đường dẫn" :style="{ minWidth: '200px' }">
          <template #body="{ data }">
            <span class="font-mono text-xs break-all">{{ data.syncPath }}</span>
          </template>
        </Column>
        <Column header="" :style="{ width: '110px' }">
          <template #body="{ data }">
            <div class="flex items-center gap-1">
              <Button
                v-if="!data.isMovedAtLocal"
                label="Copy"
                icon="pi pi-copy"
                severity="info"
                size="small"
                text
                @click="openCopyDialogForHistory(data.id, data.syncPath)"
              />
              <span v-else class="text-xs text-success">
                <i class="pi pi-check mr-1" />Copied
              </span>
              <Button
                v-tooltip.top="'Show in folder'"
                icon="pi pi-external-link"
                severity="secondary"
                size="small"
                text
                @click="explorerOpen(data.syncPath)"
              />
            </div>
          </template>
        </Column>
      </DataTable>
    </div>

    <!-- Offline Dialog -->
    <Dialog
      v-model:visible="showOfflineDialog"
      header="Lỗi kết nối"
      :modal="true"
      :closable="true"
      :style="{ width: '28rem' }"
    >
      <div class="flex items-center gap-3">
        <i class="pi pi-wifi text-3xl text-danger" />
        <span class="text-sm text-secondary">{{ offlineMessage }}</span>
      </div>
      <template #footer>
        <Button label="Đóng" @click="dismissOfflineDialog()" />
      </template>
    </Dialog>

    <!-- S3 Status Confirm Dialog -->
    <Dialog
      v-model:visible="showS3ConfirmDialog"
      header="Xác nhận"
      :modal="true"
      :closable="true"
      :style="{ width: '28rem' }"
    >
      <div class="flex items-center gap-3">
        <i class="pi pi-question-circle text-3xl text-info" />
        <span class="text-sm text-secondary">
          Bạn có muốn mở màn hình để xem trạng thái trên S3 không?
        </span>
      </div>
      <template #footer>
        <DialogFooter
          confirm-label="OK"
          confirm-icon="pi pi-check"
          @cancel="showS3ConfirmDialog = false"
          @confirm="showS3ConfirmDialog = false; showBugFoldersDialog = true"
        />
      </template>
    </Dialog>

    <!-- S3 Bug Folders Dialog -->
    <S3BugFoldersDialog v-if="showBugFoldersDialog" @close="showBugFoldersDialog = false" />

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
          <div class="mb-2 flex items-center justify-between">
            <h4 class="text-sm font-semibold text-ink">
              Danh sách phiếu bug đã tải ({{ selectedCopyNames.size }}/{{ copyFolderEntries.length }} đã chọn):
            </h4>
            <label v-if="copyFolderEntries.length > 0" class="flex cursor-pointer items-center gap-1.5 text-xs text-secondary">
              <Checkbox
                :model-value="allCopyEntriesSelected"
                :indeterminate="selectedCopyNames.size > 0 && !allCopyEntriesSelected"
                binary
                @change="toggleSelectAllCopyEntries"
              />
              Chọn tất cả
            </label>
          </div>
          <div
            class="max-h-52 overflow-y-auto rounded-lg border border-divider bg-canvas"
          >
            <label
              v-for="entry in copyEntries"
              :key="entry.path"
              class="flex items-center gap-2 border-b border-divider px-3 py-2 last:border-b-0"
              :class="entry.is_dir ? 'cursor-pointer' : 'opacity-60'"
            >
              <Checkbox
                v-if="entry.is_dir"
                :model-value="isCopyEntrySelected(entry)"
                binary
                @change="toggleCopyEntrySelected(entry)"
              />
              <span v-else class="w-[18px]" />
              <i
                :class="[
                  'pi text-sm',
                  entry.is_dir
                    ? 'pi-folder text-amber-500'
                    : 'pi-file text-muted',
                ]"
              />
              <span class="truncate text-sm text-secondary">
                {{ entry.name }}
              </span>
            </label>
            <div
              v-if="copyEntries.length === 0"
              class="px-3 py-4 text-center text-sm text-muted"
            >
              Thư mục trống
            </div>
          </div>
        </div>

        <!-- Destination path picker -->
        <div>
          <h4 class="mb-2 text-sm font-semibold text-ink">
            Đường dẫn đích nơi lưu:
          </h4>
          <InputGroup class="h-8">
            <InputText
              v-model="copyDestPath"
              placeholder="Chưa chọn thư mục đích"
              readonly
            />
            <Button
              icon="pi pi-folder-open"
              severity="secondary"
              outlined
              title="Chọn thư mục đích"
              @click="chooseCopyDest"
            />
            <Button
              v-if="copyDestPath"
              icon="pi pi-times"
              severity="danger"
              text
              title="Xoá đường dẫn"
              @click="copyDestPath = ''"
            />
          </InputGroup>
        </div>
      </div>
      <template #footer>
        <DialogFooter
          cancel-label="Đóng"
          confirm-label="Copy"
          confirm-icon="pi pi-copy"
          :confirm-disabled="!copyDestPath || selectedCopyNames.size === 0"
          :busy="isCopying"
          @cancel="showCopyDialog = false"
          @confirm="handleCopy"
        />
      </template>
    </Dialog>
  </div>
</template>
