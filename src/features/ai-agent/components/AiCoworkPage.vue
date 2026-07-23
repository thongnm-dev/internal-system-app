<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import DOMPurify from "dompurify";
import { marked } from "marked";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import Fieldset from "primevue/fieldset";
import InputText from "primevue/inputtext";
import Select from "primevue/select";
import { useAiCowork } from "../composables/useAiCowork";
import { STEP_TYPE_META } from "@/_/types/ai-workflow";
import { TASK_CATEGORY_META, TASK_CATEGORY_OPTIONS } from "@/_/types/ai-task";
import type { AiTaskCategory } from "@/tauri/commands/ai-task";
import type { AiAccount, AiAccountStatus, AiProvider } from "@/_/types/ai-usage";
import type { FileEntry } from "@/tauri/commands/explorer";

function categoryLabel(category: string): string {
  return TASK_CATEGORY_META[category as AiTaskCategory]?.label ?? category;
}

function categoryBadgeClass(category: string): string {
  return TASK_CATEGORY_META[category as AiTaskCategory]?.badgeClass ?? "bg-canvas text-muted";
}

const ctrl = useAiCowork();

onMounted(() => {
  void ctrl.init();
});

// --- Markdown preview ---
const mdViewMode = ref<"preview" | "raw">("preview");
const mdPreviewHtml = computed(() => {
  const raw = marked.parse(ctrl.mdPreviewContent.value, { async: false, breaks: true }) as string;
  return DOMPurify.sanitize(raw);
});

function openMarkdownPreview(entry: FileEntry) {
  mdViewMode.value = "preview";
  void ctrl.openMarkdownPreview(entry);
}

const workflowOptions = computed(() => [
  { id: null, name: "-- Chưa chọn --" },
  ...ctrl.workflows.value,
]);

// --- Column resize (drag handles between Tasks / Workflow / Project Directory) ---
const COL_MIN = 220;
const col1Width = ref(360);
const col2Width = ref(320);
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

/** Còn usage để dùng — chỉ cho phép chọn active các account chưa cạn quota. */
function hasUsage(account: AiAccount): boolean {
  return account.status !== "exhausted" && usagePercent(account) > 0;
}

function usageResetAt(account: AiAccount): string {
  return account.account_type === "subscription" ? account.session_reset_at : account.reset_at;
}

/** Diễn giải reset_at (`YYYY-MM-DD HH:MM:SS`) thành chuỗi ngắn, vd "còn 2h 15m · 11:10". */
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
</script>

<template>
  <div class="flex flex-1 flex-col gap-4 overflow-hidden">
    <!-- Row 1: project directory -->
    <div class="shrink-0 rounded-lg border border-divider bg-panel p-5 shadow-sm">
      <div class="flex flex-wrap items-center gap-3">
        <i class="pi pi-objects-column text-2xl text-muted" />
        <div class="min-w-0">
          <h2 class="text-lg font-semibold text-ink">AI Cowork</h2>
          <p class="text-sm text-muted">Chọn thư mục project, quản lý account AI, áp dụng AI workflow, và theo dõi công việc cần thực hiện.</p>
        </div>
      </div>

      <label class="mt-4 block">
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
    </div>

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
            class="rounded-lg border p-3"
            :class="account.is_active ? 'border-brand ring-1 ring-brand/40' : 'border-divider'"
          >
            <div class="flex flex-wrap items-center gap-2">
              <span class="truncate font-semibold text-ink" :title="account.name">{{ account.name }}</span>
              <span v-if="account.is_active" class="shrink-0 rounded-full bg-brand px-2 py-0.5 text-[11px] font-bold text-white">
                ACTIVE
              </span>
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
            <div class="mt-3 flex flex-wrap items-center gap-2">
              <Button
                icon="pi pi-check-circle"
                label="Set active"
                size="small"
                :severity="account.is_active ? 'secondary' : undefined"
                :disabled="account.is_active || !hasUsage(account)"
                :loading="ctrl.settingActiveId.value === account.id"
                :title="hasUsage(account) ? 'Chọn làm account đang dùng' : 'Account đã hết usage'"
                @click="ctrl.setActiveAccount(account.id)"
              />
            </div>
          </div>
        </div>
        <p v-else class="p-4 text-center text-xs text-muted">Chưa có account AI nào. Thêm ở màn AI Usage.</p>
      </div>
    </Fieldset>

    <!-- Row 3: 3 columns (chỉ hiển thị khi đã chọn Project Directory) -->
    <div v-if="ctrl.projectDir.value" class="flex min-h-0 flex-1 gap-2 overflow-x-auto overflow-y-hidden">
      <!-- Column 1: tasks to perform -->
      <div
        class="flex min-h-0 shrink-0 flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm"
        :style="{ width: col1Width + 'px' }"
      >
        <h3 class="mb-3 flex items-center gap-2 text-sm font-bold uppercase tracking-wide text-muted">
          <i class="pi pi-list-check" />Tasks
          <Button
            icon="pi pi-plus"
            text
            rounded
            size="small"
            class="ml-auto"
            title="Add Task"
            @click="ctrl.openTaskPicker"
          />
        </h3>

        <div class="min-h-0 flex-1 space-y-2 overflow-auto">
          <template v-if="ctrl.selectedTasks.value.length">
            <div
              v-for="task in ctrl.selectedTasks.value"
              :key="task.id"
              class="flex items-start gap-2.5 rounded-lg border p-3"
              :class="ctrl.isTaskConfirmed(task.id) ? 'border-brand ring-1 ring-brand/40 bg-brand/5' : 'border-divider bg-canvas/50'"
            >
              <Checkbox
                :model-value="ctrl.isTaskConfirmed(task.id)"
                binary
                class="mt-1 shrink-0"
                title="Xác nhận lại task này"
                @change="ctrl.toggleTaskConfirmed(task.id)"
              />
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-2">
                  <span class="truncate font-semibold text-ink">{{ task.task_code }}</span>
                  <span :class="['shrink-0 rounded-full px-2 py-0.5 text-[11px] font-bold', categoryBadgeClass(task.category)]">
                    {{ categoryLabel(task.category) }}
                  </span>
                </div>
                <p v-if="task.description" class="mt-0.5 truncate text-xs text-muted" :title="task.description">
                  {{ task.description }}
                </p>
              </div>
              <Button
                icon="pi pi-times"
                text
                rounded
                size="small"
                title="Remove"
                @click="ctrl.removeSelectedTask(task.id)"
              />
            </div>
          </template>
          <p
            v-else
            class="flex h-full items-center justify-center rounded-lg border border-dashed border-divider p-6 text-center text-xs text-muted"
          >
            Chưa có task nào. Bấm nút "+" để tìm hoặc thêm task.
          </p>
        </div>
      </div>

      <!-- Drag handle: Tasks ↔ Workflow -->
      <div
        class="group flex w-1.5 shrink-0 cursor-col-resize items-center justify-center"
        title="Drag to resize"
        @mousedown="startColDrag($event, 'col1')"
      >
        <div class="h-10 w-1 rounded-full bg-divider transition-colors group-hover:bg-brand" />
      </div>

      <!-- Column 2: workflow picker + step list -->
      <div
        class="flex min-h-0 shrink-0 flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm"
        :style="{ width: col2Width + 'px' }"
      >
        <h3 class="mb-3 flex items-center gap-2 text-sm font-bold uppercase tracking-wide text-muted">
          <i class="pi pi-sitemap" />Workflow
        </h3>

        <div class="flex shrink-0 items-center gap-2">
          <Select
            v-model="ctrl.selectedWorkflowId.value"
            :options="workflowOptions"
            option-label="name"
            option-value="id"
            placeholder="Chọn workflow"
            class="min-w-0 flex-1"
            :loading="ctrl.isLoadingWorkflows.value"
          />
          <Button
            label="Apply"
            icon="pi pi-check"
            :disabled="ctrl.selectedWorkflowId.value === null"
            :loading="ctrl.isApplying.value"
            @click="ctrl.applyWorkflow"
          />
        </div>

        <p v-if="ctrl.selectedWorkflow.value?.description" class="mt-2 shrink-0 text-xs text-muted">
          {{ ctrl.selectedWorkflow.value.description }}
        </p>

        <div class="mt-3 min-h-0 flex-1 space-y-2 overflow-auto">
          <template v-if="ctrl.appliedWorkflowId.value !== null && ctrl.steps.value.length">
            <div
              v-for="(step, index) in ctrl.steps.value"
              :key="step.id"
              class="flex items-start gap-2.5 rounded-lg border border-divider bg-canvas/50 p-3"
            >
              <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-brand/10 text-xs font-bold text-brand">
                {{ index + 1 }}
              </span>
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-2">
                  <i :class="[STEP_TYPE_META[step.type].icon, 'text-muted']" />
                  <span class="truncate font-semibold text-ink">{{ step.name }}</span>
                  <span :class="['shrink-0 rounded-full px-2 py-0.5 text-[11px] font-bold', STEP_TYPE_META[step.type].badgeClass]">
                    {{ STEP_TYPE_META[step.type].label }}
                  </span>
                </div>
                <p v-if="step.description" class="mt-0.5 truncate text-xs text-muted" :title="step.description">
                  {{ step.description }}
                </p>
              </div>
              <Button
                v-if="step.type === 'skill'"
                icon="pi pi-desktop"
                severity="secondary"
                outlined
                rounded
                size="small"
                :disabled="!ctrl.hasMatchingSkill(step)"
                :loading="ctrl.openingTerminalStepId.value === step.id"
                :title="ctrl.hasMatchingSkill(step) ? 'Mở terminal cho skill này' : 'Không tìm thấy skill khớp trong .claude/skills'"
                @click="ctrl.openStepTerminal(step)"
              />
            </div>
          </template>
          <p
            v-else
            class="flex h-full items-center justify-center rounded-lg border border-dashed border-divider p-6 text-center text-xs text-muted"
          >
            {{ ctrl.workflows.value.length ? "Chọn workflow rồi bấm Apply để nạp danh sách step." : "Chưa có workflow nào." }}
          </p>
        </div>
      </div>

      <!-- Drag handle: Workflow ↔ Project Directory -->
      <div
        class="group flex w-1.5 shrink-0 cursor-col-resize items-center justify-center"
        title="Drag to resize"
        @mousedown="startColDrag($event, 'col2')"
      >
        <div class="h-10 w-1 rounded-full bg-divider transition-colors group-hover:bg-brand" />
      </div>

      <!-- Column 3: project directory listing -->
      <div
        class="flex min-h-0 flex-1 flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm"
        style="min-width: 260px"
      >
        <h3 class="mb-3 flex items-center gap-2 text-sm font-bold uppercase tracking-wide text-muted">
          <i class="pi pi-folder-open" />Project Directory
          <Button
            icon="pi pi-book"
            text
            rounded
            size="small"
            class="ml-auto"
            :disabled="!ctrl.projectDir.value"
            title="Xem danh sách skill available"
            @click="ctrl.openSkillListDialog"
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
            {{ ctrl.projectDir.value ? "Thư mục trống." : "Chọn project directory ở trên để nạp danh sách." }}
          </p>
        </div>
      </div>
    </div>

    <!-- Row 3 placeholder: chưa chọn Project Directory -->
    <div
      v-else
      class="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-divider bg-panel/50 p-12 text-center"
    >
      <i class="pi pi-folder-open text-3xl text-muted" />
      <h3 class="text-sm font-semibold text-ink">Chưa chọn Project Directory</h3>
      <p class="max-w-md text-sm text-muted">
        Bấm <span class="font-semibold text-ink">Browse</span> ở panel phía trên để chọn thư mục project, sau đó
        áp dụng workflow và xem nội dung thư mục ở đây.
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

    <!-- Cảnh báo: không có step skill nào khớp folder .claude/skills -->
    <Dialog
      :visible="ctrl.showSkillWarning.value"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="ctrl.showSkillWarning.value = $event"
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink">
          <i class="pi pi-exclamation-triangle text-amber-500" />Không tìm thấy skill khớp
        </h3>
      </template>

      <p class="text-sm text-muted">
        Workflow này có step loại <span class="font-semibold text-ink">Skill</span> nhưng không có step nào khớp với
        skill trong <code class="rounded bg-canvas px-1">.claude/skills</code> của project directory hiện tại.
        Nút "Open Terminal" sẽ bị disable cho các step này.
      </p>

      <template #footer>
        <Button label="Đã hiểu" @click="ctrl.showSkillWarning.value = false" />
      </template>
    </Dialog>

    <!-- Danh sách skill available trong .claude/skills của project directory -->
    <Dialog
      :visible="ctrl.showSkillListDialog.value"
      class="w-full rounded-lg bg-panel shadow-xl"
      :style="{ width: '640px' }"
      :closable="true"
      modal
      @update:visible="ctrl.showSkillListDialog.value = $event"
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink">
          <i class="pi pi-book" />Available Skills
        </h3>
      </template>

      <p v-if="ctrl.isLoadingSkillList.value" class="p-6 text-center text-sm text-muted">Loading...</p>
      <div v-else-if="ctrl.skillFolders.value.length" class="grid max-h-[40vh] grid-cols-2 gap-2 overflow-auto">
        <div
          v-for="name in ctrl.skillFolders.value"
          :key="name"
          class="flex items-center gap-2 rounded-lg border border-divider bg-canvas/50 px-3 py-2 text-sm"
        >
          <i class="pi pi-book text-muted" />
          <span class="min-w-0 flex-1 truncate font-mono text-ink">{{ name }}</span>
        </div>
      </div>
      <p v-else class="rounded-lg border border-dashed border-divider p-6 text-center text-sm text-muted">
        Không tìm thấy skill nào trong <code class="rounded bg-canvas px-1">.claude/skills</code> của project directory này.
      </p>

      <template #footer>
        <Button label="Đóng" severity="secondary" @click="ctrl.showSkillListDialog.value = false" />
      </template>
    </Dialog>

    <!-- Task picker: search hoặc thêm task, cho phép chọn nhiều -->
    <Dialog
      :visible="ctrl.showTaskPicker.value"
      class="w-full rounded-lg bg-panel shadow-xl"
      :style="{ width: '640px' }"
      :content-style="{ display: 'flex', flexDirection: 'column' }"
      :closable="true"
      modal
      @update:visible="ctrl.showTaskPicker.value = $event"
    >
      <template #header>
        <h3 class="flex items-center gap-2 font-bold text-ink">
          <i class="pi pi-list-check" />Select Tasks
        </h3>
      </template>

      <div class="space-y-4">
        <span class="flex items-center gap-2 rounded-md border border-divider bg-canvas px-2">
          <i class="pi pi-search text-xs text-muted" />
          <InputText
            v-model="ctrl.taskSearchQuery.value"
            class="embedded-input w-full border-0 !bg-transparent !py-1.5 !text-sm"
            placeholder="Search by task code, category, description..."
            @input="ctrl.triggerTaskSearch"
          />
        </span>

        <div class="rounded-lg border border-dashed border-divider p-3">
          <p class="mb-2 text-xs font-bold text-muted">Add new task</p>
          <div class="flex flex-wrap items-end gap-2">
            <label class="block min-w-0 flex-1">
              <span class="text-xs font-bold text-muted">Task Code</span>
              <InputText v-model="ctrl.newTaskCode.value" class="mt-1 w-full" placeholder="e.g. SCR-001" />
            </label>
            <label class="block w-36">
              <span class="text-xs font-bold text-muted">Category</span>
              <Select
                v-model="ctrl.newTaskCategory.value"
                :options="TASK_CATEGORY_OPTIONS"
                option-label="label"
                option-value="value"
                class="mt-1 w-full"
              />
            </label>
            <Button
              label="Create"
              icon="pi pi-plus"
              :disabled="!ctrl.newTaskCode.value.trim()"
              :loading="ctrl.isCreatingTask.value"
              @click="ctrl.createInlineTask"
            />
          </div>
          <label class="mt-2 block">
            <span class="text-xs font-bold text-muted">Description</span>
            <InputText v-model="ctrl.newTaskDescription.value" class="mt-1 w-full" placeholder="Optional" />
          </label>
        </div>

        <div class="max-h-[40vh] space-y-1.5 overflow-auto">
          <p v-if="ctrl.isSearchingTasks.value" class="p-4 text-center text-xs text-muted">Loading...</p>
          <template v-else-if="ctrl.taskSearchResults.value.length">
            <label
              v-for="task in ctrl.taskSearchResults.value"
              :key="task.id"
              class="flex cursor-pointer items-center gap-2.5 rounded-lg border p-2.5"
              :class="ctrl.isTaskPicked(task) ? 'border-brand ring-1 ring-brand/40' : 'border-divider'"
            >
              <Checkbox
                :model-value="ctrl.isTaskPicked(task)"
                binary
                class="shrink-0"
                @change="ctrl.toggleTaskPicked(task)"
              />
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-2">
                  <span class="truncate font-semibold text-ink">{{ task.task_code }}</span>
                  <span :class="['shrink-0 rounded-full px-2 py-0.5 text-[11px] font-bold', categoryBadgeClass(task.category)]">
                    {{ categoryLabel(task.category) }}
                  </span>
                </div>
                <p v-if="task.description" class="mt-0.5 truncate text-xs text-muted" :title="task.description">
                  {{ task.description }}
                </p>
              </div>
            </label>
          </template>
          <p v-else class="rounded-lg border border-dashed border-divider p-6 text-center text-xs text-muted">
            Không tìm thấy task nào.
          </p>
        </div>
      </div>

      <template #footer>
        <div class="flex w-full items-center justify-between gap-2">
          <span class="text-xs text-muted">{{ ctrl.pickerSelectedCount.value }} task đã chọn</span>
          <div class="flex items-center gap-2">
            <Button label="Cancel" severity="secondary" @click="ctrl.showTaskPicker.value = false" />
            <Button
              label="Add Selected"
              :disabled="ctrl.pickerSelectedCount.value === 0"
              @click="ctrl.confirmTaskPicker"
            />
          </div>
        </div>
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
