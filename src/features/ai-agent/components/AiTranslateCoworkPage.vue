<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import DOMPurify from "dompurify";
import { marked } from "marked";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import Fieldset from "primevue/fieldset";
import InputText from "primevue/inputtext";
import RadioButton from "primevue/radiobutton";
import { useAiTranslateCowork } from "../composables/useAiTranslateCowork";
import type { AiAccount, AiAccountStatus, AiProvider } from "@/_/types/ai-usage";
import type { FileEntry } from "@/tauri/commands/explorer";

const ctrl = useAiTranslateCowork();

type PanelKey = "input" | "output";

function panelByKey(key: PanelKey) {
  return key === "input" ? ctrl.input : ctrl.output;
}

// Account đang active — dùng làm giá trị cho nhóm radio chọn account.
const activeAccountId = computed(() => ctrl.accounts.value.find((a) => a.is_active)?.id ?? null);

function selectAccount(id: number | null) {
  if (id == null || id === activeAccountId.value) return;
  void ctrl.setActiveAccount(id);
}

onMounted(() => {
  void ctrl.init();
});

// --- Markdown preview (dialog) ---
const mdViewMode = ref<"preview" | "raw">("preview");
const mdPreviewHtml = computed(() => {
  const raw = marked.parse(ctrl.mdPreviewContent.value, { async: false, breaks: true }) as string;
  return DOMPurify.sanitize(raw);
});

function openMarkdownPreview(entry: FileEntry) {
  mdViewMode.value = "preview";
  void ctrl.openMarkdownPreview(entry);
}

// --- Folder quick-view (Input panel, folders only) ---
const showFolderPeek = ref(false);
const folderPeekName = ref("");
const folderPeekFiles = ref<FileEntry[]>([]);
const isLoadingFolderPeek = ref(false);
const folderPeekSelected = ref<Set<string>>(new Set());

async function openFolderPeek(entry: FileEntry) {
  folderPeekName.value = entry.name;
  folderPeekFiles.value = [];
  folderPeekSelected.value = new Set();
  showFolderPeek.value = true;
  isLoadingFolderPeek.value = true;
  try {
    folderPeekFiles.value = await ctrl.readFolderEntries(entry.path);
  } finally {
    isLoadingFolderPeek.value = false;
  }
}

function openFolderPeekEntry(entry: FileEntry) {
  if (entry.is_dir) {
    void openFolderPeek(entry);
    return;
  }
  void ctrl.input.openEntry(entry);
}

function isFolderPeekSelected(entry: FileEntry): boolean {
  return folderPeekSelected.value.has(entry.path);
}

function toggleFolderPeekSelected(entry: FileEntry) {
  const next = new Set(folderPeekSelected.value);
  if (next.has(entry.path)) next.delete(entry.path);
  else next.add(entry.path);
  folderPeekSelected.value = next;
}

const showFolderPeekDeleteConfirm = ref(false);
const folderPeekDeleteCount = computed(() => folderPeekSelected.value.size);

function openFolderPeekDeleteConfirm() {
  if (!folderPeekSelected.value.size) return;
  showFolderPeekDeleteConfirm.value = true;
}

async function confirmFolderPeekDelete() {
  const paths = Array.from(folderPeekSelected.value);
  await ctrl.deletePaths(paths);
  folderPeekFiles.value = folderPeekFiles.value.filter((f) => !folderPeekSelected.value.has(f.path));
  folderPeekSelected.value = new Set();
  showFolderPeekDeleteConfirm.value = false;
}

// --- Import file/folder từ ngoài app vào Input (hỏi loại trước khi mở native picker) ---
const showImportTypeDialog = ref(false);

function openImportDialog() {
  showImportTypeDialog.value = true;
}

function chooseImportFiles() {
  showImportTypeDialog.value = false;
  void ctrl.pickAndImportToInput(false, focusedEntry.value?.path);
}

function chooseImportFolders() {
  showImportTypeDialog.value = false;
  void ctrl.pickAndImportToInput(true, focusedEntry.value?.path);
}

// --- Inline new folder ---
const inlineNewFolder = ref(false);
const inlineNewFolderName = ref("");
const inlineNewFolderInput = ref<HTMLInputElement | null>(null);

function openNewFolder(_key?: PanelKey) {
  inlineNewFolderName.value = "New Folder";
  inlineNewFolder.value = true;
  void nextTick(() => {
    const el = inlineNewFolderInput.value;
    if (el) { el.focus(); el.select(); }
  });
}

let newFolderPending = false;
async function confirmNewFolder() {
  if (newFolderPending) return;
  const name = inlineNewFolderName.value.trim();
  inlineNewFolder.value = false;
  if (!name) return;
  newFolderPending = true;
  try { await ctrl.input.createFolder(name); }
  finally { newFolderPending = false; }
}

function cancelNewFolder() {
  inlineNewFolder.value = false;
}

// --- Delete confirmation (per panel) ---
const showDeleteConfirm = ref(false);
const deleteTarget = ref<PanelKey>("input");

function openDeleteConfirm(key: PanelKey) {
  deleteTarget.value = key;
  showDeleteConfirm.value = true;
}

async function confirmDelete() {
  await panelByKey(deleteTarget.value).deleteSelected();
  showDeleteConfirm.value = false;
}

const deleteCount = computed(() => panelByKey(deleteTarget.value).selected.value.size);

// --- Skill terminal: require exactly one Input item selected ---
const showInputSelectionWarning = ref(false);
const inputSelectionWarningMessage = computed(() => {
  const count = ctrl.input.selected.value.size;
  return count === 0
    ? "Vui lòng chọn 1 mục trong Input trước khi mở terminal cho skill này."
    : "Vui lòng chỉ chọn 1 mục trong Input trước khi mở terminal cho skill này.";
});

// --- Skill terminal: choose "all" vs. specific files when the selected Input folder has multiple entries ---
const showSkillFileDialog = ref(false);
const skillFileDialogSkillName = ref("");
const skillFileDialogFolderName = ref("");
const skillFileDialogFiles = ref<FileEntry[]>([]);
const skillFileDialogSelected = ref<Set<string>>(new Set());
const checkingSkillName = ref<string | null>(null);

function isSkillFileSelected(entry: FileEntry): boolean {
  return skillFileDialogSelected.value.has(entry.path);
}

function toggleSkillFileSelected(entry: FileEntry) {
  const next = new Set(skillFileDialogSelected.value);
  if (next.has(entry.path)) next.delete(entry.path);
  else next.add(entry.path);
  skillFileDialogSelected.value = next;
}

async function openSkillTerminal(name: string) {
  if (ctrl.input.selected.value.size !== 1) {
    showInputSelectionWarning.value = true;
    return;
  }
  const selectedPath = Array.from(ctrl.input.selected.value)[0];
  const selectedEntry = ctrl.input.entries.value.find((e) => e.path === selectedPath);
  if (!selectedEntry) return;

  if (selectedEntry.is_dir) {
    checkingSkillName.value = name;
    const children = await ctrl.readFolderEntries(selectedEntry.path);
    checkingSkillName.value = null;
    if (children.length > 1) {
      skillFileDialogSkillName.value = name;
      skillFileDialogFolderName.value = selectedEntry.name;
      skillFileDialogFiles.value = children;
      skillFileDialogSelected.value = new Set();
      showSkillFileDialog.value = true;
      return;
    }
  }
  void ctrl.openSkillTerminal(name);
}

function confirmSkillFileDialogAll() {
  showSkillFileDialog.value = false;
  void ctrl.openSkillTerminal(skillFileDialogSkillName.value);
}

function confirmSkillFileDialogSelected() {
  const names = skillFileDialogFiles.value
    .filter((f) => skillFileDialogSelected.value.has(f.path))
    .map((f) => f.name);
  if (!names.length) return;
  showSkillFileDialog.value = false;
  void ctrl.openSkillTerminal(skillFileDialogSkillName.value, names);
}

// --- Focused folder (Input panel) ---
// Folder đang được focus (click chọn, hoặc right-click) — dùng làm đích cho Paste (context menu)
// và Import (toolbar), thay vì folder gốc/đang duyệt.
const focusedEntry = ref<FileEntry | null>(null);

function setFocusedEntry(entry: FileEntry) {
  focusedEntry.value = entry.is_dir ? entry : null;
}

function clearFocusedEntry() {
  focusedEntry.value = null;
}

// --- Context menu (Input panel) ---
const ctxMenu = ref(false);
const ctxX = ref(0);
const ctxY = ref(0);
const ctxMenuEl = ref<HTMLElement | null>(null);

function openCtxMenu(ev: MouseEvent, entry?: FileEntry) {
  ev.preventDefault();
  ctxX.value = ev.clientX;
  ctxY.value = ev.clientY;
  if (entry) setFocusedEntry(entry);
  ctxMenu.value = true;
  void nextTick(() => clampCtxMenu());
}

function clampCtxMenu() {
  const el = ctxMenuEl.value;
  if (!el) return;
  const r = el.getBoundingClientRect();
  if (r.right > window.innerWidth) ctxX.value -= r.right - window.innerWidth + 4;
  if (r.bottom > window.innerHeight) ctxY.value -= r.bottom - window.innerHeight + 4;
}

function closeCtxMenu() {
  ctxMenu.value = false;
}

function ctxCopy() {
  ctrl.input.copySelected();
  closeCtxMenu();
}

function ctxPaste() {
  if (focusedEntry.value) void ctrl.input.pasteInto(focusedEntry.value.path);
  else void ctrl.input.paste();
  closeCtxMenu();
}

function ctxDelete() {
  closeCtxMenu();
  openDeleteConfirm("input");
}

// --- Column resize (Input/Output ↔ Skills ↔ Project Directory) ---
const COL_MIN = 240;
const col1Width = ref(380);
const col2Width = ref(300);
let cleanupDrag: (() => void) | null = null;

function startColDrag(event: MouseEvent, col: "col1" | "col2") {
  event.preventDefault();
  const startX = event.clientX;
  const startWidth = col === "col1" ? col1Width.value : col2Width.value;
  function onMove(ev: MouseEvent) {
    const next = Math.max(COL_MIN, startWidth + (ev.clientX - startX));
    if (col === "col1") col1Width.value = next;
    else col2Width.value = next;
  }
  function onUp() {
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
    document.body.style.userSelect = "";
    cleanupDrag = null;
  }
  document.body.style.userSelect = "none";
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
  cleanupDrag = onUp;
}

// --- Row resize within Column 1 (Input ↔ Output) ---
const ROW_HANDLE_H = 10;
const ROW_MIN_H = 100;
const col1TopH = ref<number | null>(null);
let cleanupRowDrag: (() => void) | null = null;
const col1Container = ref<HTMLElement | null>(null);

function resolvedTopH(): number {
  const container = col1Container.value;
  if (!container) return 200;
  if (col1TopH.value != null) return col1TopH.value;
  return (container.getBoundingClientRect().height - ROW_HANDLE_H) / 2;
}

function startRowDrag(event: MouseEvent) {
  const container = col1Container.value;
  if (!container) return;
  const startY = event.clientY;
  const startH = resolvedTopH();
  const maxH = container.getBoundingClientRect().height - ROW_HANDLE_H - ROW_MIN_H;
  function onMove(ev: MouseEvent) {
    const next = startH + (ev.clientY - startY);
    col1TopH.value = Math.min(maxH, Math.max(ROW_MIN_H, next));
  }
  function onUp() {
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
    document.body.style.userSelect = "";
    document.body.style.cursor = "";
    cleanupRowDrag = null;
  }
  document.body.style.userSelect = "none";
  document.body.style.cursor = "row-resize";
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
  cleanupRowDrag = onUp;
}

onBeforeUnmount(() => { cleanupDrag?.(); cleanupRowDrag?.(); });

// --- Account helpers (copied from AI Cowork) ---
function providerLabel(p: AiProvider): string {
  return p === "codex" ? "Codex" : "Claude";
}

function statusLabel(status: AiAccountStatus): string {
  switch (status) {
    case "healthy":
      return "Healthy";
    case "low":
      return "Low";
    case "exhausted":
      return "Exhausted";
    case "error":
      return "Error";
    default:
      return "Unknown";
  }
}

function statusClass(status: AiAccountStatus): string {
  switch (status) {
    case "healthy":
      return "bg-emerald-100 text-emerald-700";
    case "low":
      return "bg-amber-100 text-amber-700";
    case "exhausted":
    case "error":
      return "bg-red-100 text-red-700";
    default:
      return "bg-canvas text-muted";
  }
}

function usagePercent(account: AiAccount): number {
  return account.account_type === "subscription" ? account.session_percent : account.usage_percent;
}

function hasUsage(account: AiAccount): boolean {
  return account.status !== "exhausted" && usagePercent(account) > 0;
}

function usageResetAt(account: AiAccount): string {
  return account.account_type === "subscription" ? account.session_reset_at : account.reset_at;
}

function resetHint(resetAt: string): string {
  const raw = resetAt?.trim();
  if (!raw) return "—";
  const target = new Date(raw.replace(" ", "T"));
  if (Number.isNaN(target.getTime())) return raw;
  const diffMs = target.getTime() - Date.now();
  const clock = raw.slice(11, 16) || "";
  if (diffMs <= 0) return `sắp reset · ${clock}`;
  const mins = Math.round(diffMs / 60000);
  const days = Math.floor(mins / 1440);
  const hours = Math.floor((mins % 1440) / 60);
  const rem = mins % 60;
  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (days === 0 && rem > 0) parts.push(`${rem}m`);
  const rel = parts.length ? parts.join(" ") : "<1m";
  return `còn ${rel} · ${clock}`;
}

// --- File-entry helpers ---
function formatSize(entry: FileEntry): string {
  if (entry.is_dir) return "—";
  const units = ["B", "KB", "MB", "GB"];
  let size = entry.size;
  let i = 0;
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024;
    i++;
  }
  return `${i === 0 ? size : size.toFixed(1)} ${units[i]}`;
}

function entryIcon(entry: FileEntry): string {
  return entry.is_dir ? "pi pi-folder text-amber-500" : "pi pi-file text-muted";
}

function isMarkdown(entry: FileEntry): boolean {
  return !entry.is_dir && entry.extension === "md";
}

function isTextResult(entry: FileEntry): boolean {
  return !entry.is_dir && ["md", "txt", "json", "csv", "log"].includes(entry.extension);
}
</script>

<template>
  <div class="flex flex-1 flex-col gap-4 overflow-hidden">
    <!-- Row 1: project directory -->
    <Fieldset class="shrink-0 rounded-lg border border-divider bg-panel p-5 shadow-sm fieldset-nested"
      legend="Project Directory"
      toggleable>
      <div class="flex flex-wrap items-center gap-3">
        <i class="pi pi-language text-2xl text-muted" />
        <div class="min-w-0">
          <p class="text-[12px] text-muted">Chọn thư mục project, quản lý account AI, chuẩn bị input và chạy skill dịch thuật.</p>
        </div>
      </div>

      <label class="block">
        <span class="text-xs font-bold text-muted">Project Directory</span>
        <div class="mt-1 flex gap-1.5">
          <input
            :value="ctrl.projectDir.value"
            type="text"
            readonly
            placeholder="Chọn thư mục project..."
            class="min-w-0 flex-1 rounded border border-divider bg-canvas px-3 py-2 font-mono text-sm text-ink"
          />
          <Button icon="pi pi-folder-open" label="Browse" severity="secondary" @click="ctrl.pickProjectDir" />
          <Button
            icon="pi pi-refresh"
            severity="secondary"
            outlined
            :disabled="!ctrl.projectDir.value"
            :loading="ctrl.isLoadingDir.value"
            title="Reload directory"
            @click="ctrl.loadDirectory"
          />
          <Button
            icon="pi pi-times"
            severity="secondary"
            outlined
            :disabled="!ctrl.projectDir.value"
            title="Clear"
            @click="ctrl.clearProjectDir"
          />
        </div>
      </label>
    </Fieldset>

    <!-- Row 2: Account AI -->
    <Fieldset
      class="shrink-0 rounded-lg border border-divider bg-panel p-4 shadow-sm fieldset-nested"
      legend="Account AI"
      toggleable
    >
      <div class="mt-4 max-h-56 overflow-auto">
        <p v-if="ctrl.isLoadingAccounts.value" class="p-4 text-center text-xs text-muted">Loading accounts...</p>
        <div v-else-if="ctrl.accounts.value.length" class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          <div
            v-for="account in ctrl.accounts.value"
            :key="account.id"
            class="rounded-lg border p-3 transition-colors"
            :class="[
              account.is_active ? 'border-brand ring-1 ring-brand/40' : 'border-divider',
              hasUsage(account) ? 'cursor-pointer hover:border-brand/60' : 'opacity-60',
            ]"
            :title="hasUsage(account) ? 'Chọn account này để sử dụng' : 'Account đã hết usage'"
            @click="hasUsage(account) && selectAccount(account.id)"
          >
            <div class="flex flex-wrap items-center gap-2">
              <RadioButton
                :model-value="activeAccountId"
                :value="account.id"
                :disabled="!hasUsage(account) || ctrl.settingActiveId.value !== null"
                :name="'ai-account'"
                class="shrink-0"
                @update:model-value="selectAccount"
                @click.stop
              />
              <i v-if="ctrl.settingActiveId.value === account.id" class="pi pi-spinner pi-spin shrink-0 text-xs text-brand" />
              <span class="truncate font-semibold text-ink" :title="account.name">{{ account.name }}</span>
              <span :class="['ml-auto shrink-0 rounded-full px-2 py-0.5 text-[11px] font-bold', statusClass(account.status)]">
                {{ statusLabel(account.status) }}
              </span>
            </div>
            <p class="mt-0.5 text-[11px] text-muted">
              {{ providerLabel(account.provider) }}<span v-if="account.subscription_type"> · {{ account.subscription_type }}</span>
            </p>
            <div class="mt-2 flex items-center justify-between text-[11px]">
              <span class="font-bold text-muted">
                {{ account.account_type === "subscription" ? "Current session" : "Usage remaining" }}
              </span>
              <span class="font-bold text-ink">{{ Math.round(usagePercent(account)) }}%</span>
            </div>
            <div class="mt-1 h-1.5 overflow-hidden rounded-full bg-canvas">
              <div
                class="h-full rounded-full bg-brand transition-all"
                :style="{ width: `${Math.min(100, Math.max(0, usagePercent(account)))}%` }"
              />
            </div>
            <p class="mt-1 flex items-center gap-1 text-[11px] text-muted">
              <i class="pi pi-clock" />reset {{ resetHint(usageResetAt(account)) }}
            </p>
          </div>
        </div>
        <p v-else class="p-4 text-center text-xs text-muted">Chưa có account AI nào. Thêm ở màn AI Usage.</p>
      </div>
    </Fieldset>

    <!-- Row 3: 3 columns (chỉ hiển thị khi đã chọn Project Directory) -->
    <div v-if="ctrl.projectDir.value" class="flex min-h-0 flex-1 gap-2 overflow-x-auto overflow-y-hidden">
      <!-- Column 1: input folder (top) + drag handle + output folder (bottom) -->
      <div ref="col1Container" class="flex min-h-0 shrink-0 flex-col" :style="{ width: col1Width + 'px' }">

        <!-- Input panel (top) -->
        <div
          class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel p-4 shadow-sm"
          :style="col1TopH != null ? { height: col1TopH + 'px', flexShrink: 0 } : { flex: '1 1 50%', minHeight: '100px' }"
        >
          <h3 class="mb-2 flex items-center gap-2 text-sm font-bold uppercase tracking-wide text-muted">
            <i class="pi pi-inbox" />Input
          </h3>
          <div class="mb-2 flex flex-wrap items-center gap-1.5">
            <Button icon="pi pi-folder-plus" severity="secondary" outlined size="small" title="New folder" @click="openNewFolder('input')" />
            <Button icon="pi pi-copy" severity="secondary" outlined size="small" :disabled="!ctrl.input.selected.value.size" title="Copy" @click="ctrl.input.copySelected" />
            <Button icon="pi pi-clipboard" severity="secondary" outlined size="small" title="Paste" @click="ctrl.input.paste" />
            <Button icon="pi pi-upload" severity="secondary" outlined size="small" title="Import file/folder" @click="openImportDialog" />
            <Button icon="pi pi-trash" severity="danger" outlined size="small" :disabled="!ctrl.input.selected.value.size" title="Delete" @click="openDeleteConfirm('input')" />
            <Button icon="pi pi-refresh" severity="secondary" text size="small" class="ml-auto" :loading="ctrl.input.isLoading.value" title="Reload" @click="ctrl.input.load" />
          </div>
          <!-- Input entries list -->
          <div class="min-h-0 flex-1 overflow-auto" @contextmenu="openCtxMenu" @click.self="clearFocusedEntry">
            <!-- Inline new folder row -->
            <div v-if="inlineNewFolder" class="flex items-center gap-2 rounded bg-brand/5 px-2 py-1.5 text-sm">
              <i class="pi pi-folder text-amber-500" />
              <input
                ref="inlineNewFolderInput"
                v-model="inlineNewFolderName"
                type="text"
                class="min-w-0 flex-1 rounded border border-brand bg-canvas px-2 py-0.5 text-sm text-ink outline-none focus:ring-1 focus:ring-brand"
                @keydown.enter="confirmNewFolder"
                @keydown.escape="cancelNewFolder"
                @blur="confirmNewFolder"
              />
            </div>
            <p v-if="ctrl.input.isLoading.value" class="p-6 text-center text-xs text-muted">Loading...</p>
            <template v-else-if="ctrl.input.entries.value.length">
              <div
                v-for="entry in ctrl.input.entries.value"
                :key="entry.path"
                class="flex items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-canvas/70"
                :class="[
                  ctrl.input.isSelected(entry) ? 'bg-brand/5' : '',
                  focusedEntry?.path === entry.path ? 'ring-1 ring-inset ring-brand' : '',
                  ctrl.clipboard.value?.cut && ctrl.clipboard.value.paths.includes(entry.path) ? 'opacity-50' : '',
                ]"
                :title="entry.path"
                @click="setFocusedEntry(entry)"
                @contextmenu.stop="openCtxMenu($event, entry)"
              >
                <Checkbox :model-value="ctrl.input.isSelected(entry)" binary @change="ctrl.input.toggleSelected(entry)" />
                <i :class="entryIcon(entry)" />
                <span class="min-w-0 flex-1 truncate" :class="entry.is_dir ? 'cursor-pointer font-semibold text-brand' : 'text-ink'" @dblclick="ctrl.input.openEntry(entry)">{{ entry.name }}</span>
                <span class="shrink-0 text-[11px] text-muted">{{ formatSize(entry) }}</span>
                <Button v-if="isTextResult(entry)" icon="pi pi-eye" text rounded size="small" title="Xem nội dung" @click="openMarkdownPreview(entry)" />
                <Button v-if="entry.is_dir" icon="pi pi-eye" text rounded size="small" title="View - xem nhanh" @click="openFolderPeek(entry)" />
                <Button v-if="entry.is_dir" icon="pi pi-external-link" text rounded size="small" title="Show in folder" @click="ctrl.showInFolder(entry.path)" />
              </div>
            </template>
            <p v-else class="flex h-full items-center justify-center rounded-lg border border-dashed border-divider p-6 text-center text-xs text-muted">Thư mục trống.</p>
          </div>
        </div>

        <!-- Drag handle: Input ↔ Output -->
        <div
          class="group flex h-2.5 shrink-0 cursor-row-resize items-center justify-center"
          title="Drag to resize"
          @mousedown.prevent="startRowDrag($event)"
        >
          <div class="h-1 w-12 rounded-full bg-divider transition-colors group-hover:bg-brand group-active:bg-brand" />
        </div>

        <!-- Output panel (bottom) -->
        <div
          class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel p-4 shadow-sm"
          style="min-height: 100px"
        >
          <h3 class="mb-2 flex items-center gap-2 text-sm font-bold uppercase tracking-wide text-muted">
            <i class="pi pi-sparkles" />Output (Skill Result)
          </h3>
          <div class="mb-2 flex flex-wrap items-center gap-1.5">
            <Button icon="pi pi-copy" severity="secondary" outlined size="small" :disabled="!ctrl.output.selected.value.size" title="Copy selected" @click="ctrl.output.copySelected" />
            <Button icon="pi pi-trash" severity="danger" outlined size="small" :disabled="!ctrl.output.selected.value.size" title="Delete selected" @click="openDeleteConfirm('output')" />
            <Button icon="pi pi-refresh" severity="secondary" text size="small" class="ml-auto" :loading="ctrl.output.isLoading.value" title="Reload" @click="ctrl.output.load" />
          </div>
          <div class="min-h-0 flex-1 overflow-auto">
            <p v-if="ctrl.output.isLoading.value" class="p-6 text-center text-xs text-muted">Loading...</p>
            <template v-else-if="ctrl.output.entries.value.length">
              <div v-for="entry in ctrl.output.entries.value" :key="entry.path" class="flex items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-canvas/70" :class="[ctrl.output.isSelected(entry) ? 'bg-brand/5' : '', ctrl.clipboard.value?.cut && ctrl.clipboard.value.paths.includes(entry.path) ? 'opacity-50' : '']" :title="entry.path">
                <Checkbox :model-value="ctrl.output.isSelected(entry)" binary @change="ctrl.output.toggleSelected(entry)" />
                <i :class="entryIcon(entry)" />
                <span class="min-w-0 flex-1 truncate" :class="entry.is_dir ? 'cursor-pointer font-semibold text-brand' : 'text-ink'" @dblclick="ctrl.output.openEntry(entry)">{{ entry.name }}</span>
                <span class="shrink-0 text-[11px] text-muted">{{ formatSize(entry) }}</span>
                <Button v-if="isTextResult(entry)" icon="pi pi-eye" text rounded size="small" title="Xem nội dung" @click="openMarkdownPreview(entry)" />
                <Button v-if="entry.is_dir" icon="pi pi-external-link" text rounded size="small" title="Show in folder" @click="ctrl.showInFolder(entry.path)" />
              </div>
            </template>
            <p v-else class="flex h-full items-center justify-center rounded-lg border border-dashed border-divider p-6 text-center text-xs text-muted">Thư mục trống.</p>
          </div>
        </div>
      </div>

      <!-- Drag handle: Column 1 ↔ Skills -->
      <div
        class="group flex w-1.5 shrink-0 cursor-col-resize items-center justify-center"
        title="Drag to resize"
        @mousedown="startColDrag($event, 'col1')"
      >
        <div class="h-10 w-1 rounded-full bg-divider transition-colors group-hover:bg-brand" />
      </div>

      <!-- Column 2: skills list -->
      <div
        class="flex min-h-0 shrink-0 flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm"
        :style="{ width: col2Width + 'px' }"
      >
        <h3 class="mb-3 flex items-center gap-2 text-sm font-bold uppercase tracking-wide text-muted">
          <i class="pi pi-book" />Skills
          <Button
            icon="pi pi-refresh"
            text
            rounded
            size="small"
            class="ml-auto"
            :loading="ctrl.isLoadingSkills.value"
            title="Reload skills"
            @click="ctrl.loadSkillFolders"
          />
        </h3>

        <div class="min-h-0 flex-1 space-y-2 overflow-auto">
          <template v-if="ctrl.skillFolders.value.length">
            <div
              v-for="name in ctrl.skillFolders.value"
              :key="name"
              class="flex items-center gap-2.5 rounded-lg border border-divider bg-canvas/50 p-3"
            >
              <i class="pi pi-book text-muted" />
              <span class="min-w-0 flex-1 truncate font-mono text-sm text-ink" :title="name">{{ name }}</span>
              <Button
                icon="pi pi-desktop"
                severity="secondary"
                outlined
                rounded
                size="small"
                :disabled="!ctrl.activeAccount.value"
                :loading="ctrl.openingSkillTerminal.value === name || checkingSkillName === name"
                :title="ctrl.activeAccount.value ? 'Mở terminal cho skill này' : 'Chưa có account AI active có CLAUDE_CONFIG_DIR'"
                @click="openSkillTerminal(name)"
              />
            </div>
          </template>
          <p
            v-else
            class="flex h-full items-center justify-center rounded-lg border border-dashed border-divider p-6 text-center text-xs text-muted"
          >
            Không tìm thấy skill nào trong <code class="mx-1 rounded bg-canvas px-1">.claude/skills</code> của project directory.
          </p>
        </div>
      </div>

      <!-- Drag handle: Skills ↔ Project Directory -->
      <div
        class="group flex w-1.5 shrink-0 cursor-col-resize items-center justify-center"
        title="Drag to resize"
        @mousedown="startColDrag($event, 'col2')"
      >
        <div class="h-10 w-1 rounded-full bg-divider transition-colors group-hover:bg-brand" />
      </div>

      <!-- Column 3: project directory listing -->
      <div class="flex min-h-0 flex-1 flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm" style="min-width: 260px">
        <h3 class="mb-3 flex items-center gap-2 text-sm font-bold uppercase tracking-wide text-muted">
          <i class="pi pi-folder-open" />Project Directory
          <Button
            icon="pi pi-refresh"
            text
            rounded
            size="small"
            class="ml-auto"
            :loading="ctrl.isLoadingDir.value"
            title="Reload"
            @click="ctrl.loadDirectory"
          />
        </h3>

        <div class="min-h-0 flex-1 overflow-auto">
          <p v-if="ctrl.isLoadingDir.value" class="p-6 text-center text-xs text-muted">Loading...</p>
          <template v-else-if="ctrl.dirEntries.value.length">
            <div
              v-for="entry in ctrl.dirEntries.value"
              :key="entry.path"
              class="flex items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-canvas/70"
              :title="entry.path"
            >
              <i :class="entryIcon(entry)" />
              <span class="min-w-0 flex-1 truncate text-ink">{{ entry.name }}</span>
              <span class="shrink-0 text-[11px] text-muted">{{ formatSize(entry) }}</span>
              <Button
                v-if="isMarkdown(entry)"
                icon="pi pi-eye"
                text
                rounded
                size="small"
                title="Xem nội dung markdown"
                @click="openMarkdownPreview(entry)"
              />
              <Button
                v-if="entry.is_dir"
                icon="pi pi-external-link"
                text
                rounded
                size="small"
                title="Show in folder"
                @click="ctrl.showInFolder(entry.path)"
              />
            </div>
          </template>
          <p v-else class="flex h-full items-center justify-center rounded-lg border border-dashed border-divider p-6 text-center text-xs text-muted">
            Thư mục trống.
          </p>
        </div>
      </div>
    </div>

    <!-- Row 3 placeholder: chưa chọn Project Directory -->
    <div
      v-else
      class="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-divider bg-panel/50 p-12 text-center"
    >
      <i class="pi pi-language text-3xl text-muted" />
      <h3 class="text-sm font-semibold text-ink">Chưa chọn Project Directory</h3>
      <p class="max-w-md text-sm text-muted">
        Bấm <span class="font-semibold text-ink">Browse</span> ở panel phía trên để chọn thư mục project, sau đó
        chuẩn bị input, chạy skill dịch, và xem nội dung thư mục ở đây.
      </p>
    </div>

    <!-- Markdown preview dialog -->
    <Dialog
      :visible="ctrl.mdPreviewOpen.value"
      class="w-full rounded-lg bg-panel shadow-xl"
      :style="{ width: '1100px' }"
      :content-style="{ display: 'flex', flexDirection: 'column' }"
      :closable="true"
      modal
      maximizable
      @update:visible="ctrl.mdPreviewOpen.value = $event"
    >
      <template #header>
        <div class="flex flex-1 items-center gap-3">
          <h3 class="flex min-w-0 items-center gap-2 font-bold text-ink">
            <i class="pi pi-file shrink-0" /><span class="truncate">{{ ctrl.mdPreviewName.value }}</span>
          </h3>
          <div class="ml-auto flex shrink-0 gap-1">
            <Button
              label="Preview"
              size="small"
              :severity="mdViewMode === 'preview' ? undefined : 'secondary'"
              :outlined="mdViewMode !== 'preview'"
              @click="mdViewMode = 'preview'"
            />
            <Button
              label="Raw"
              size="small"
              :severity="mdViewMode === 'raw' ? undefined : 'secondary'"
              :outlined="mdViewMode !== 'raw'"
              @click="mdViewMode = 'raw'"
            />
          </div>
        </div>
      </template>

      <p v-if="ctrl.isLoadingMdPreview.value" class="flex-1 p-6 text-center text-sm text-muted">Loading...</p>
      <div
        v-else-if="mdViewMode === 'preview'"
        class="md-preview min-h-[50vh] flex-1 overflow-auto rounded-lg border border-divider bg-canvas p-4 text-sm text-ink"
        v-html="mdPreviewHtml"
      />
      <pre
        v-else
        class="min-h-[50vh] flex-1 overflow-auto whitespace-pre-wrap break-words rounded-lg border border-divider bg-canvas p-4 text-sm text-ink"
      >{{ ctrl.mdPreviewContent.value }}</pre>

      <template #footer>
        <Button label="Đóng" severity="secondary" @click="ctrl.mdPreviewOpen.value = false" />
      </template>
    </Dialog>

    <!-- Folder quick-view (Input panel, folders only) -->
    <Dialog
      v-model:visible="showFolderPeek"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink"><i class="pi pi-folder text-amber-500" />{{ folderPeekName }}</h3>
      </template>
      <p v-if="isLoadingFolderPeek" class="p-6 text-center text-xs text-muted">Loading...</p>
      <div v-else-if="folderPeekFiles.length" class="max-h-[50vh] space-y-1 overflow-auto">
        <div
          v-for="entry in folderPeekFiles"
          :key="entry.path"
          class="flex items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-canvas/70"
          :title="entry.path"
        >
          <Checkbox :model-value="isFolderPeekSelected(entry)" binary @change="toggleFolderPeekSelected(entry)" />
          <i :class="entryIcon(entry)" />
          <span class="min-w-0 flex-1 truncate text-ink">{{ entry.name }}</span>
          <span class="shrink-0 text-[11px] text-muted">{{ formatSize(entry) }}</span>
          <Button
            icon="pi pi-eye"
            text
            rounded
            size="small"
            :title="entry.is_dir ? 'Xem nhanh folder này' : 'Mở file'"
            @click="openFolderPeekEntry(entry)"
          />
        </div>
      </div>
      <p v-else class="flex items-center justify-center rounded-lg border border-dashed border-divider p-6 text-center text-xs text-muted">
        Thư mục trống.
      </p>
      <template #footer>
        <Button
          icon="pi pi-trash"
          label="Xoá"
          severity="danger"
          outlined
          size="small"
          :disabled="!folderPeekSelected.size"
          @click="openFolderPeekDeleteConfirm"
        />
        <Button label="Đóng" severity="secondary" @click="showFolderPeek = false" />
      </template>
    </Dialog>

    <!-- Xoá file/folder đã chọn trong dialog xem nhanh (View) -->
    <Dialog
      v-model:visible="showFolderPeekDeleteConfirm"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink"><i class="pi pi-exclamation-triangle text-red-500" />Confirm Delete</h3>
      </template>
      <p class="text-sm text-muted">
        Xoá <strong class="text-ink">{{ folderPeekDeleteCount }}</strong> mục đã chọn trong "{{ folderPeekName }}"? Hành động này không thể hoàn tác.
      </p>
      <template #footer>
        <Button label="Cancel" severity="secondary" @click="showFolderPeekDeleteConfirm = false" />
        <Button label="Delete" severity="danger" @click="confirmFolderPeekDelete" />
      </template>
    </Dialog>

    <!-- Import file/folder từ ngoài app vào Input: hỏi loại trước khi mở native picker -->
    <Dialog
      v-model:visible="showImportTypeDialog"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink"><i class="pi pi-upload" />Import vào Input</h3>
      </template>
      <p class="text-sm text-muted">
        Bạn muốn chọn file hay folder? Nội dung sẽ được copy vào
        <strong v-if="focusedEntry" class="text-ink">folder "{{ focusedEntry.name }}"</strong>
        <span v-else>folder đang mở trong Input (folder gốc nếu chưa duyệt vào folder con nào)</span>.
      </p>
      <template #footer>
        <Button label="Cancel" severity="secondary" @click="showImportTypeDialog = false" />
        <Button label="Chọn folder" severity="secondary" outlined @click="chooseImportFolders" />
        <Button label="Chọn file" @click="chooseImportFiles" />
      </template>
    </Dialog>

    <!-- Delete confirmation -->
    <Dialog
      v-model:visible="showDeleteConfirm"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink"><i class="pi pi-exclamation-triangle text-red-500" />Confirm Delete</h3>
      </template>
      <p class="text-sm text-muted">
        Xoá <strong class="text-ink">{{ deleteCount }}</strong> mục đã chọn? Hành động này không thể hoàn tác.
      </p>
      <template #footer>
        <Button label="Cancel" severity="secondary" @click="showDeleteConfirm = false" />
        <Button label="Delete" severity="danger" @click="confirmDelete" />
      </template>
    </Dialog>

    <!-- Input selection warning (skill terminal) -->
    <Dialog
      v-model:visible="showInputSelectionWarning"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink"><i class="pi pi-exclamation-triangle text-yellow-500" />Chưa chọn Input</h3>
      </template>
      <p class="text-sm text-muted">{{ inputSelectionWarningMessage }}</p>
      <template #footer>
        <Button label="Đóng" severity="secondary" @click="showInputSelectionWarning = false" />
      </template>
    </Dialog>

    <!-- Skill terminal: choose all vs. specific files inside the selected Input folder -->
    <Dialog
      v-model:visible="showSkillFileDialog"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink"><i class="pi pi-folder-open" />Chọn file trong "{{ skillFileDialogFolderName }}"</h3>
      </template>
      <p class="mb-2 text-sm text-muted">Folder này có nhiều file. Chạy skill trên toàn bộ folder hay chỉ những file đã chọn?</p>
      <div class="max-h-[50vh] space-y-1 overflow-auto rounded-lg border border-divider p-2">
        <div
          v-for="entry in skillFileDialogFiles"
          :key="entry.path"
          class="flex items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-canvas/70"
        >
          <Checkbox :model-value="isSkillFileSelected(entry)" binary @change="toggleSkillFileSelected(entry)" />
          <i :class="entryIcon(entry)" />
          <span class="min-w-0 flex-1 truncate text-ink" :title="entry.name">{{ entry.name }}</span>
        </div>
      </div>
      <template #footer>
        <Button label="Cancel" severity="secondary" @click="showSkillFileDialog = false" />
        <Button label="Toàn bộ" severity="secondary" outlined @click="confirmSkillFileDialogAll" />
        <Button label="Chỉ file chọn" :disabled="!skillFileDialogSelected.size" @click="confirmSkillFileDialogSelected" />
      </template>
    </Dialog>

    <!-- Context menu (Input panel) -->
    <Teleport to="body">
      <div v-if="ctxMenu" class="fixed inset-0 z-[9998]" @click="closeCtxMenu" @contextmenu.prevent="closeCtxMenu" />
      <div
        v-if="ctxMenu"
        ref="ctxMenuEl"
        class="fixed z-[9999] min-w-[140px] rounded-lg border border-divider bg-panel py-1 shadow-xl"
        :style="{ left: ctxX + 'px', top: ctxY + 'px' }"
      >
        <button
          class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-sm text-ink hover:bg-canvas/80"
          @click="closeCtxMenu(); openNewFolder('input')"
        >
          <i class="pi pi-folder-plus text-xs text-muted" />New Folder
        </button>
        <div class="my-1 border-t border-divider" />
        <button
          class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-sm text-ink hover:bg-canvas/80 disabled:opacity-40 disabled:hover:bg-transparent"
          :disabled="!ctrl.input.selected.value.size"
          @click="ctxCopy"
        >
          <i class="pi pi-copy text-xs text-muted" />Copy
        </button>
        <button
          class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-sm text-ink hover:bg-canvas/80 disabled:opacity-40 disabled:hover:bg-transparent"
          @click="ctxPaste"
        >
          <i class="pi pi-clipboard text-xs text-muted" />Paste
        </button>
        <div class="my-1 border-t border-divider" />
        <button
          class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-sm text-red-600 hover:bg-canvas/80 disabled:opacity-40 disabled:hover:bg-transparent"
          :disabled="!ctrl.input.selected.value.size"
          @click="ctxDelete"
        >
          <i class="pi pi-trash text-xs" />Delete
        </button>
      </div>
    </Teleport>
  </div>
</template>

<!-- Không dùng `scoped` để class trong v-html (nội dung markdown render) được áp style. -->
<style>
.md-preview h1,
.md-preview h2,
.md-preview h3,
.md-preview h4 {
  color: rgb(var(--color-ink));
  font-weight: 700;
  margin-top: 1.25em;
  margin-bottom: 0.5em;
  line-height: 1.3;
}
.md-preview h1 { font-size: 1.5em; }
.md-preview h2 { font-size: 1.25em; }
.md-preview h3 { font-size: 1.1em; }
.md-preview p { margin: 0.6em 0; line-height: 1.6; }
.md-preview ul,
.md-preview ol {
  margin: 0.6em 0;
  padding-left: 1.5em;
}
.md-preview ul { list-style: disc; }
.md-preview ol { list-style: decimal; }
.md-preview li { margin: 0.25em 0; }
.md-preview a {
  color: rgb(var(--color-brand));
  text-decoration: underline;
}
.md-preview blockquote {
  margin: 0.75em 0;
  padding: 0.25em 1em;
  border-left: 3px solid rgb(var(--color-border));
  color: rgb(var(--color-text-muted));
}
.md-preview code {
  font-family: ui-monospace, monospace;
  font-size: 0.9em;
  background: rgb(var(--color-border) / 0.3);
  border-radius: 0.25em;
  padding: 0.1em 0.35em;
}
.md-preview pre {
  margin: 0.75em 0;
  padding: 0.75em 1em;
  border-radius: 0.5em;
  background: rgb(var(--color-border) / 0.2);
  overflow: auto;
}
.md-preview pre code {
  background: none;
  padding: 0;
}
.md-preview hr {
  margin: 1.25em 0;
  border: none;
  border-top: 1px solid rgb(var(--color-border));
}
.md-preview table {
  border-collapse: collapse;
  margin: 0.75em 0;
  width: 100%;
}
.md-preview th,
.md-preview td {
  border: 1px solid rgb(var(--color-border));
  padding: 0.4em 0.6em;
  text-align: left;
}
.md-preview th {
  background: rgb(var(--color-border) / 0.2);
  font-weight: 700;
}
.md-preview img {
  max-width: 100%;
}
</style>
