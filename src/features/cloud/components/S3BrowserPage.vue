<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import InputText from "primevue/inputtext";
import { useS3Browser } from "../composables/useS3Browser";
import { useToast } from "@/shared/composables/useToast";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import MessageBanner from "@/shared/components/MessageBanner.vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { S3Object, LocalFileEntry } from "@/_/types/s3";

const ctrl = useS3Browser();
const toast = useToast();
const globalLoading = useGlobalLoading();

const showCreateFolder = ref(false);
const newFolderName = ref("");
const confirmDeleteTarget = ref<{ keys: string[]; label: string } | null>(null);

const isOperating = computed(
  () => ctrl.isLoading.value || ctrl.isUploading.value || ctrl.isDownloading.value || ctrl.isDeleting.value || ctrl.isMoving.value,
);

// --- More Actions Dropdown ---
const showMoreActions = ref(false);

function toggleMoreActions() {
  showMoreActions.value = !showMoreActions.value;
}

function closeMoreActions() {
  showMoreActions.value = false;
}

watch(showMoreActions, (open) => {
  if (open) {
    setTimeout(() => window.addEventListener("click", closeMoreActions), 0);
  } else {
    window.removeEventListener("click", closeMoreActions);
  }
});

// --- Move Dialog ---
const showMoveDialog = ref(false);
const moveKeys = ref<Set<string>>(new Set());
const moveDestPrefix = ref("");
const moveBrowseFolders = ref<S3Object[]>([]);
const isLoadingMoveFolders = ref(false);

const moveBreadcrumbs = computed(() => {
  const parts: { label: string; prefix: string }[] = [{ label: "Root", prefix: "" }];
  if (!moveDestPrefix.value) return parts;
  const segments = moveDestPrefix.value.split("/").filter(Boolean);
  let accumulated = "";
  for (const seg of segments) {
    accumulated += seg + "/";
    parts.push({ label: seg, prefix: accumulated });
  }
  return parts;
});

function openMoveDialog() {
  closeMoreActions();
  const keys = Array.from(ctrl.selectedKeys.value);
  if (keys.length === 0) return;
  moveKeys.value = new Set(keys);
  moveDestPrefix.value = ctrl.currentPrefix.value;
  showMoveDialog.value = true;
  loadMoveFolders(moveDestPrefix.value);
}

function closeMoveDialog() {
  showMoveDialog.value = false;
  moveKeys.value = new Set();
  moveBrowseFolders.value = [];
}

function toggleMoveKey(key: string) {
  const newSet = new Set(moveKeys.value);
  if (newSet.has(key)) {
    newSet.delete(key);
  } else {
    newSet.add(key);
  }
  moveKeys.value = newSet;
}

async function loadMoveFolders(prefix: string) {
  isLoadingMoveFolders.value = true;
  moveBrowseFolders.value = await ctrl.browseFolders(prefix);
  isLoadingMoveFolders.value = false;
}

async function navigateMoveFolder(prefix: string) {
  moveDestPrefix.value = prefix;
  await loadMoveFolders(prefix);
}

async function executeMove() {
  const keys = Array.from(moveKeys.value);
  if (keys.length === 0) return;
  closeMoveDialog();
  const result = await globalLoading.run(() => ctrl.moveObjects(keys, moveDestPrefix.value));
  if (result?.success) toast.success("Moved successfully.");
}

// --- Upload Folder ---
const showUploadFolder = ref(false);
const folderPath = ref("");
const scannedFiles = ref<LocalFileEntry[]>([]);
const isScanning = ref(false);
const isDragOver = ref(false);
let unlistenDrop: (() => void) | null = null;

const folderDisplayName = computed(() => {
  if (!folderPath.value) return "";
  const parts = folderPath.value.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || parts[parts.length - 2] || folderPath.value;
});

async function openUploadFolderDialog() {
  showUploadFolder.value = true;
  folderPath.value = "";
  scannedFiles.value = [];
  try {
    const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop" && showUploadFolder.value) {
        const paths = event.payload.paths;
        if (paths.length > 0) handleDroppedPath(paths[0]);
      }
      if (event.payload.type === "over" && showUploadFolder.value) {
        isDragOver.value = true;
      }
      if (event.payload.type !== "drop" && event.payload.type !== "over" && showUploadFolder.value) {
        isDragOver.value = false;
      }
    });
    unlistenDrop = unlisten;
  } catch {
    // Tauri drag-drop not available, browse button still works
  }
}

function closeUploadFolderDialog() {
  showUploadFolder.value = false;
  folderPath.value = "";
  scannedFiles.value = [];
  isDragOver.value = false;
  if (unlistenDrop) {
    unlistenDrop();
    unlistenDrop = null;
  }
}

onUnmounted(() => {
  if (unlistenDrop) {
    unlistenDrop();
    unlistenDrop = null;
  }
});

async function handleDroppedPath(path: string) {
  isDragOver.value = false;
  folderPath.value = path;
  isScanning.value = true;
  scannedFiles.value = await ctrl.scanLocalFolder(path);
  isScanning.value = false;
}

async function browseFolder() {
  try {
    const dir = await open({ directory: true, title: "Chọn thư mục để tải lên" });
    if (!dir) return;
    await handleDroppedPath(dir as string);
  } catch {
    // User cancelled
  }
}

function clearFolder() {
  folderPath.value = "";
  scannedFiles.value = [];
}

async function executeUploadFolder() {
  if (!folderPath.value || scannedFiles.value.length === 0) return;
  globalLoading.start();
  try {
    const result = await ctrl.uploadFolder(folderPath.value);
    if (result?.success) {
      toast.success(`Uploaded ${result.processed} file(s) successfully.`);
      closeUploadFolderDialog();
    }
  } finally {
    globalLoading.stop();
  }
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

function formatDate(iso: string): string {
  if (!iso) return "—";
  try {
    const d = new Date(iso);
    return d.toLocaleDateString("ja-JP") + " " + d.toLocaleTimeString("ja-JP", { hour: "2-digit", minute: "2-digit" });
  } catch {
    return iso;
  }
}

function handleRowClick(obj: S3Object) {
  if (obj.isFolder) {
    ctrl.navigateToPrefix(obj.key);
  }
}

function openCreateFolder() {
  newFolderName.value = "";
  showCreateFolder.value = true;
}

async function submitCreateFolder() {
  if (!newFolderName.value.trim()) return;
  const result = await globalLoading.run(() => ctrl.createFolder(newFolderName.value));
  showCreateFolder.value = false;
  if (result?.success) toast.success("Folder created successfully.");
}

function confirmDeleteSelected() {
  const keys = Array.from(ctrl.selectedKeys.value);
  if (keys.length === 0) return;
  confirmDeleteTarget.value = {
    keys,
    label: `${keys.length} selected item(s)`,
  };
}

function confirmDeleteSingle(obj: S3Object) {
  confirmDeleteTarget.value = {
    keys: [obj.key],
    label: obj.displayName,
  };
}

async function executeDelete() {
  if (!confirmDeleteTarget.value) return;
  const keys = confirmDeleteTarget.value.keys;
  confirmDeleteTarget.value = null;
  const result = await globalLoading.run(() =>
    keys.length === 1 ? ctrl.deleteSingle(keys[0]) : ctrl.deleteSelected(),
  );
  if (result?.success) toast.success("Deleted successfully.");
}

// --- Context Menu ---
const contextMenu = ref<{ obj: S3Object; x: number; y: number } | null>(null);

function openContextMenu(event: MouseEvent, obj: S3Object) {
  event.preventDefault();
  event.stopPropagation();
  contextMenu.value = { obj, x: event.clientX, y: event.clientY };
  window.addEventListener("click", closeContextMenu);
  window.addEventListener("scroll", closeContextMenu, true);
}

function closeContextMenu() {
  contextMenu.value = null;
  window.removeEventListener("click", closeContextMenu);
  window.removeEventListener("scroll", closeContextMenu, true);
}

async function handleUpload() {
  const result = await globalLoading.run(() => ctrl.uploadFile());
  if (result?.success) toast.success("File uploaded successfully.");
}

async function handleDownloadSelected() {
  const result = await globalLoading.run(() => ctrl.downloadSelected());
  if (result?.success) toast.success("Download completed successfully.");
}

async function contextDownload() {
  if (!contextMenu.value) return;
  const key = contextMenu.value.obj.key;
  closeContextMenu();
  const result = await globalLoading.run(() => ctrl.downloadSingle(key));
  if (result?.success) toast.success("Download completed successfully.");
}

function contextDelete() {
  if (!contextMenu.value) return;
  const obj = contextMenu.value.obj;
  closeContextMenu();
  confirmDeleteSingle(obj);
}

function contextCopyKey() {
  if (!contextMenu.value) return;
  const key = contextMenu.value.obj.key;
  closeContextMenu();
  navigator.clipboard.writeText(key);
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Message -->
    <MessageBanner :message="ctrl.message.value" :mode="ctrl.messageMode.value" />

    <!-- Browser Section -->
    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <!-- Toolbar -->
      <div class="flex items-center justify-between gap-3 border-b border-divider px-4 py-2.5">
        <!-- Breadcrumbs -->
        <div class="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto text-sm">
          <template v-for="(crumb, idx) in ctrl.breadcrumbs.value" :key="crumb.prefix">
            <i v-if="idx > 0" class="pi pi-chevron-right text-[10px] text-muted" />
            <Button
              :class="[
                'whitespace-nowrap rounded px-1.5 py-0.5 font-semibold hover:bg-canvas',
                idx === ctrl.breadcrumbs.value.length - 1 ? 'text-brand' : 'text-secondary',
              ]"
              unstyled
              @click="ctrl.navigateToBreadcrumb(crumb.prefix)"
            >
              <i v-if="idx === 0" class="pi pi-server mr-1" />
              {{ crumb.label }}
            </Button>
          </template>
        </div>
        <!-- Actions -->
        <div class="flex items-center gap-1.5">
          <!-- Connected: show normal actions -->
          <template v-if="ctrl.isConnected.value">
            <span v-if="ctrl.selectedCount.value > 0" class="mr-1 text-xs font-bold text-brand">
              {{ ctrl.selectedCount.value }} selected
            </span>
            <Button
              :icon="ctrl.isUploading.value ? 'pi pi-spinner pi-spin' : 'pi pi-upload'"
              label="Upload"
              size="small"
              :disabled="isOperating"
              @click="handleUpload()"
            />
            <Button icon="pi pi-folder-open" label="Upload Folder" severity="secondary" outlined size="small" :disabled="isOperating" @click="openUploadFolderDialog()" />
            <Button icon="pi pi-folder-plus" label="New Folder" severity="secondary" outlined size="small" :disabled="isOperating" @click="openCreateFolder()" />
            <Button
              v-if="ctrl.selectedCount.value > 0"
              :icon="ctrl.isDownloading.value ? 'pi pi-spinner pi-spin' : 'pi pi-download'"
              label="Download"
              severity="secondary"
              outlined
              size="small"
              :disabled="isOperating"
              @click="handleDownloadSelected()"
            />
            <div v-if="ctrl.selectedCount.value > 0" class="relative">
              <Button icon="pi pi-ellipsis-v" label="Actions" severity="secondary" outlined size="small" :disabled="isOperating" @click="toggleMoreActions()" />
              <div
                v-if="showMoreActions"
                class="absolute right-0 top-full z-50 mt-1 min-w-40 overflow-hidden rounded-md border border-divider bg-panel py-1 shadow-xl"
                @click.stop
              >
                <Button icon="pi pi-arrow-right-arrow-left" label="Move" text size="small" class="w-full justify-start" @click="openMoveDialog()" />
                <div class="my-0.5 border-t border-divider" />
                <Button icon="pi pi-trash" label="Delete" text severity="danger" size="small" class="w-full justify-start" @click="closeMoreActions(); confirmDeleteSelected()" />
              </div>
            </div>
            <Button :icon="ctrl.isLoading.value ? 'pi pi-spinner pi-spin' : 'pi pi-refresh'" severity="secondary" outlined size="small" :disabled="isOperating" @click="ctrl.refresh()" />
          </template>
          <!-- Disconnected: show test connection button -->
          <Button
            v-else
            :icon="ctrl.isTesting.value ? 'pi pi-spinner pi-spin' : 'pi pi-link'"
            label="Kiểm tra kết nối"
            severity="secondary"
            outlined
            size="small"
            :disabled="ctrl.isTesting.value"
            @click="ctrl.connect()"
          />
        </div>
      </div>

      <!-- DataTable -->
      <DataTable
        class="app-data-table min-h-0"
        :value="ctrl.objects.value"
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '700px' }"
        empty-message="This folder is empty."
        @row-click="(e: any) => handleRowClick(e.data)"
      >
        <Column header="" style="width: 40px" :sortable="false">
          <template #header>
            <Checkbox
              :model-value="ctrl.allSelected.value"
              :indeterminate="ctrl.selectedCount.value > 0 && !ctrl.allSelected.value"
              binary
              @change="ctrl.toggleSelectAll()"
              @click.stop
            />
          </template>
          <template #body="{ data }">
            <Checkbox
              :model-value="ctrl.selectedKeys.value.has(data.key)"
              binary
              @change="ctrl.toggleSelect(data.key)"
              @click.stop
            />
          </template>
        </Column>
        <Column header="Name" field="displayName" :sortable="false" style="min-width: 300px">
          <template #body="{ data }">
            <div
              class="flex items-center gap-2"
              :class="data.isFolder ? 'cursor-pointer font-semibold text-brand' : 'text-ink'"
              @contextmenu="openContextMenu($event, data)"
            >
              <i :class="data.isFolder ? 'pi pi-folder text-amber-500' : 'pi pi-file text-muted'" />
              {{ data.displayName }}
            </div>
          </template>
        </Column>
        <Column header="Size" field="size" :sortable="false" style="width: 120px" header-class="num" body-class="num text-muted">
          <template #body="{ data }">
            {{ formatSize(data.size) }}
          </template>
        </Column>
        <Column header="Last Modified" field="lastModified" :sortable="false" style="width: 180px" body-class="whitespace-nowrap text-muted">
          <template #body="{ data }">
            {{ formatDate(data.lastModified) }}
          </template>
        </Column>
        <Column header="" :sortable="false" style="width: 80px">
          <template #body="{ data }">
            <div class="flex items-center justify-center gap-1">
              <Button icon="pi pi-download" severity="secondary" text rounded size="small" title="Download" @click.stop="ctrl.downloadSingle(data.key)" />
              <Button icon="pi pi-trash" severity="danger" text rounded size="small" title="Delete" @click.stop="confirmDeleteSingle(data)" />
            </div>
          </template>
        </Column>
      </DataTable>
    </section>

    <!-- Context Menu -->
    <div
      v-if="contextMenu"
      class="fixed z-50 min-w-48 overflow-hidden rounded-md border border-divider bg-panel py-1 text-sm shadow-xl"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      @click.stop
      @contextmenu.prevent
    >
      <Button icon="pi pi-download" label="Download" text size="small" class="w-full justify-start" @click="contextDownload()" />
      <Button icon="pi pi-copy" label="Copy Key" text size="small" class="w-full justify-start" @click="contextCopyKey()" />
      <div class="my-1 border-t border-divider" />
      <Button icon="pi pi-trash" label="Delete" text severity="danger" size="small" class="w-full justify-start" @click="contextDelete()" />
    </div>

    <!-- Create Folder Dialog -->
    <Dialog
      :visible="showCreateFolder"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="showCreateFolder = $event"
    >
      <template #header>
        <h3 class="font-bold text-ink">Create Folder</h3>
      </template>
      <label class="block">
        <span class="text-xs font-bold text-muted">Folder Name</span>
        <InputText
          v-model="newFolderName"
          class="mt-1 w-full"
          placeholder="new-folder"
          @keyup.enter="submitCreateFolder()"
        />
      </label>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" outlined @click="showCreateFolder = false" />
          <Button label="Create" :disabled="!newFolderName.trim()" @click="submitCreateFolder()" />
        </div>
      </template>
    </Dialog>

    <!-- Delete Confirmation Dialog -->
    <Dialog
      :visible="confirmDeleteTarget !== null"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="confirmDeleteTarget = $event ? confirmDeleteTarget : null"
    >
      <template #header>
        <h3 class="font-bold text-ink">Confirm Delete</h3>
      </template>
      <p class="text-sm text-secondary">
        Are you sure you want to delete <strong>{{ confirmDeleteTarget?.label }}</strong>? This action cannot be undone.
      </p>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" outlined @click="confirmDeleteTarget = null" />
          <Button label="Delete" severity="danger" @click="executeDelete()" />
        </div>
      </template>
    </Dialog>

    <!-- Upload Folder Dialog -->
    <Dialog
      :visible="showUploadFolder"
      class="w-full max-w-lg rounded-lg bg-panel shadow-xl"
      :closable="!ctrl.isUploading.value"
      modal
      @update:visible="(v: boolean) => { if (!v) closeUploadFolderDialog(); }"
    >
      <template #header>
        <h3 class="font-bold text-ink">Upload Folder</h3>
      </template>

      <!-- Drop Zone (no folder selected yet) -->
      <div
        v-if="!folderPath"
        class="flex flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed px-8 py-10 transition-colors"
        :class="isDragOver ? 'border-brand bg-brand/5' : 'border-divider'"
      >
        <i class="pi pi-cloud-upload text-4xl text-muted" />
        <p class="text-sm font-semibold text-secondary">Kéo thả thư mục vào đây</p>
        <span class="text-xs text-muted">hoặc</span>
        <Button icon="pi pi-folder-open" label="Chọn thư mục" severity="secondary" outlined size="small" @click="browseFolder()" />
      </div>

      <!-- Folder selected: show preview -->
      <div v-else class="flex flex-col gap-3">
        <div class="flex items-center gap-2 rounded-md bg-canvas px-3 py-2">
          <i class="pi pi-folder text-amber-500" />
          <span class="min-w-0 flex-1 truncate text-sm font-semibold text-ink">{{ folderDisplayName }}</span>
          <Button v-if="!ctrl.isUploading.value" icon="pi pi-times" text rounded size="small" @click="clearFolder()" />
        </div>

        <!-- Scanning spinner -->
        <div v-if="isScanning" class="flex items-center justify-center gap-2 py-4">
          <i class="pi pi-spinner pi-spin text-brand" />
          <span class="text-xs text-muted">Scanning files...</span>
        </div>

        <!-- File list -->
        <div v-else-if="scannedFiles.length > 0" class="max-h-60 overflow-y-auto rounded-md border border-divider">
          <div
            v-for="file in scannedFiles"
            :key="file.fullPath"
            class="flex items-center gap-2 border-b border-divider px-3 py-1.5 last:border-b-0"
          >
            <i class="pi pi-file text-xs text-muted" />
            <span class="min-w-0 flex-1 truncate text-xs text-secondary">{{ file.relativePath }}</span>
            <span class="whitespace-nowrap text-xs text-muted">{{ formatSize(file.size) }}</span>
          </div>
        </div>

        <!-- Empty folder -->
        <div v-else class="py-4 text-center text-xs text-muted">Thư mục không chứa tập tin nào.</div>

        <p v-if="scannedFiles.length > 0" class="text-xs text-muted">{{ scannedFiles.length }} tập tin</p>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" outlined :disabled="ctrl.isUploading.value" @click="closeUploadFolderDialog()" />
          <Button
            :icon="ctrl.isUploading.value ? 'pi pi-spinner pi-spin' : undefined"
            label="Upload"
            :disabled="!folderPath || scannedFiles.length === 0 || ctrl.isUploading.value || isScanning"
            @click="executeUploadFolder()"
          />
        </div>
      </template>
    </Dialog>

    <!-- Move Dialog -->
    <Dialog
      :visible="showMoveDialog"
      class="rounded-lg bg-panel shadow-xl"
      :style="{ width: '50vw' }"
      :closable="!ctrl.isMoving.value"
      modal
      @update:visible="(v: boolean) => { if (!v) closeMoveDialog(); }"
    >
      <template #header>
        <h3 class="font-bold text-ink">Move Items</h3>
      </template>

      <div class="flex flex-col gap-4">
        <!-- Selected items -->
        <div>
          <span class="text-xs font-bold text-muted">Items to move</span>
          <div class="mt-1 max-h-36 overflow-y-auto rounded-md border border-divider">
            <div
              v-for="key in ctrl.selectedKeys.value"
              :key="key"
              class="flex items-center gap-2 border-b border-divider px-3 py-1.5 last:border-b-0"
            >
              <Checkbox
                :model-value="moveKeys.has(key)"
                binary
                @change="toggleMoveKey(key)"
              />
              <i :class="key.endsWith('/') ? 'pi pi-folder text-amber-500 text-xs' : 'pi pi-file text-muted text-xs'" />
              <span class="min-w-0 flex-1 truncate text-xs text-secondary">
                {{ key.endsWith('/') ? key.slice(0, -1).split('/').pop() : key.split('/').pop() }}
              </span>
            </div>
          </div>
        </div>

        <!-- Destination browser -->
        <div>
          <span class="text-xs font-bold text-muted">Destination</span>
          <!-- Breadcrumbs -->
          <div class="mt-1 flex items-center gap-1 overflow-x-auto text-xs">
            <template v-for="(crumb, idx) in moveBreadcrumbs" :key="crumb.prefix">
              <i v-if="idx > 0" class="pi pi-chevron-right text-[8px] text-muted" />
              <Button
                :class="[
                  'whitespace-nowrap rounded px-1.5 py-0.5 font-semibold hover:bg-canvas',
                  idx === moveBreadcrumbs.length - 1 ? 'text-brand' : 'text-secondary',
                ]"
                unstyled
                @click="navigateMoveFolder(crumb.prefix)"
              >
                <i v-if="idx === 0" class="pi pi-server mr-0.5" />
                {{ crumb.label }}
              </Button>
            </template>
          </div>
          <!-- Folder list -->
          <div class="mt-1 max-h-44 min-h-[80px] overflow-y-auto rounded-md border border-divider">
            <div v-if="isLoadingMoveFolders" class="flex items-center justify-center gap-2 py-6">
              <i class="pi pi-spinner pi-spin text-brand" />
              <span class="text-xs text-muted">Loading...</span>
            </div>
            <div v-else-if="moveBrowseFolders.length === 0" class="py-6 text-center text-xs text-muted">
              No subfolders
            </div>
            <template v-else>
              <Button
                v-for="folder in moveBrowseFolders"
                :key="folder.key"
                class="flex h-9 w-full items-center gap-2 border-b border-divider px-3 text-left last:border-b-0 hover:bg-canvas"
                unstyled
                @click="navigateMoveFolder(folder.key)"
              >
                <i class="pi pi-folder text-amber-500" />
                <span class="min-w-0 flex-1 truncate text-xs font-semibold text-ink">{{ folder.displayName }}</span>
                <i class="pi pi-chevron-right text-[10px] text-muted" />
              </Button>
            </template>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" outlined :disabled="ctrl.isMoving.value" @click="closeMoveDialog()" />
          <Button
            :icon="ctrl.isMoving.value ? 'pi pi-spinner pi-spin' : undefined"
            label="Move"
            :disabled="moveKeys.size === 0 || ctrl.isMoving.value"
            @click="executeMove()"
          />
        </div>
      </template>
    </Dialog>

    <!-- Offline Dialog -->
    <Dialog
      v-model:visible="ctrl.showOfflineDialog.value"
      header="Lỗi kết nối"
      :modal="true"
      :closable="true"
      :style="{ width: '28rem' }"
    >
      <div class="flex items-center gap-3">
        <i class="pi pi-wifi text-3xl text-red-500" />
        <span class="text-sm text-secondary">{{ ctrl.offlineMessage }}</span>
      </div>
      <template #footer>
        <Button label="Đóng" @click="ctrl.dismissOfflineDialog()" />
      </template>
    </Dialog>
  </section>
</template>
