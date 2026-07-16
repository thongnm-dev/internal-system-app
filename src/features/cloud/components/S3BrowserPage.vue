<script setup lang="ts">
import { ref, computed } from "vue";
import { useS3Browser } from "../composables/useS3Browser";
import { useToast } from "@/shared/composables/useToast";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import MessageBanner from "@/shared/components/MessageBanner.vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import type { S3Object } from "@/shared/types/s3";

const ctrl = useS3Browser();
const toast = useToast();
const globalLoading = useGlobalLoading();

const showCreateFolder = ref(false);
const newFolderName = ref("");
const confirmDeleteTarget = ref<{ keys: string[]; label: string } | null>(null);

const isOperating = computed(
  () => ctrl.isLoading.value || ctrl.isUploading.value || ctrl.isDownloading.value || ctrl.isDeleting.value,
);

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
            <button
              class="whitespace-nowrap rounded px-1.5 py-0.5 font-semibold hover:bg-canvas"
              :class="idx === ctrl.breadcrumbs.value.length - 1 ? 'text-brand' : 'text-secondary'"
              @click="ctrl.navigateToBreadcrumb(crumb.prefix)"
            >
              <i v-if="idx === 0" class="pi pi-server mr-1" />
              {{ crumb.label }}
            </button>
          </template>
        </div>
        <!-- Actions -->
        <div class="flex items-center gap-1.5">
          <span v-if="ctrl.selectedCount.value > 0" class="mr-1 text-xs font-bold text-brand">
            {{ ctrl.selectedCount.value }} selected
          </span>
          <button
            class="flex h-8 items-center gap-1.5 rounded-md bg-brand px-3 text-xs font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="isOperating"
            @click="handleUpload()"
          >
            <i :class="ctrl.isUploading.value ? 'pi pi-spinner pi-spin' : 'pi pi-upload'" />
            Upload
          </button>
          <button
            class="flex h-8 items-center gap-1.5 rounded-md border border-divider bg-panel px-3 text-xs font-bold text-secondary hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="isOperating"
            @click="openCreateFolder()"
          >
            <i class="pi pi-folder-plus" /> New Folder
          </button>
          <button
            v-if="ctrl.selectedCount.value > 0"
            class="flex h-8 items-center gap-1.5 rounded-md border border-divider bg-panel px-3 text-xs font-bold text-secondary hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="isOperating"
            @click="handleDownloadSelected()"
          >
            <i :class="ctrl.isDownloading.value ? 'pi pi-spinner pi-spin' : 'pi pi-download'" />
            Download
          </button>
          <button
            v-if="ctrl.selectedCount.value > 0"
            class="flex h-8 items-center gap-1.5 rounded-md bg-red-600 px-3 text-xs font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="isOperating"
            @click="confirmDeleteSelected()"
          >
            <i :class="ctrl.isDeleting.value ? 'pi pi-spinner pi-spin' : 'pi pi-trash'" />
            Delete
          </button>
          <button
            class="flex h-8 w-8 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="isOperating"
            @click="ctrl.refresh()"
          >
            <i :class="ctrl.isLoading.value ? 'pi pi-spinner pi-spin' : 'pi pi-refresh'" />
          </button>
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
            <input
              type="checkbox"
              :checked="ctrl.allSelected.value"
              :indeterminate="ctrl.selectedCount.value > 0 && !ctrl.allSelected.value"
              class="accent-brand"
              @change="ctrl.toggleSelectAll()"
              @click.stop
            />
          </template>
          <template #body="{ data }">
            <input
              type="checkbox"
              :checked="ctrl.selectedKeys.value.has(data.key)"
              class="accent-brand"
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
              <button
                class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-canvas hover:text-ink"
                title="Download"
                @click.stop="ctrl.downloadSingle(data.key)"
              >
                <i class="pi pi-download text-xs" />
              </button>
              <button
                class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-canvas hover:text-red-600"
                title="Delete"
                @click.stop="confirmDeleteSingle(data)"
              >
                <i class="pi pi-trash text-xs" />
              </button>
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
      <button
        class="flex h-10 w-full items-center gap-2 rounded-none px-3 text-left text-sm font-semibold text-secondary hover:bg-canvas"
        @click="contextDownload()"
      >
        <i class="pi pi-download" /> Download
      </button>
      <button
        class="flex h-10 w-full items-center gap-2 rounded-none px-3 text-left text-sm font-semibold text-secondary hover:bg-canvas"
        @click="contextCopyKey()"
      >
        <i class="pi pi-copy" /> Copy Key
      </button>
      <div class="my-1 border-t border-divider" />
      <button
        class="flex h-10 w-full items-center gap-2 rounded-none px-3 text-left text-sm font-semibold text-red-600 hover:bg-canvas"
        @click="contextDelete()"
      >
        <i class="pi pi-trash" /> Delete
      </button>
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
        <input
          v-model="newFolderName"
          class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
          placeholder="new-folder"
          @keyup.enter="submitCreateFolder()"
        />
      </label>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <button
            class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            @click="showCreateFolder = false"
          >
            Cancel
          </button>
          <button
            class="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="!newFolderName.trim()"
            @click="submitCreateFolder()"
          >
            Create
          </button>
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
          <button
            class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            @click="confirmDeleteTarget = null"
          >
            Cancel
          </button>
          <button
            class="h-10 rounded-md bg-red-600 px-4 text-sm font-bold text-white hover:opacity-90"
            @click="executeDelete()"
          >
            Delete
          </button>
        </div>
      </template>
    </Dialog>
  </section>
</template>
