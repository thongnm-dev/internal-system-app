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

// --- Clipboard ---
const clipboard = ref<{ paths: string[]; cut: boolean } | null>(null);

function clipboardCut(paths: string[]) {
  clipboard.value = { paths, cut: true };
}

function clipboardCopy(paths: string[]) {
  clipboard.value = { paths, cut: false };
}

async function clipboardPaste() {
  if (!clipboard.value) return;
  await ctrl.pasteEntries(clipboard.value.paths, clipboard.value.cut);
  if (clipboard.value.cut) clipboard.value = null;
}

// --- Context Menu ---
type CtxMenuType = "entry" | "background";
const ctxMenu = ref<{ type: CtxMenuType; x: number; y: number; entry?: FileEntry } | null>(null);
const ctxSubmenu = ref<string | null>(null);

function openEntryMenu(e: MouseEvent, entry: FileEntry) {
  e.preventDefault();
  e.stopPropagation();
  ctxSubmenu.value = null;
  ctxMenu.value = { type: "entry", x: e.clientX, y: e.clientY, entry };
  requestAnimationFrame(() => {
    window.addEventListener("click", closeCtxMenu);
    window.addEventListener("contextmenu", closeCtxMenu);
    window.addEventListener("scroll", closeCtxMenu, true);
  });
}

function openBgMenu(e: MouseEvent) {
  e.preventDefault();
  ctxSubmenu.value = null;
  ctxMenu.value = { type: "background", x: e.clientX, y: e.clientY };
  requestAnimationFrame(() => {
    window.addEventListener("click", closeCtxMenu);
    window.addEventListener("contextmenu", closeCtxMenu);
    window.addEventListener("scroll", closeCtxMenu, true);
  });
}

function closeCtxMenu() {
  ctxMenu.value = null;
  ctxSubmenu.value = null;
  window.removeEventListener("click", closeCtxMenu);
  window.removeEventListener("contextmenu", closeCtxMenu);
  window.removeEventListener("scroll", closeCtxMenu, true);
}

function toggleSubmenu(name: string) {
  ctxSubmenu.value = ctxSubmenu.value === name ? null : name;
}

// --- Context actions: entry ---
function ctxCut() {
  if (!ctxMenu.value?.entry) return;
  clipboardCut([ctxMenu.value.entry.path]);
  closeCtxMenu();
}

function ctxCopy() {
  if (!ctxMenu.value?.entry) return;
  clipboardCopy([ctxMenu.value.entry.path]);
  closeCtxMenu();
}

function ctxDelete() {
  if (!ctxMenu.value?.entry) return;
  confirmDeleteTarget.value = [ctxMenu.value.entry.path];
  confirmDeleteLabel.value = ctxMenu.value.entry.name;
  closeCtxMenu();
}

function ctxRename() {
  if (!ctxMenu.value?.entry) return;
  startRename(ctxMenu.value.entry);
  closeCtxMenu();
}

// --- Context actions: background ---
function ctxNewFile() {
  closeCtxMenu();
  startNewEntry("file");
}

function ctxNewFolder() {
  closeCtxMenu();
  startNewEntry("folder");
}

async function ctxPaste() {
  closeCtxMenu();
  await clipboardPaste();
}

function ctxSortBy(key: SortKey) {
  toggleSort(key);
  closeCtxMenu();
}

function ctxGroupBy(key: string) {
  groupBy.value = groupBy.value === key ? "" : key;
  closeCtxMenu();
}

function ctxRefresh() {
  closeCtxMenu();
  handleRefresh();
}

// --- Delete confirmation ---
const confirmDeleteTarget = ref<string[] | null>(null);
const confirmDeleteLabel = ref("");

function deleteSelected() {
  const paths = Array.from(selectedPaths.value);
  if (paths.length === 0) return;
  confirmDeleteTarget.value = paths;
  confirmDeleteLabel.value = `${paths.length} item(s)`;
}

async function executeDelete() {
  if (!confirmDeleteTarget.value) return;
  const paths = confirmDeleteTarget.value;
  confirmDeleteTarget.value = null;
  await ctrl.deleteEntries(paths);
  const newSel = new Set(selectedPaths.value);
  paths.forEach(p => newSel.delete(p));
  selectedPaths.value = newSel;
}

// --- Inline rename ---
const renamingPath = ref("");
const renameValue = ref("");
const renameInputEl = ref<HTMLInputElement | null>(null);

function startRename(entry: FileEntry) {
  renamingPath.value = entry.path;
  renameValue.value = entry.name;
  nextTick(() => {
    renameInputEl.value?.focus();
    const dotIdx = entry.name.lastIndexOf(".");
    if (!entry.is_dir && dotIdx > 0) {
      renameInputEl.value?.setSelectionRange(0, dotIdx);
    } else {
      renameInputEl.value?.select();
    }
  });
}

async function commitRename() {
  const path = renamingPath.value;
  const newName = renameValue.value.trim();
  renamingPath.value = "";
  if (!newName || !path) return;
  const oldName = path.split(/[/\\]/).pop() || "";
  if (newName !== oldName) {
    await ctrl.renameEntry(path, newName);
  }
}

function cancelRename() {
  renamingPath.value = "";
}

function handleRenameKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    commitRename();
  } else if (e.key === "Escape") {
    cancelRename();
  }
}

// --- Inline new file/folder ---
const newEntryMode = ref<"file" | "folder" | "">("");
const newEntryName = ref("");
const newEntryInputEl = ref<HTMLInputElement | null>(null);

function startNewEntry(mode: "file" | "folder") {
  newEntryMode.value = mode;
  newEntryName.value = mode === "file" ? "New File.txt" : "New Folder";
  nextTick(() => {
    newEntryInputEl.value?.focus();
    if (mode === "file") {
      newEntryInputEl.value?.setSelectionRange(0, 8);
    } else {
      newEntryInputEl.value?.select();
    }
  });
}

async function commitNewEntry() {
  const mode = newEntryMode.value;
  const name = newEntryName.value.trim();
  newEntryMode.value = "";
  if (!name) return;
  if (mode === "file") {
    await ctrl.createFile(name);
  } else {
    await ctrl.createFolder(name);
  }
}

function cancelNewEntry() {
  newEntryMode.value = "";
}

function handleNewEntryKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    commitNewEntry();
  } else if (e.key === "Escape") {
    cancelNewEntry();
  }
}

// --- Keyboard shortcuts ---
const fileListEl = ref<HTMLElement | null>(null);

function isInputFocused(): boolean {
  const tag = document.activeElement?.tagName;
  return tag === "INPUT" || tag === "TEXTAREA";
}

function getSelectedEntries(): FileEntry[] {
  return sortedEntries.value.filter(e => selectedPaths.value.has(e.path));
}

function handleKeydown(e: KeyboardEvent) {
  if (isInputFocused()) return;

  const ctrl_ = e.ctrlKey || e.metaKey;

  if (ctrl_ && e.key === "x") {
    e.preventDefault();
    const paths = Array.from(selectedPaths.value);
    if (paths.length > 0) clipboardCut(paths);
  } else if (ctrl_ && e.key === "c") {
    e.preventDefault();
    const paths = Array.from(selectedPaths.value);
    if (paths.length > 0) clipboardCopy(paths);
  } else if (ctrl_ && e.key === "v") {
    e.preventDefault();
    clipboardPaste();
  } else if (ctrl_ && e.key === "a") {
    e.preventDefault();
    selectedPaths.value = new Set(sortedEntries.value.map(e => e.path));
  } else if (e.key === "Delete") {
    e.preventDefault();
    deleteSelected();
  } else if (e.key === "F2") {
    e.preventDefault();
    const sel = getSelectedEntries();
    if (sel.length === 1) startRename(sel[0]);
  } else if (e.key === "F5") {
    e.preventDefault();
    handleRefresh();
  } else if (e.key === "Enter" && !ctrl_) {
    const sel = getSelectedEntries();
    if (sel.length === 1) {
      e.preventDefault();
      handleEntryClick(sel[0]);
    }
  } else if (e.key === "Backspace") {
    e.preventDefault();
    handleGoUp();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});

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

// --- Group by ---
const groupBy = ref("");

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

type GroupedEntries = { label: string; entries: FileEntry[] }[];

const groupedEntries = computed<GroupedEntries>(() => {
  const entries = sortedEntries.value;
  if (!groupBy.value) return [{ label: "", entries }];

  const groups = new Map<string, FileEntry[]>();
  for (const entry of entries) {
    let key = "";
    if (groupBy.value === "type") {
      key = entry.is_dir ? "Folder" : "File";
    } else if (groupBy.value === "extension") {
      key = entry.is_dir ? "Folder" : (entry.extension.toUpperCase() || "No Extension");
    }
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(entry);
  }
  return Array.from(groups, ([label, entries]) => ({ label, entries }));
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
      <div
        class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm"
        @contextmenu.self="openBgMenu"
      >
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
          @contextmenu.prevent="openBgMenu"
        >
          <div class="text-center">
            <i class="pi pi-folder-open mb-2 text-3xl opacity-40" />
            <p v-if="ctrl.isSearchMode.value">No files found matching your search.</p>
            <p v-else>This folder is empty.</p>
          </div>
        </div>

        <!-- Entries -->
        <div v-else class="min-h-0 flex-1 overflow-y-auto" @contextmenu.self.prevent="openBgMenu">
          <!-- Group sections -->
          <template v-for="group in groupedEntries" :key="group.label">
            <div
              v-if="group.label"
              class="sticky top-0 z-10 border-b border-divider bg-canvas/95 px-4 py-1 text-xs font-bold text-muted backdrop-blur-sm"
            >
              {{ group.label }}
              <span class="ml-1 font-normal">({{ group.entries.length }})</span>
            </div>

            <div
              v-for="entry in group.entries"
              :key="entry.path"
              class="group grid grid-cols-[32px_minmax(0,1fr)_80px_100px_160px_80px] items-center gap-2 border-b border-divider px-4 py-1.5 text-sm transition-colors hover:bg-canvas"
              :class="[
                entry.is_dir ? 'cursor-pointer' : '',
                selectedPaths.has(entry.path) ? 'bg-brand/5' : '',
                clipboard?.cut && clipboard.paths.includes(entry.path) ? 'opacity-50' : '',
              ]"
              @dblclick="handleEntryClick(entry)"
              @contextmenu="openEntryMenu($event, entry)"
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
                  <!-- Rename mode -->
                  <input
                    v-if="renamingPath === entry.path"
                    ref="renameInputEl"
                    v-model="renameValue"
                    class="block w-full rounded border border-brand bg-canvas px-1.5 py-0.5 text-sm outline-none ring-1 ring-brand/30"
                    @keydown="handleRenameKeydown"
                    @blur="commitRename"
                    @click.stop
                  />
                  <!-- Normal display -->
                  <template v-else>
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
                  </template>
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
          </template>

          <!-- New entry inline row -->
          <div
            v-if="newEntryMode"
            class="grid grid-cols-[32px_minmax(0,1fr)_80px_100px_160px_80px] items-center gap-2 border-b border-divider bg-brand/5 px-4 py-1.5 text-sm"
          >
            <span />
            <div class="flex min-w-0 items-center gap-2">
              <i :class="newEntryMode === 'folder' ? 'pi pi-folder text-amber-500' : 'pi pi-file text-muted'" class="shrink-0 text-sm" />
              <input
                ref="newEntryInputEl"
                v-model="newEntryName"
                class="block w-full rounded border border-brand bg-canvas px-1.5 py-0.5 text-sm outline-none ring-1 ring-brand/30"
                @keydown="handleNewEntryKeydown"
                @blur="commitNewEntry"
                @click.stop
              />
            </div>
            <span />
            <span />
            <span />
            <span />
          </div>
        </div>

        <!-- Footer status -->
        <div class="flex items-center justify-between border-t border-divider px-4 py-1.5 text-xs text-muted">
          <span>
            {{ sortedEntries.length }} items
            <template v-if="selectedPaths.size > 0">
              <span class="ml-1 text-brand">({{ selectedPaths.size }} selected)</span>
            </template>
            <template v-if="clipboard">
              <span class="ml-2 text-amber-600">
                <i :class="clipboard.cut ? 'pi pi-scissors' : 'pi pi-copy'" class="text-[10px]" />
                {{ clipboard.paths.length }} in clipboard
              </span>
            </template>
          </span>
          <span v-if="!ctrl.isSearchMode.value">{{ ctrl.currentPath.value }}</span>
          <span v-if="ctrl.searchTruncated.value" class="text-amber-600">Results truncated to 500 items</span>
        </div>
      </div>
    </div>

    <!-- ===== Context Menu ===== -->
    <div
      v-if="ctxMenu"
      class="fixed z-50 min-w-52 overflow-visible rounded-lg border border-divider bg-panel py-1 text-sm shadow-xl"
      :style="{ left: `${ctxMenu.x}px`, top: `${ctxMenu.y}px` }"
      @click.stop
      @contextmenu.prevent.stop
    >
      <!-- === Entry context menu === -->
      <template v-if="ctxMenu.type === 'entry' && ctxMenu.entry">
        <button v-if="ctxMenu.entry.is_dir" class="ctx-item" @click="handleEntryClick(ctxMenu.entry!); closeCtxMenu()">
          <i class="pi pi-folder-open" /> Open
        </button>
        <button class="ctx-item" @click="ctrl.openInExplorer(ctxMenu.entry!.path); closeCtxMenu()">
          <i class="pi pi-external-link" /> Show in Explorer
        </button>
        <div class="ctx-sep" />
        <button class="ctx-item" @click="ctxCut">
          <i class="pi pi-scissors" /> Cut
          <span class="ctx-shortcut">Ctrl+X</span>
        </button>
        <button class="ctx-item" @click="ctxCopy">
          <i class="pi pi-copy" /> Copy
          <span class="ctx-shortcut">Ctrl+C</span>
        </button>
        <div class="ctx-sep" />
        <button class="ctx-item" @click="ctxRename">
          <i class="pi pi-pencil" /> Rename
          <span class="ctx-shortcut">F2</span>
        </button>
        <button class="ctx-item text-red-600 hover:!bg-red-50" @click="ctxDelete">
          <i class="pi pi-trash" /> Delete
          <span class="ctx-shortcut">Del</span>
        </button>
      </template>

      <!-- === Background context menu === -->
      <template v-else-if="ctxMenu.type === 'background'">
        <!-- New submenu -->
        <div class="relative">
          <button class="ctx-item" @click.stop="toggleSubmenu('new')">
            <i class="pi pi-plus" /> New
            <i class="pi pi-chevron-right ml-auto text-[10px]" />
          </button>
          <div
            v-if="ctxSubmenu === 'new'"
            class="absolute left-full top-0 z-10 min-w-40 rounded-lg border border-divider bg-panel py-1 shadow-xl"
          >
            <button class="ctx-item" @click="ctxNewFolder">
              <i class="pi pi-folder" /> Folder
            </button>
            <button class="ctx-item" @click="ctxNewFile">
              <i class="pi pi-file" /> File
            </button>
          </div>
        </div>
        <div class="ctx-sep" />

        <!-- Paste -->
        <button class="ctx-item" :class="!clipboard ? 'opacity-40 pointer-events-none' : ''" @click="ctxPaste">
          <i class="pi pi-clipboard" /> Paste
          <span class="ctx-shortcut">Ctrl+V</span>
        </button>
        <div class="ctx-sep" />

        <!-- Sort submenu -->
        <div class="relative">
          <button class="ctx-item" @click.stop="toggleSubmenu('sort')">
            <i class="pi pi-sort-alt" /> Sort by
            <i class="pi pi-chevron-right ml-auto text-[10px]" />
          </button>
          <div
            v-if="ctxSubmenu === 'sort'"
            class="absolute left-full top-0 z-10 min-w-40 rounded-lg border border-divider bg-panel py-1 shadow-xl"
          >
            <button class="ctx-item" :class="sortKey === 'name' ? 'font-bold text-brand' : ''" @click="ctxSortBy('name')">
              <i class="pi pi-check text-[10px]" :class="sortKey === 'name' ? '' : 'invisible'" /> Name
            </button>
            <button class="ctx-item" :class="sortKey === 'extension' ? 'font-bold text-brand' : ''" @click="ctxSortBy('extension')">
              <i class="pi pi-check text-[10px]" :class="sortKey === 'extension' ? '' : 'invisible'" /> Extension
            </button>
            <button class="ctx-item" :class="sortKey === 'size' ? 'font-bold text-brand' : ''" @click="ctxSortBy('size')">
              <i class="pi pi-check text-[10px]" :class="sortKey === 'size' ? '' : 'invisible'" /> Size
            </button>
            <button class="ctx-item" :class="sortKey === 'modified' ? 'font-bold text-brand' : ''" @click="ctxSortBy('modified')">
              <i class="pi pi-check text-[10px]" :class="sortKey === 'modified' ? '' : 'invisible'" /> Date modified
            </button>
          </div>
        </div>

        <!-- Group by submenu -->
        <div class="relative">
          <button class="ctx-item" @click.stop="toggleSubmenu('group')">
            <i class="pi pi-objects-column" /> Group by
            <i class="pi pi-chevron-right ml-auto text-[10px]" />
          </button>
          <div
            v-if="ctxSubmenu === 'group'"
            class="absolute left-full top-0 z-10 min-w-40 rounded-lg border border-divider bg-panel py-1 shadow-xl"
          >
            <button class="ctx-item" :class="!groupBy ? 'font-bold text-brand' : ''" @click="ctxGroupBy('')">
              <i class="pi pi-check text-[10px]" :class="!groupBy ? '' : 'invisible'" /> None
            </button>
            <button class="ctx-item" :class="groupBy === 'type' ? 'font-bold text-brand' : ''" @click="ctxGroupBy('type')">
              <i class="pi pi-check text-[10px]" :class="groupBy === 'type' ? '' : 'invisible'" /> Type
            </button>
            <button class="ctx-item" :class="groupBy === 'extension' ? 'font-bold text-brand' : ''" @click="ctxGroupBy('extension')">
              <i class="pi pi-check text-[10px]" :class="groupBy === 'extension' ? '' : 'invisible'" /> Extension
            </button>
          </div>
        </div>

        <div class="ctx-sep" />
        <button class="ctx-item" @click="ctxRefresh">
          <i class="pi pi-refresh" /> Refresh
        </button>
      </template>
    </div>

    <!-- ===== Delete Confirmation ===== -->
    <div
      v-if="confirmDeleteTarget"
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/40"
      @click.self="confirmDeleteTarget = null"
    >
      <div class="w-full max-w-sm rounded-lg border border-divider bg-panel p-6 shadow-2xl">
        <h3 class="mb-3 text-base font-bold text-ink">Confirm Delete</h3>
        <p class="mb-5 text-sm text-secondary">
          Are you sure you want to delete <strong>{{ confirmDeleteLabel }}</strong>? This action cannot be undone.
        </p>
        <div class="flex items-center justify-end gap-2">
          <button
            class="h-9 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            @click="confirmDeleteTarget = null"
          >
            Cancel
          </button>
          <button
            class="h-9 rounded-md bg-red-600 px-4 text-sm font-bold text-white hover:opacity-90"
            @click="executeDelete"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.ctx-item {
  @apply flex h-8 w-full items-center gap-2.5 px-3 text-left text-sm text-secondary hover:bg-canvas;
}
.ctx-sep {
  @apply my-1 border-t border-divider;
}
.ctx-shortcut {
  @apply ml-auto text-[11px] text-muted;
}
</style>
