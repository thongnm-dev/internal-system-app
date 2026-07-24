<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
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

// The two symmetric folder panels of column 1 (both rooted in the project directory).
type PanelKey = "input" | "output";
const panels = [
  { key: "input" as PanelKey, title: "Input", icon: "pi pi-inbox", panel: ctrl.input },
  { key: "output" as PanelKey, title: "Output (Skill Result)", icon: "pi pi-sparkles", panel: ctrl.output },
];

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

// --- New folder dialog (per panel) ---
const showNewFolderDialog = ref(false);
const newFolderName = ref("");
const newFolderTarget = ref<PanelKey>("input");

function openNewFolder(key: PanelKey) {
  newFolderTarget.value = key;
  newFolderName.value = "New Folder";
  showNewFolderDialog.value = true;
}

async function confirmNewFolder() {
  await panelByKey(newFolderTarget.value).createFolder(newFolderName.value);
  showNewFolderDialog.value = false;
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

onBeforeUnmount(() => cleanupDrag?.());

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
          <h2 class="text-lg font-semibold text-ink">AI Translate Cowork</h2>
          <p class="text-sm text-muted">Chọn thư mục project, quản lý account AI, chuẩn bị input và chạy skill dịch thuật.</p>
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
      <!-- Column 1: input folder (top) + output / skill result folder (bottom) -->
      <div class="flex min-h-0 shrink-0 flex-col gap-2" :style="{ width: col1Width + 'px' }">
        <div
          v-for="p in panels"
          :key="p.key"
          class="flex min-h-0 flex-1 flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm"
        >
          <h3 class="mb-2 flex items-center gap-2 text-sm font-bold uppercase tracking-wide text-muted">
            <i :class="p.icon" />{{ p.title }}
          </h3>

          <!-- Toolbar -->
          <div class="mb-2 flex flex-wrap items-center gap-1.5">
            <Button
              icon="pi pi-arrow-up"
              severity="secondary"
              outlined
              size="small"
              :disabled="p.panel.isAtRoot.value"
              title="Up one folder"
              @click="p.panel.goUp"
            />
            <Button
              v-if="p.key === 'input'"
              icon="pi pi-folder-plus"
              label="New"
              severity="secondary"
              outlined
              size="small"
              title="New folder"
              @click="openNewFolder(p.key)"
            />
            <Button
              icon="pi pi-copy"
              label="Copy"
              severity="secondary"
              outlined
              size="small"
              :disabled="!p.panel.selected.value.size"
              title="Copy selected"
              @click="p.panel.copySelected"
            />
            <Button
              v-if="p.key === 'input'"
              icon="pi pi-clipboard"
              label="Paste"
              severity="secondary"
              outlined
              size="small"
              :disabled="!ctrl.clipboard.value"
              title="Paste"
              @click="p.panel.paste"
            />
            <Button
              icon="pi pi-trash"
              severity="danger"
              outlined
              size="small"
              :disabled="!p.panel.selected.value.size"
              title="Delete selected"
              @click="openDeleteConfirm(p.key)"
            />
            <Button
              icon="pi pi-refresh"
              severity="secondary"
              text
              size="small"
              class="ml-auto"
              :loading="p.panel.isLoading.value"
              title="Reload"
              @click="p.panel.load"
            />
          </div>

          <!-- File list -->
          <div class="min-h-0 flex-1 overflow-auto">
            <p v-if="p.panel.isLoading.value" class="p-6 text-center text-xs text-muted">Loading...</p>
            <template v-else-if="p.panel.entries.value.length">
              <div
                v-for="entry in p.panel.entries.value"
                :key="entry.path"
                class="flex items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-canvas/70"
                :class="[
                  p.panel.isSelected(entry) ? 'bg-brand/5' : '',
                  ctrl.clipboard.value?.cut && ctrl.clipboard.value.paths.includes(entry.path) ? 'opacity-50' : '',
                ]"
                :title="entry.path"
              >
                <Checkbox :model-value="p.panel.isSelected(entry)" binary @change="p.panel.toggleSelected(entry)" />
                <i :class="entryIcon(entry)" />
                <span
                  class="min-w-0 flex-1 truncate"
                  :class="entry.is_dir ? 'cursor-pointer font-semibold text-brand' : 'text-ink'"
                  @dblclick="p.panel.openEntry(entry)"
                >
                  {{ entry.name }}
                </span>
                <span class="shrink-0 text-[11px] text-muted">{{ formatSize(entry) }}</span>
                <Button
                  v-if="isTextResult(entry)"
                  icon="pi pi-eye"
                  text
                  rounded
                  size="small"
                  title="Xem nội dung"
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
                :loading="ctrl.openingSkillTerminal.value === name"
                :title="ctrl.activeAccount.value ? 'Mở terminal cho skill này' : 'Chưa có account AI active có CLAUDE_CONFIG_DIR'"
                @click="ctrl.openSkillTerminal(name)"
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

    <!-- New folder dialog -->
    <Dialog
      v-model:visible="showNewFolderDialog"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink"><i class="pi pi-folder-plus" />New Folder</h3>
      </template>
      <label class="block">
        <span class="text-xs font-bold text-muted">Folder name</span>
        <InputText v-model="newFolderName" class="mt-1 w-full" autofocus @keydown.enter="confirmNewFolder" />
      </label>
      <template #footer>
        <Button label="Cancel" severity="secondary" @click="showNewFolderDialog = false" />
        <Button label="Create" :disabled="!newFolderName.trim()" @click="confirmNewFolder" />
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
