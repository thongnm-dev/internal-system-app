<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed, nextTick } from "vue";
import { useExploreFaster } from "../composables/useExploreFaster";
import type { FileEntry } from "@/tauri/commands/explorer";

const ctrl = useExploreFaster();
const pathInput = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const pathInputEl = ref<HTMLInputElement | null>(null);
const isEditingPath = ref(false);

function startEditPath() {
  pathInput.value = ctrl.currentPath.value;
  isEditingPath.value = true;
  nextTick(() => {
    pathInputEl.value?.focus();
    pathInputEl.value?.select();
  });
}

function commitPath() {
  isEditingPath.value = false;
  const trimmed = pathInput.value.trim();
  if (trimmed && trimmed !== ctrl.currentPath.value) {
    ctrl.handlePathSubmit(trimmed).then(() => {
      pathInput.value = ctrl.currentPath.value;
    });
  }
}

function cancelEditPath() {
  isEditingPath.value = false;
  pathInput.value = ctrl.currentPath.value;
}

function handlePathEditKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    commitPath();
  } else if (e.key === "Escape") {
    cancelEditPath();
  }
}

// --- Selection ---
const selectedPaths = ref<Set<string>>(new Set());

const isAllSelected = computed(() => {
  return sortedEntries.value.length > 0 && sortedEntries.value.every(e => selectedPaths.value.has(e.path));
});

const isSomeSelected = computed(() => {
  return !isAllSelected.value && sortedEntries.value.some(e => selectedPaths.value.has(e.path));
});

function toggleSelectAll() {
  if (isAllSelected.value) {
    selectedPaths.value = new Set();
  } else {
    selectedPaths.value = new Set(sortedEntries.value.map(e => e.path));
  }
}

function toggleSelect(entry: FileEntry) {
  const newSet = new Set(selectedPaths.value);
  if (newSet.has(entry.path)) {
    newSet.delete(entry.path);
  } else {
    newSet.add(entry.path);
  }
  selectedPaths.value = newSet;
}

// --- Resizable sidebar ---
const sidebarWidth = ref(176);
const isResizing = ref(false);
const sidebarRef = ref<HTMLElement | null>(null);

function startResize(e: MouseEvent) {
  e.preventDefault();
  isResizing.value = true;
  document.addEventListener("mousemove", onResize);
  document.addEventListener("mouseup", stopResize);
}

function onResize(e: MouseEvent) {
  if (!isResizing.value) return;
  const containerLeft = sidebarRef.value?.parentElement?.getBoundingClientRect().left ?? 0;
  const newWidth = e.clientX - containerLeft;
  sidebarWidth.value = Math.max(120, Math.min(400, newWidth));
}

function stopResize() {
  isResizing.value = false;
  document.removeEventListener("mousemove", onResize);
  document.removeEventListener("mouseup", stopResize);
}

onUnmounted(() => {
  document.removeEventListener("mousemove", onResize);
  document.removeEventListener("mouseup", stopResize);
});

// --- Sort ---
type SortKey = "name" | "extension" | "size" | "modified";
type SortDir = "asc" | "desc";
const sortKey = ref<SortKey>("name");
const sortDir = ref<SortDir>("asc");

function toggleSort(key: SortKey) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === "asc" ? "desc" : "asc";
  } else {
    sortKey.value = key;
    sortDir.value = "asc";
  }
}

function sortIcon(key: SortKey): string {
  if (sortKey.value !== key) return "pi pi-sort-alt text-[10px] opacity-0 group-hover/th:opacity-50";
  return sortDir.value === "asc"
    ? "pi pi-sort-amount-up-alt text-[10px] text-brand"
    : "pi pi-sort-amount-down text-[10px] text-brand";
}

const rawEntries = computed(() => {
  return ctrl.isSearchMode.value ? ctrl.searchResults.value : ctrl.entries.value;
});

const sortedEntries = computed(() => {
  const list = [...rawEntries.value];
  const dir = sortDir.value === "asc" ? 1 : -1;
  const key = sortKey.value;

  list.sort((a, b) => {
    if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;

    if (key === "name") {
      return dir * a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
    }
    if (key === "extension") {
      return dir * a.extension.localeCompare(b.extension, undefined, { sensitivity: "base" });
    }
    if (key === "size") {
      return dir * (a.size - b.size);
    }
    if (key === "modified") {
      return dir * a.modified.localeCompare(b.modified);
    }
    return 0;
  });
  return list;
});

// --- Lifecycle ---
onMounted(async () => {
  await ctrl.loadDrives();
  pathInput.value = ctrl.currentPath.value;
});


function onSearchInput() {
  ctrl.searchQuery.value = (searchInput.value?.value ?? "").toString();
  ctrl.triggerSearch();
}

function onSearchKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    ctrl.clearSearch();
    if (searchInput.value) searchInput.value.value = "";
  }
}

function handleEntryClick(entry: FileEntry) {
  ctrl.openEntry(entry).then(() => {
    pathInput.value = ctrl.currentPath.value;
  });
}

function handleBreadcrumbClick(path: string) {
  ctrl.navigateTo(path).then(() => {
    pathInput.value = ctrl.currentPath.value;
  });
}

function handleDriveClick(drive: string) {
  ctrl.navigateTo(drive).then(() => {
    pathInput.value = ctrl.currentPath.value;
  });
}

function handleGoBack() {
  ctrl.goBack();
  setTimeout(() => (pathInput.value = ctrl.currentPath.value), 50);
}

function handleGoForward() {
  ctrl.goForward();
  setTimeout(() => (pathInput.value = ctrl.currentPath.value), 50);
}

function handleGoUp() {
  ctrl.goUp().then(() => {
    pathInput.value = ctrl.currentPath.value;
  });
}

function handleRefresh() {
  ctrl.refresh();
}

function fileIcon(entry: FileEntry): string {
  if (entry.is_dir) return "pi pi-folder text-amber-500";
  const ext = entry.extension;
  if (["xlsx", "xls", "xlsm", "csv"].includes(ext)) return "pi pi-file-excel text-green-600";
  if (["doc", "docx"].includes(ext)) return "pi pi-file-word text-blue-600";
  if (["pdf"].includes(ext)) return "pi pi-file-pdf text-red-600";
  if (["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp"].includes(ext)) return "pi pi-image text-purple-500";
  if (["zip", "rar", "7z", "tar", "gz"].includes(ext)) return "pi pi-box text-orange-500";
  if (["txt", "log", "md"].includes(ext)) return "pi pi-file-edit text-muted";
  if (["js", "ts", "vue", "jsx", "tsx", "py", "rs", "java", "cs", "cpp", "c", "h"].includes(ext)) return "pi pi-code text-cyan-500";
  return "pi pi-file text-muted";
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

function parentPath(fullPath: string): string {
  const normalized = fullPath.replace(/\//g, "\\");
  const lastSep = normalized.lastIndexOf("\\");
  if (lastSep <= 0) return "";
  return normalized.substring(0, lastSep);
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-3 overflow-hidden">
    <!-- Navigation bar -->
    <div class="flex items-center gap-2 rounded-lg border border-divider bg-panel px-3 py-2 shadow-sm">
      <!-- Nav buttons -->
      <button
        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-secondary hover:bg-canvas disabled:opacity-40 disabled:hover:bg-transparent"
        :disabled="!ctrl.canGoBack.value"
        title="Back"
        @click="handleGoBack"
      >
        <i class="pi pi-arrow-left text-sm" />
      </button>
      <button
        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-secondary hover:bg-canvas disabled:opacity-40 disabled:hover:bg-transparent"
        :disabled="!ctrl.canGoForward.value"
        title="Forward"
        @click="handleGoForward"
      >
        <i class="pi pi-arrow-right text-sm" />
      </button>
      <button
        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-secondary hover:bg-canvas"
        title="Up"
        @click="handleGoUp"
      >
        <i class="pi pi-arrow-up text-sm" />
      </button>
      <button
        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-secondary hover:bg-canvas"
        title="Refresh"
        @click="handleRefresh"
      >
        <i :class="ctrl.isLoading.value ? 'pi pi-spinner pi-spin text-sm' : 'pi pi-refresh text-sm'" />
      </button>

      <!-- Path bar: breadcrumb / edit mode -->
      <div class="relative min-w-0 flex-1">
        <!-- Edit mode -->
        <div v-if="isEditingPath" class="relative">
          <i class="pi pi-folder pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-xs text-muted" />
          <input
            ref="pathInputEl"
            v-model="pathInput"
            class="h-9 w-full rounded-md border border-brand bg-canvas pl-8 pr-3 text-sm outline-none ring-2 ring-emerald-100"
            placeholder="Enter path... (e.g. D:\Projects)"
            @keydown="handlePathEditKeydown"
            @blur="commitPath"
          />
        </div>
        <!-- Breadcrumb mode -->
        <div
          v-else
          class="flex h-9 w-full cursor-text items-center gap-0.5 overflow-x-auto rounded-md border border-divider bg-canvas px-2 text-sm"
          @click="startEditPath"
        >
          <template v-for="(crumb, idx) in ctrl.breadcrumbs.value" :key="crumb.path">
            <i v-if="idx > 0" class="pi pi-chevron-right shrink-0 text-[8px] text-muted" />
            <button
              class="shrink-0 whitespace-nowrap rounded px-1.5 py-0.5 transition-colors hover:bg-brand/10 hover:text-brand"
              :class="idx === ctrl.breadcrumbs.value.length - 1 ? 'font-semibold text-brand' : 'text-secondary'"
              @click.stop="handleBreadcrumbClick(crumb.path)"
            >
              <i v-if="idx === 0" class="pi pi-desktop mr-1 text-xs" />
              {{ crumb.label }}
            </button>
          </template>
        </div>
      </div>

      <!-- Search bar -->
      <div class="relative w-56 shrink-0">
        <i class="pi pi-search pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-xs text-muted" />
        <input
          ref="searchInput"
          class="h-9 w-full rounded-md border border-divider bg-canvas pl-8 pr-8 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
          placeholder="Search files..."
          @input="onSearchInput"
          @keydown="onSearchKeydown"
        />
        <button
          v-if="ctrl.searchQuery.value"
          class="absolute right-2 top-1/2 -translate-y-1/2 text-muted hover:text-ink"
          @click="ctrl.clearSearch(); if (searchInput) searchInput.value = ''"
        >
          <i class="pi pi-times text-xs" />
        </button>
        <i
          v-if="ctrl.isSearching.value"
          class="pi pi-spinner pi-spin absolute right-2 top-1/2 -translate-y-1/2 text-xs text-brand"
        />
      </div>
    </div>

    <!-- Error message -->
    <div
      v-if="ctrl.error.value"
      class="flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 px-4 py-2.5 text-sm text-red-700"
    >
      <i class="pi pi-exclamation-triangle" />
      {{ ctrl.error.value }}
    </div>

    <!-- Search status bar -->
    <div
      v-if="ctrl.isSearchMode.value"
      class="flex items-center justify-between rounded-lg border border-divider bg-panel px-4 py-2 shadow-sm"
    >
      <span class="text-sm text-secondary">
        <i class="pi pi-search mr-1 text-brand" />
        Search results for "<strong class="text-ink">{{ ctrl.searchQuery.value }}</strong>"
        in <strong class="text-ink">{{ ctrl.currentPath.value }}</strong>
        <span class="ml-2 text-muted">({{ ctrl.searchResults.value.length }} items{{ ctrl.searchTruncated.value ? ', showing first 500' : '' }})</span>
      </span>
      <button
        class="text-xs font-bold text-brand hover:underline"
        @click="ctrl.clearSearch(); if (searchInput) searchInput.value = ''"
      >
        Clear Search
      </button>
    </div>

    <!-- Main content -->
    <div class="flex min-h-0 flex-1 overflow-hidden" :class="isResizing ? 'select-none' : ''">
      <!-- Sidebar: Drives -->
      <aside
        ref="sidebarRef"
        class="flex shrink-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm"
        :style="{ width: sidebarWidth + 'px' }"
      >
        <div class="border-b border-divider px-3 py-2">
          <span class="text-xs font-bold uppercase tracking-wide text-muted">Drives</span>
        </div>
        <div class="flex-1 overflow-y-auto p-1.5">
          <button
            v-for="drive in ctrl.drives.value"
            :key="drive"
            class="flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm transition-colors hover:bg-canvas"
            :class="ctrl.currentPath.value.startsWith(drive) ? 'bg-canvas font-semibold text-brand' : 'text-secondary'"
            @click="handleDriveClick(drive)"
          >
            <i class="pi pi-desktop text-xs" />
            {{ drive }}
          </button>
        </div>

        <!-- Quick Breadcrumbs -->
        <div class="border-t border-divider p-1.5">
          <div class="max-h-40 overflow-y-auto">
            <button
              v-for="crumb in ctrl.breadcrumbs.value"
              :key="crumb.path"
              class="flex w-full items-center gap-1.5 rounded-md px-2.5 py-1.5 text-left text-xs transition-colors hover:bg-canvas"
              :class="crumb.path === ctrl.currentPath.value ? 'font-semibold text-brand' : 'text-muted'"
              @click="handleBreadcrumbClick(crumb.path)"
            >
              <i class="pi pi-chevron-right text-[8px]" />
              <span class="truncate">{{ crumb.label }}</span>
            </button>
          </div>
        </div>
      </aside>

      <!-- Resize handle -->
      <div
        class="flex w-2 shrink-0 cursor-col-resize items-center justify-center hover:bg-brand/10"
        :class="isResizing ? 'bg-brand/20' : ''"
        @mousedown="startResize"
      >
        <div class="h-8 w-0.5 rounded-full bg-divider" :class="isResizing ? 'bg-brand' : ''" />
      </div>

      <!-- File list -->
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
        <!-- Table header -->
        <div class="grid grid-cols-[32px_minmax(0,1fr)_80px_100px_160px_80px] items-center gap-2 border-b border-divider bg-canvas px-4 py-2 text-xs font-bold text-ink">
          <label class="flex items-center justify-center">
            <input
              type="checkbox"
              class="accent-brand h-3.5 w-3.5 cursor-pointer rounded"
              :checked="isAllSelected"
              :indeterminate="isSomeSelected"
              @change="toggleSelectAll"
            />
          </label>
          <button
            class="group/th flex items-center gap-1.5 text-left"
            @click="toggleSort('name')"
          >
            Name
            <i :class="sortIcon('name')" />
          </button>
          <button
            class="group/th flex items-center gap-1.5 text-left"
            @click="toggleSort('extension')"
          >
            Ext
            <i :class="sortIcon('extension')" />
          </button>
          <button
            class="group/th flex items-center justify-end gap-1.5"
            @click="toggleSort('size')"
          >
            Size
            <i :class="sortIcon('size')" />
          </button>
          <button
            class="group/th flex items-center gap-1.5 text-left"
            @click="toggleSort('modified')"
          >
            Modified
            <i :class="sortIcon('modified')" />
          </button>
          <span class="text-center">Actions</span>
        </div>

        <!-- Loading -->
        <div v-if="ctrl.isLoading.value && !ctrl.isSearchMode.value" class="flex flex-1 items-center justify-center">
          <div class="flex items-center gap-2 text-sm text-muted">
            <i class="pi pi-spinner pi-spin" />
            Loading...
          </div>
        </div>

        <!-- Empty state -->
        <div
          v-else-if="sortedEntries.length === 0 && !ctrl.isLoading.value"
          class="flex flex-1 items-center justify-center text-sm text-muted"
        >
          <div class="text-center">
            <i class="pi pi-folder-open mb-2 text-3xl opacity-40" />
            <p v-if="ctrl.isSearchMode.value">No files found matching your search.</p>
            <p v-else>This folder is empty.</p>
          </div>
        </div>

        <!-- Entries -->
        <div v-else class="min-h-0 flex-1 overflow-y-auto">
          <div
            v-for="entry in sortedEntries"
            :key="entry.path"
            class="group grid grid-cols-[32px_minmax(0,1fr)_80px_100px_160px_80px] items-center gap-2 border-b border-divider px-4 py-1.5 text-sm transition-colors hover:bg-canvas"
            :class="[entry.is_dir ? 'cursor-pointer' : '', selectedPaths.has(entry.path) ? 'bg-brand/5' : '']"
            @dblclick="handleEntryClick(entry)"
          >
            <!-- Checkbox -->
            <label class="flex items-center justify-center" @click.stop>
              <input
                type="checkbox"
                class="accent-brand h-3.5 w-3.5 cursor-pointer rounded"
                :checked="selectedPaths.has(entry.path)"
                @change="toggleSelect(entry)"
              />
            </label>

            <!-- Name -->
            <div class="flex min-w-0 items-center gap-2">
              <i :class="fileIcon(entry)" class="shrink-0 text-sm" />
              <div class="min-w-0">
                <span
                  class="block truncate"
                  :class="entry.is_dir ? 'font-semibold text-brand' : 'text-ink'"
                  :title="entry.name"
                >
                  {{ entry.name }}
                </span>
                <span
                  v-if="ctrl.isSearchMode.value"
                  class="block truncate text-xs text-muted"
                  :title="parentPath(entry.path)"
                >
                  {{ parentPath(entry.path) }}
                </span>
              </div>
            </div>

            <!-- Extension -->
            <span class="truncate text-xs text-muted">
              {{ entry.is_dir ? '' : entry.extension }}
            </span>

            <!-- Size -->
            <span class="text-right text-xs tabular-nums text-muted">
              {{ entry.is_dir ? '' : formatSize(entry.size) }}
            </span>

            <!-- Modified -->
            <span class="truncate text-xs text-muted">
              {{ entry.modified }}
            </span>

            <!-- Actions -->
            <div class="flex items-center justify-center gap-0.5 opacity-0 group-hover:opacity-100">
              <button
                v-if="entry.is_dir"
                class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-panel hover:text-brand"
                title="Open folder"
                @click.stop="handleEntryClick(entry)"
              >
                <i class="pi pi-folder-open text-xs" />
              </button>
              <button
                class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-panel hover:text-brand"
                title="Show in Explorer"
                @click.stop="ctrl.openInExplorer(entry.path)"
              >
                <i class="pi pi-external-link text-xs" />
              </button>
            </div>
          </div>
        </div>

        <!-- Footer status -->
        <div class="flex items-center justify-between border-t border-divider px-4 py-1.5 text-xs text-muted">
          <span>
            {{ sortedEntries.length }} items
            <template v-if="selectedPaths.size > 0">
              <span class="ml-1 text-brand">({{ selectedPaths.size }} selected)</span>
            </template>
          </span>
          <span v-if="!ctrl.isSearchMode.value">{{ ctrl.currentPath.value }}</span>
          <span v-if="ctrl.searchTruncated.value" class="text-amber-600">Results truncated to 500 items</span>
        </div>
      </div>
    </div>
  </section>
</template>
