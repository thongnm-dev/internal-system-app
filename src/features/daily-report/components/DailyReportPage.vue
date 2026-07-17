<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import InputNumber from "primevue/inputnumber";
import Checkbox from "primevue/checkbox";
import Calendar from "primevue/calendar";
import Select from "primevue/select";
import MessageBanner from "@/shared/components/MessageBanner.vue";
import { useToast } from "@/shared/composables/useToast";
import { useAuthStore } from "@/app/stores/auth";
import { syncDailyReport, type SyncResult } from "@/tauri/commands/sync";
import {
  dayTotal,
  emptyEntry,
  entryHour,
  entryKey,
  formatHoursDisplay,
  normalizeEntryForForm,
  projectDayTotal,
  projectTotal,
  useDailyReport,
  type DailyReportDay,
  type DailyReportEntries,
  type DailyReportEntry,
  type DailyReportProject,
  type DailyReportTaskRow,
  type NewTaskInput,
} from "../composables/useDailyReport";

const auth = useAuthStore();
const router = useRouter();
const toast = useToast();
const ctrl = useDailyReport(auth.user?.username);

// Estimate (giờ dự kiến hoàn thành task) — parse từ chuỗi, 0 nếu không hợp lệ.
function taskEstimate(task: DailyReportTaskRow): number {
  const n = Number(task.estimateHour);
  return Number.isFinite(n) && n > 0 ? n : 0;
}

// True khi tổng giờ tích luỹ đã đạt/ vượt estimate — để tô màu badge.
function taskReachedEstimate(task: DailyReportTaskRow): boolean {
  const est = taskEstimate(task);
  return est > 0 && ctrl.cumulativeHours(task.id) >= est;
}

// --- Editing cell dialog ---
type EditingCell = {
  day: DailyReportDay;
  entry: DailyReportEntry;
  task: DailyReportTaskRow;
};

const editingCell = ref<EditingCell | null>(null);
const editForm = ref<DailyReportEntry>(emptyEntry());

function isRowDisabled(task: DailyReportTaskRow): boolean {
  return !ctrl.isEditable.value || !!task.isCompleted;
}

function openEditDialog(task: DailyReportTaskRow, day: DailyReportDay) {
  if (isRowDisabled(task)) return;
  const entry = ctrl.entries.value[entryKey(task.rowId, day.day)];
  const normalized = normalizeEntryForForm(entry);
  editingCell.value = { day, entry: normalized, task };
  editForm.value = { ...normalized };
}

function saveEntry() {
  if (!editingCell.value) return;
  ctrl.updateEntry(editingCell.value.task.rowId, editingCell.value.day.day, editForm.value);
  editingCell.value = null;
}

function clearEntry() {
  if (!editingCell.value) return;
  ctrl.updateEntry(editingCell.value.task.rowId, editingCell.value.day.day, null);
  editingCell.value = null;
}

// --- Add project dialog ---
const isAddingProject = ref(false);
const projectKeyword = ref("");
const selectedProjectIds = ref<string[]>([]);

function openProjectPicker() {
  projectKeyword.value = "";
  selectedProjectIds.value = [];
  isAddingProject.value = true;
}

function toggleProjectSelection(projectId: string) {
  const index = selectedProjectIds.value.indexOf(projectId);
  if (index >= 0) {
    selectedProjectIds.value = selectedProjectIds.value.filter((id) => id !== projectId);
  } else {
    selectedProjectIds.value = [...selectedProjectIds.value, projectId];
  }
}

function confirmAddProjects() {
  selectedProjectIds.value.forEach((id) => ctrl.addProject(id));
  isAddingProject.value = false;
}

// --- Task dialog (add / edit) ---
const isTaskDialogOpen = ref(false);
const taskTargetProject = ref<DailyReportProject | null>(null);
const editingTaskId = ref<string | null>(null);
const taskForm = ref<NewTaskInput>(emptyTaskForm());
const savingTask = ref(false);
const taskError = ref("");

const isEditingTask = computed(() => editingTaskId.value !== null);

function emptyTaskForm(): NewTaskInput {
  return {
    shortName: "",
    description: "",
    category: "",
    assignee: auth.user?.username ?? "",
    estimateHour: "",
    dueDate: "",
    issueKey: "",
  };
}

function openTaskDialog() {
  if (!contextMenu.value) return;
  taskTargetProject.value = contextMenu.value.project;
  editingTaskId.value = null;
  taskForm.value = emptyTaskForm();
  taskError.value = "";
  contextMenu.value = null;
  isTaskDialogOpen.value = true;
}

function openEditTaskDialog(project: DailyReportProject, task: DailyReportTaskRow) {
  taskTargetProject.value = project;
  editingTaskId.value = task.id;
  taskForm.value = {
    shortName: task.name,
    description: task.description ?? "",
    category: task.category ?? "",
    assignee: task.assignee ?? auth.user?.username ?? "",
    estimateHour: task.estimateHour ?? "",
    dueDate: task.dueDate ?? "",
    issueKey: task.issueKey ?? "",
  };
  taskError.value = "";
  taskContextMenu.value = null;
  isTaskDialogOpen.value = true;
}

const canSaveTask = computed(() => taskForm.value.shortName.trim().length > 0);

async function saveTask() {
  if (!taskTargetProject.value || !canSaveTask.value || savingTask.value) return;
  savingTask.value = true;
  taskError.value = "";
  try {
    if (editingTaskId.value) {
      const task = await ctrl.updateTask(taskTargetProject.value.id, editingTaskId.value, taskForm.value);
      if (!task) {
        taskError.value = ctrl.error.value || "Cannot update task. Please try again.";
        return;
      }
      toast.success("Đã cập nhật task thành công.");
    } else {
      const task = await ctrl.addTask(taskTargetProject.value.id, taskForm.value);
      if (!task) {
        taskError.value = ctrl.error.value || "Cannot save task. Please try again.";
        return;
      }
    }
    isTaskDialogOpen.value = false;
    taskTargetProject.value = null;
    editingTaskId.value = null;
  } catch (e) {
    taskError.value = e instanceof Error ? e.message : String(e);
  } finally {
    savingTask.value = false;
  }
}

// --- Context menu ---
type ContextMenuState = {
  canDelete: boolean;
  project: DailyReportProject;
  x: number;
  y: number;
};

const contextMenu = ref<ContextMenuState | null>(null);

function openContextMenu(project: DailyReportProject, event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  contextMenu.value = {
    canDelete: Boolean(project.isUserAdded) && projectTotal(project, ctrl.entries.value) === 0,
    project,
    x: event.clientX,
    y: event.clientY,
  };
}

function closeContextMenu() {
  contextMenu.value = null;
}

function openBacklog() {
  if (!contextMenu.value) return;
  const project = contextMenu.value.project.name;
  contextMenu.value = null;
  router.push({ path: "/issue-backlog", query: { project } });
}

function deleteProject() {
  if (contextMenu.value) {
    ctrl.removeProject(contextMenu.value.project.id);
    contextMenu.value = null;
  }
}

watch(contextMenu, (val) => {
  if (val) {
    window.addEventListener("click", closeContextMenu);
    window.addEventListener("contextmenu", closeContextMenu);
    window.addEventListener("keydown", closeContextMenu);
  } else {
    window.removeEventListener("click", closeContextMenu);
    window.removeEventListener("contextmenu", closeContextMenu);
    window.removeEventListener("keydown", closeContextMenu);
  }
});

onUnmounted(() => {
  window.removeEventListener("click", closeContextMenu);
  window.removeEventListener("contextmenu", closeContextMenu);
  window.removeEventListener("keydown", closeContextMenu);
  window.removeEventListener("click", closeTaskContextMenu);
  window.removeEventListener("contextmenu", closeTaskContextMenu);
  window.removeEventListener("keydown", closeTaskContextMenu);
});

// --- Task context menu ---
type TaskContextMenuState = {
  project: DailyReportProject;
  task: DailyReportTaskRow;
  x: number;
  y: number;
};

const taskContextMenu = ref<TaskContextMenuState | null>(null);
const taskDeleteConfirm = ref<{ project: DailyReportProject; task: DailyReportTaskRow } | null>(null);
const deletingTask = ref(false);

function openTaskContextMenu(project: DailyReportProject, task: DailyReportTaskRow, event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  contextMenu.value = null;
  taskContextMenu.value = { project, task, x: event.clientX, y: event.clientY };
}

function closeTaskContextMenu() {
  taskContextMenu.value = null;
}

function openTaskBacklog(task: DailyReportTaskRow) {
  taskContextMenu.value = null;
  router.push({ path: "/issue-backlog", query: { task: task.id } });
}

function confirmDeleteTask() {
  if (!taskContextMenu.value) return;
  const { project, task } = taskContextMenu.value;
  taskContextMenu.value = null;
  taskDeleteConfirm.value = { project, task };
}

async function executeDeleteTask() {
  if (!taskDeleteConfirm.value || deletingTask.value) return;
  const { project, task } = taskDeleteConfirm.value;
  deletingTask.value = true;
  taskDeleteConfirm.value = null;
  try {
    await ctrl.removeTask(project.id, task.id);
    toast.success("Đã xóa task thành công.");
  } catch (e) {
    toast.error(e instanceof Error ? e.message : String(e));
  } finally {
    deletingTask.value = false;
  }
}

watch(taskContextMenu, (val) => {
  if (val) {
    window.addEventListener("click", closeTaskContextMenu);
    window.addEventListener("contextmenu", closeTaskContextMenu);
    window.addEventListener("keydown", closeTaskContextMenu);
  } else {
    window.removeEventListener("click", closeTaskContextMenu);
    window.removeEventListener("contextmenu", closeTaskContextMenu);
    window.removeEventListener("keydown", closeTaskContextMenu);
  }
});

// --- Scroll sync ---
const dayHeaderRef = ref<HTMLDivElement | null>(null);
const dayRowsRef = ref<HTMLDivElement | null>(null);
const tableScrollTop = ref(0);
const dayRowsScrollbarHeight = ref(0);

function syncFromDayRows(event: Event) {
  const el = event.currentTarget as HTMLDivElement;
  tableScrollTop.value = el.scrollTop;
  if (dayHeaderRef.value) {
    dayHeaderRef.value.scrollLeft = el.scrollLeft;
  }
}

function scrollFromProjectColumn(event: WheelEvent) {
  if (!dayRowsRef.value) return;
  event.preventDefault();
  dayRowsRef.value.scrollTop += event.deltaY;
  dayRowsRef.value.scrollLeft += event.deltaX;
}

let resizeObserver: ResizeObserver | null = null;

function updateScrollbarHeight() {
  if (dayRowsRef.value) {
    dayRowsScrollbarHeight.value = dayRowsRef.value.offsetHeight - dayRowsRef.value.clientHeight;
  }
}

onMounted(() => {
  nextTick(() => {
    updateScrollbarHeight();
    if (dayRowsRef.value) {
      resizeObserver = new ResizeObserver(updateScrollbarHeight);
      resizeObserver.observe(dayRowsRef.value);
    }
  });
});

watch([() => ctrl.days.value.length, () => ctrl.projects.value.length], () => {
  nextTick(updateScrollbarHeight);
});

onUnmounted(() => {
  resizeObserver?.disconnect();
});

// --- Month navigation helpers ---
function parseMonth(value: string) {
  const [year, month] = value.split("-").map(Number);
  if (!year || !month) return "";
  return value;
}

// --- Filtered projects for picker ---
function filteredPickerProjects() {
  const kw = projectKeyword.value.trim().toLowerCase();
  const available = ctrl.availableProjects.value;
  if (!kw) return available;
  return available.filter((p) => `${p.code} ${p.name}`.toLowerCase().includes(kw));
}

// --- Sync dialog ---
type SyncRow = {
  projectCode: string;
  projectName: string;
  taskName: string;
  processCode: string;
  categoryLabel: string;
  hour: number;
  comment: string;
  regularOt: number;
  midnightOt: number;
};

const isSyncDialogOpen = ref(false);
const syncDate = ref<Date>(new Date());

function formatDateStr(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function openSyncDialog() {
  syncDate.value = new Date();
  syncResult.value = null;
  isSyncing.value = false;
  isSyncDialogOpen.value = true;
}

const syncRows = computed<SyncRow[]>(() => {
  const sd = syncDate.value;
  if (!sd) return [];
  const selectedYear = sd.getFullYear();
  const selectedMonthIdx = sd.getMonth();
  const selectedDay = sd.getDate();

  const currentYear = ctrl.days.value.length > 0 ? Number(ctrl.monthValue.value.split("-")[0]) : 0;
  const currentMonthIdx = ctrl.days.value.length > 0 ? Number(ctrl.monthValue.value.split("-")[1]) - 1 : -1;
  if (selectedYear !== currentYear || selectedMonthIdx !== currentMonthIdx) return [];

  const rows: SyncRow[] = [];
  for (const project of ctrl.projects.value) {
    for (const task of project.tasks) {
      const key = entryKey(task.rowId, selectedDay);
      const entry = ctrl.entries.value[key];
      const hour = entryHour(entry);
      if (hour <= 0) continue;
      const parsed = typeof entry === "string" ? null : entry;
      rows.push({
        projectCode: project.code,
        projectName: project.name,
        taskName: task.name,
        processCode: task.category || "",
        categoryLabel: task.categoryLabel || "",
        hour,
        comment: parsed?.comment ?? "",
        regularOt: Number(parsed?.regularOt) || 0,
        midnightOt: Number(parsed?.midnightOt) || 0,
      });
    }
  }
  return rows;
});

const syncTotalHours = computed(() => syncRows.value.reduce((sum, r) => sum + r.hour, 0));

const isSyncing = ref(false);
const syncResult = ref<SyncResult | null>(null);

async function executeSyncDailyReport() {
  if (isSyncing.value || syncRows.value.length === 0) return;
  isSyncing.value = true;
  syncResult.value = null;
  try {
    const result = await syncDailyReport({
      date: formatDateStr(syncDate.value),
      entries: syncRows.value.map((r) => ({
        project_code: r.projectCode,
        project_name: r.projectName,
        task_name: r.taskName,
        process_code: r.processCode,
        category_label: r.categoryLabel,
        hour: r.hour,
        comment: r.comment,
        regular_ot: r.regularOt,
        midnight_ot: r.midnightOt,
      })),
    });
    syncResult.value = result;
    if (result.success) {
      toast.success(result.message);
    } else {
      toast.error(result.message);
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    syncResult.value = { success: false, message: msg, synced_count: 0, total_count: syncRows.value.length };
    toast.error(msg);
  } finally {
    isSyncing.value = false;
  }
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
    <!-- Top bar: project list + month navigation -->
    <section class="grid h-[76px] shrink-0 grid-cols-[380px_minmax(0,1fr)] border-b border-divider bg-panel">
      <div class="flex min-w-0 items-center justify-between gap-3 border-r border-divider px-4">
        <div class="min-w-0">
          <h3 class="truncate font-bold">Assigned work</h3>
          <p class="mt-1 truncate text-xs text-muted">{{ ctrl.projects.value.length.toLocaleString("en-US") }} projects</p>
        </div>
        <Button icon="pi pi-plus" size="small" title="Add project" :disabled="ctrl.availableProjects.value.length === 0" @click="openProjectPicker" />
      </div>

      <div class="flex min-w-0 items-center justify-between gap-4 px-4">
        <div class="flex min-w-0 items-center gap-3">
          <span class="flex h-10 w-10 items-center justify-center rounded-md bg-emerald-50 text-brand">
            <i class="pi pi-calendar text-xl" />
          </span>
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <h3 class="truncate font-bold">{{ ctrl.monthLabel.value }}</h3>
              <span
                v-if="!ctrl.isEditable.value"
                class="shrink-0 rounded-md bg-amber-100 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-amber-700"
              >
                Read-only
              </span>
            </div>
            <p class="mt-1 truncate text-xs text-muted">
              {{ ctrl.isEditable.value ? "Daily work hour input" : "Locked — editing is disabled for this month" }}
            </p>
          </div>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <Button icon="pi pi-refresh" severity="secondary" outlined size="small" title="Làm mới" @click="ctrl.reload()" />
          <Button icon="pi pi-sync" label="Đồng bộ" severity="secondary" outlined size="small" title="Đồng bộ hệ thống nội bộ" @click="openSyncDialog" />
          <Button icon="pi pi-chevron-left" severity="secondary" outlined size="small" title="Previous month" @click="ctrl.previousMonth()" />
          <InputText
            class="h-9 w-32 rounded-md border border-divider bg-panel px-3 text-center text-sm font-bold text-secondary outline-none hover:border-brand focus:border-brand focus:ring-2 focus:ring-emerald-100"
            type="month"
            :max="ctrl.maxMonthValue.value"
            :model-value="parseMonth(ctrl.monthValue.value)"
            @change="ctrl.selectMonth(($event.target as HTMLInputElement).value)"
          />
          <Button icon="pi pi-chevron-right" severity="secondary" outlined size="small" title="Next month" :disabled="!ctrl.canGoNextMonth.value" @click="ctrl.nextMonth()" />
        </div>
      </div>
    </section>

    <!-- Table section -->
    <section class="flex min-h-0 flex-1 flex-col overflow-hidden bg-panel">
      <!-- Column headers -->
      <div class="grid h-[76px] shrink-0 grid-cols-[380px_minmax(0,1fr)] border-b border-divider">
        <div class="flex min-w-0 items-center justify-between gap-3 border-r border-divider px-4">
          <div class="min-w-0">
            <h3 class="truncate text-sm font-bold text-ink">Project / task</h3>
            <p class="mt-1 truncate text-xs text-muted">Grouped by assigned project</p>
          </div>
          <span class="rounded-md bg-canvas px-2 py-1 text-xs font-bold text-muted">Total</span>
        </div>

        <div ref="dayHeaderRef" class="min-w-0 overflow-hidden">
          <div class="flex h-[76px] min-w-max">
            <div
              v-for="day in ctrl.days.value"
              :key="day.day"
              :class="[
                'flex w-12 shrink-0 flex-col items-center justify-center border-r border-divider px-1 text-center',
                day.isToday ? 'bg-brand text-white' : day.isWeekend ? 'bg-canvas' : '',
              ]"
            >
              <span class="text-sm font-extrabold leading-none">{{ day.label }}</span>
              <span :class="['mt-1 text-[8px] font-semibold', day.isToday ? 'text-white/60' : 'text-muted']">{{ day.weekday }}</span>
              <span class="mt-1 text-[11px] font-extrabold leading-none text-red-600">{{ formatHoursDisplay(dayTotal(ctrl.projects.value, ctrl.entries.value, day.day)) }}h</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Data rows -->
      <div class="grid min-h-0 flex-1 grid-cols-[380px_minmax(0,1fr)]">
        <!-- Project/task column (synced scroll via transform) -->
        <div class="min-w-0 overflow-hidden border-r border-divider" @wheel.prevent="scrollFromProjectColumn">
          <div
            :style="{
              transform: `translateY(-${tableScrollTop}px)`,
              willChange: 'transform',
            }"
          >
            <template v-for="project in ctrl.projects.value" :key="project.id">
              <!-- Project row -->
              <div
                class="flex h-12 items-center border-b border-emerald-200 bg-[#4cbd9b] px-4"
                @contextmenu="openContextMenu(project, $event)"
              >
                <div class="min-w-0 flex-1">
                  <strong class="block truncate text-sm text-white">{{ project.code }} - {{ project.name }}</strong>
                </div>
                <span class="ml-3 shrink-0 rounded-md bg-panel/20 px-2 py-1 text-xs font-bold text-white">
                  {{ formatHoursDisplay(projectTotal(project, ctrl.entries.value)) }}h
                </span>
              </div>
              <!-- Task rows (expanded by category) -->
              <div
                v-for="(task, tIdx) in project.tasks"
                :key="task.rowId"
                :class="['flex h-14 items-center gap-2 border-b border-divider px-4', tIdx % 2 === 0 ? 'bg-panel' : 'bg-canvas']"
                @contextmenu="openTaskContextMenu(project, task, $event)"
              >
                <Button
                  icon="pi pi-check"
                  :class="[
                    'flex !h-6 !w-6 shrink-0 items-center justify-center !rounded-full border transition',
                    task.isCompleted
                      ? 'border-brand bg-brand text-white hover:opacity-80'
                      : 'border-divider bg-panel text-muted hover:border-brand hover:text-brand',
                  ]"
                  unstyled
                  :title="task.isCompleted ? 'Bỏ delivery' : 'Delivery'"
                  @click="ctrl.setTaskCompleted(task.id, !task.isCompleted)"
                />
                <div class="min-w-0 flex-1">
                  <strong
                    class="block truncate text-sm"
                    :class="task.isCompleted ? 'text-muted line-through' : 'text-ink'"
                  >{{ task.categoryLabel ? `【${task.categoryLabel}】` : '' }}{{ task.name }}</strong>
                  <div class="mt-1 flex items-center gap-2">
                    <span
                      v-if="task.isCompleted"
                      class="shrink-0 rounded bg-emerald-50 px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wide text-brand"
                    >Delivered</span>
                  </div>
                </div>
                <div class="ml-1 flex shrink-0 flex-col items-end gap-0.5">
                  <span class="rounded-md bg-canvas px-2 py-0.5 text-xs font-bold text-secondary">
                    {{ formatHoursDisplay(ctrl.totalHours(task.rowId)) }}h
                  </span>
                  <span
                    class="text-[10px] font-bold tabular-nums"
                    :class="taskReachedEstimate(task) ? 'text-brand' : 'text-muted'"
                    :title="taskEstimate(task) ? 'Tổng giờ tích luỹ mọi tháng / estimate' : 'Tổng giờ tích luỹ mọi tháng'"
                  >
                    Σ {{ formatHoursDisplay(ctrl.cumulativeHours(task.id)) }}{{ taskEstimate(task) ? ` / ${formatHoursDisplay(taskEstimate(task))}h` : 'h' }}
                  </span>
                </div>
              </div>
            </template>
            <div aria-hidden="true" :style="{ height: `${dayRowsScrollbarHeight}px` }" />
          </div>
        </div>

        <!-- Day cells (scrollable) -->
        <div ref="dayRowsRef" class="min-w-0 overflow-auto" @scroll="syncFromDayRows">
          <div class="min-w-max">
            <template v-for="project in ctrl.projects.value" :key="project.id">
              <!-- Project day totals -->
              <div class="flex h-12 border-b border-emerald-200 bg-[#4cbd9b]">
                <div
                  v-for="day in ctrl.days.value"
                  :key="`${project.id}-${day.day}`"
                  :class="[
                    'flex h-12 w-12 shrink-0 items-center justify-center border-r border-white/20 px-1 text-xs font-extrabold tabular-nums text-white',
                    day.isWeekend ? 'bg-panel/10' : '',
                  ]"
                  :title="`${project.code} total - ${day.label} ${day.weekday}`"
                >
                  {{ formatHoursDisplay(projectDayTotal(project, ctrl.entries.value, day.day)) }}
                </div>
              </div>
              <!-- Task day cells -->
              <div v-for="(task, tIdx) in project.tasks" :key="task.rowId" class="flex h-14 border-b border-divider">
                <div
                  v-for="day in ctrl.days.value"
                  :key="`${task.rowId}-${day.day}`"
                  :class="[
                    'flex h-14 w-12 shrink-0 items-center justify-center border-r border-divider px-1',
                    day.isWeekend ? 'bg-canvas' : (tIdx % 2 === 0 ? 'bg-panel' : 'bg-canvas'),
                  ]"
                >
                  <Button
                    :class="[
                      'flex h-9 w-10 items-center justify-center rounded-md border text-sm font-bold tabular-nums outline-none transition focus:ring-2 focus:ring-emerald-100',
                      entryHour(ctrl.entries.value[entryKey(task.rowId, day.day)]) > 0
                        ? 'border-brand bg-emerald-50 text-brand'
                        : 'border-divider bg-panel text-muted',
                      isRowDisabled(task)
                        ? 'cursor-not-allowed opacity-60'
                        : entryHour(ctrl.entries.value[entryKey(task.rowId, day.day)]) > 0
                          ? 'hover:bg-emerald-100'
                          : 'hover:border-brand hover:text-brand',
                    ]"
                    unstyled
                    :disabled="isRowDisabled(task)"
                    :title="isRowDisabled(task)
                      ? `${task.categoryLabel ? `【${task.categoryLabel}】` : ''}${task.name} - ${day.label} ${day.weekday} (${task.isCompleted ? 'completed' : 'read-only'})`
                      : `${task.categoryLabel ? `【${task.categoryLabel}】` : ''}${task.name} - ${day.label} ${day.weekday}`"
                    @click="openEditDialog(task, day)"
                  >
                    <template v-if="entryHour(ctrl.entries.value[entryKey(task.rowId, day.day)]) > 0">
                      {{ formatHoursDisplay(entryHour(ctrl.entries.value[entryKey(task.rowId, day.day)])) }}
                    </template>
                    <i v-else-if="!isRowDisabled(task)" class="pi pi-plus" />
                    <span v-else class="text-muted">-</span>
                  </Button>
                </div>
              </div>
            </template>
          </div>
        </div>
      </div>
    </section>

    <!-- Edit entry dialog -->
    <Dialog
      :visible="!!editingCell"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      modal
      @update:visible="editingCell = null"
    >
      <template #header>
        <div>
          <h3 class="font-bold text-ink">Daily report detail</h3>
          <p v-if="editingCell" class="mt-1 text-sm text-muted">
            {{ editingCell.task.categoryLabel ? `【${editingCell.task.categoryLabel}】` : '' }}{{ editingCell.task.name }}
            — {{ editingCell.day.label }} {{ editingCell.day.weekday }}
          </p>
        </div>
      </template>

      <div class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Hour</span>
          <InputNumber
            :model-value="Number(editForm.hour) || null"
            class="mt-1 w-full"
            :min="0"
            :max="24"
            :step="0.25"
            :minFractionDigits="0"
            :maxFractionDigits="2"
            :useGrouping="false"
            @update:model-value="editForm.hour = String($event ?? '')"
          />
          <span v-if="Number(editForm.hour) > 0" class="mt-1 block text-xs tabular-nums text-muted">
            = {{ Math.round(Number(editForm.hour) * 60) }} phút
          </span>
        </label>

        <label class="block">
          <span class="text-xs font-bold text-muted">Comment</span>
          <textarea
            v-model="editForm.comment"
            class="mt-1 min-h-24 w-full resize-none rounded-md border border-divider bg-panel px-3 py-2 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
          />
        </label>

        <label class="flex items-center gap-2 text-sm font-bold text-secondary">
          <Checkbox
            v-model="editForm.isOt"
            :binary="true"
            class="h-4 w-4 accent-brand"
          />
          OT
        </label>

        <div v-if="editForm.isOt" class="grid gap-2 rounded-md border border-divider bg-canvas p-3">
          <label class="block">
            <span class="text-xs font-bold text-muted">Regular OT</span>
            <InputNumber
              :model-value="Number(editForm.regularOt) || null"
              class="mt-1 w-full"
              :min="0"
              :max="24"
              :step="0.25"
              :minFractionDigits="0"
              :maxFractionDigits="2"
              :useGrouping="false"
              @update:model-value="editForm.regularOt = String($event ?? '')"
            />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Midnight OT</span>
            <InputNumber
              :model-value="Number(editForm.midnightOt) || null"
              class="mt-1 w-full"
              :min="0"
              :max="24"
              :step="0.25"
              :minFractionDigits="0"
              :maxFractionDigits="2"
              :useGrouping="false"
              @update:model-value="editForm.midnightOt = String($event ?? '')"
            />
          </label>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-between gap-3">
          <Button label="Clear" severity="danger" outlined @click="clearEntry" />
          <div class="flex items-center gap-2">
            <Button label="Cancel" severity="secondary" @click="editingCell = null" />
            <Button label="Save" @click="saveEntry" />
          </div>
        </div>
      </template>
    </Dialog>

    <!-- Add project dialog -->
    <Dialog
      :visible="isAddingProject"
      class="w-full max-w-3xl overflow-hidden rounded-lg bg-panel shadow-xl"
      :style="{ maxHeight: '86vh' }"
      modal
      @update:visible="isAddingProject = $event"
    >
      <template #header>
        <div class="min-w-0">
          <h3 class="truncate font-bold text-ink">Add project</h3>
          <p class="mt-1 truncate text-sm text-muted">Select a project to add to daily report.</p>
        </div>
      </template>

      <div class="border-b border-divider bg-canvas px-5 py-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Project code / name</span>
          <InputText
            v-model="projectKeyword"
            class="mt-1 w-full"
            placeholder="Search by code or name"
          />
        </label>
      </div>

      <div class="min-h-0 flex-1 overflow-auto p-3">
        <p v-if="ctrl.availableProjects.value.length === 0" class="px-2 py-6 text-center text-sm text-muted">No more projects.</p>
        <p v-else-if="filteredPickerProjects().length === 0" class="px-2 py-6 text-center text-sm text-muted">No projects found.</p>
        <div v-else class="grid gap-2">
          <label
            v-for="project in filteredPickerProjects()"
            :key="project.id"
            :class="[
              'flex cursor-pointer items-center gap-3 rounded-md border px-4 py-3 hover:border-brand hover:bg-brand/10',
              selectedProjectIds.includes(project.id) ? 'border-brand bg-brand/10' : 'border-divider bg-panel',
            ]"
          >
            <Checkbox
              :model-value="selectedProjectIds.includes(project.id)"
              :binary="true"
              class="h-4 w-4 shrink-0 accent-brand"
              @update:model-value="toggleProjectSelection(project.id)"
            />
            <span class="min-w-0">
              <strong class="block truncate text-sm text-ink">{{ project.code }} - {{ project.name }}</strong>
              <span class="mt-1 block truncate text-xs font-semibold text-muted">{{ project.client }}</span>
            </span>
          </label>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-between gap-3">
          <span class="text-sm font-semibold text-muted">{{ selectedProjectIds.length }} selected</span>
          <div class="flex items-center gap-2">
            <Button label="Cancel" severity="secondary" @click="isAddingProject = false" />
            <Button label="Add projects" :disabled="selectedProjectIds.length === 0" @click="confirmAddProjects" />
          </div>
        </div>
      </template>
    </Dialog>

    <!-- Task dialog (add / edit) -->
    <Dialog
      :visible="isTaskDialogOpen"
      class="w-full max-w-lg rounded-lg bg-panel shadow-xl"
      :style="{ maxHeight: '90vh' }"
      modal
      @update:visible="isTaskDialogOpen = $event"
    >
      <template #header>
        <div class="min-w-0">
          <h3 class="truncate font-bold text-ink">{{ isEditingTask ? 'Chỉnh sửa Task' : 'New Task' }}</h3>
          <p v-if="taskTargetProject" class="mt-1 truncate text-sm text-muted">
            {{ taskTargetProject.code }} - {{ taskTargetProject.name }}
          </p>
        </div>
      </template>

      <div class="space-y-4">
        <!-- Short name -->
        <label class="block">
          <span class="text-xs font-bold text-muted">Short name <span class="text-red-500">*</span></span>
          <InputText
            v-model="taskForm.shortName"
            class="mt-1 w-full"
            placeholder="Task short name"
            autofocus
          />
        </label>

        <!-- Description -->
        <label class="block">
          <span class="text-xs font-bold text-muted">Mô tả</span>
          <textarea
            v-model="taskForm.description"
            class="mt-1 min-h-24 w-full resize-none rounded-md border border-divider bg-panel px-3 py-2 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            placeholder="Task description"
          />
        </label>

        <!-- Task category -->
        <label class="block">
          <span class="text-xs font-bold text-muted">Phân loại task</span>
          <Select
            v-model="taskForm.category"
            :options="ctrl.phases.value"
            option-label="name"
            option-value="code"
            placeholder="Chọn phân loại"
            class="mt-1 w-full"
          />
        </label>

        <div class="grid grid-cols-2 gap-3">
          <!-- Estimate hour -->
          <label class="block">
            <span class="text-xs font-bold text-muted">Estimate Hour</span>
            <InputNumber
              :model-value="Number(taskForm.estimateHour) || null"
              class="mt-1 w-full"
              :min="0"
              :step="0.25"
              :minFractionDigits="0"
              :maxFractionDigits="2"
              :useGrouping="false"
              placeholder="0"
              @update:model-value="taskForm.estimateHour = String($event ?? '')"
            />
          </label>

          <!-- Due date -->
          <label class="block">
            <span class="text-xs font-bold text-muted">Due Date</span>
            <Calendar
              :model-value="taskForm.dueDate ? new Date(taskForm.dueDate + 'T00:00:00') : null"
              class="mt-1 w-full"
              date-format="yy/mm/dd"
              placeholder="Select date"
              show-icon
              show-button-bar
              @update:model-value="taskForm.dueDate = $event ? `${$event.getFullYear()}-${String($event.getMonth() + 1).padStart(2, '0')}-${String($event.getDate()).padStart(2, '0')}` : ''"
            />
          </label>

          <!-- Backlog issue key -->
          <label class="block">
            <span class="text-xs font-bold text-muted">Link Issue Backlog</span>
            <InputText
              v-model="taskForm.issueKey"
              class="mt-1 w-full"
              placeholder="Issue Key"
            />
          </label>
        </div>
      </div>

      <template #footer>
        <div class="flex flex-col gap-2">
          <p v-if="taskError" class="text-right text-sm font-semibold text-red-500">{{ taskError }}</p>
          <div class="flex items-center justify-end gap-2">
            <Button label="Cancel" severity="secondary" @click="isTaskDialogOpen = false" />
            <Button :label="savingTask ? 'Saving…' : isEditingTask ? 'Cập nhật' : 'Add task'" :disabled="!canSaveTask || savingTask" @click="saveTask" />
          </div>
        </div>
      </template>
    </Dialog>

    <!-- Context menu -->
    <div
      v-if="contextMenu"
      class="fixed z-[60] w-52 overflow-hidden rounded-md border border-divider bg-panel py-1 text-sm text-secondary shadow-xl"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      @click.stop
      @contextmenu.prevent
    >
      <Button icon="pi pi-plus" label="Thêm nhanh task" text size="small" class="w-full justify-start" @click="openTaskDialog" />
      <Button icon="pi pi-folder-open" label="Xem backlog" text size="small" class="w-full justify-start" @click="openBacklog" />
      <Button icon="pi pi-upload" label="Import task" text size="small" class="w-full justify-start" />
    </div>

    <!-- Task context menu -->
    <div
      v-if="taskContextMenu"
      class="fixed z-[60] w-52 overflow-hidden rounded-md border border-divider bg-panel py-1 text-sm text-secondary shadow-xl"
      :style="{ left: `${taskContextMenu.x}px`, top: `${taskContextMenu.y}px` }"
      @click.stop
      @contextmenu.prevent
    >
      <Button v-if="taskContextMenu.task.isUserAdded" icon="pi pi-pencil" label="Chỉnh sửa task" text size="small" class="w-full justify-start" @click="openEditTaskDialog(taskContextMenu.project, taskContextMenu.task)" />
      <Button icon="pi pi-folder-open" label="Xem backlog" text size="small" class="w-full justify-start" @click="openTaskBacklog(taskContextMenu.task)" />
      <Button v-if="taskContextMenu.task.isUserAdded && !taskContextMenu.task.isCompleted" icon="pi pi-trash" label="Xóa task" text severity="danger" size="small" class="w-full justify-start" @click="confirmDeleteTask" />
    </div>

    <!-- Delete task confirm dialog -->
    <Dialog
      :visible="!!taskDeleteConfirm"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      modal
      @update:visible="taskDeleteConfirm = null"
    >
      <template #header>
        <h3 class="font-bold text-ink">Xác nhận xóa task</h3>
      </template>
      <p class="text-sm text-secondary">
        Bạn có chắc muốn xóa task
        <strong class="text-ink">{{ taskDeleteConfirm?.task.name }}</strong>?
      </p>
      <template #footer>
        <div class="flex justify-end gap-2">
          <Button label="Hủy" severity="secondary" @click="taskDeleteConfirm = null" />
          <Button label="Xóa" severity="danger" :disabled="deletingTask" @click="executeDeleteTask" />
        </div>
      </template>
    </Dialog>

    <!-- Sync dialog -->
    <Dialog
      :visible="isSyncDialogOpen"
      class="w-full max-w-2xl rounded-lg bg-panel shadow-xl"
      :style="{ maxHeight: '86vh' }"
      modal
      @update:visible="isSyncDialogOpen = $event"
    >
      <template #header>
        <div class="min-w-0">
          <h3 class="truncate font-bold text-ink">Đồng bộ hệ thống nội bộ</h3>
          <p class="mt-1 truncate text-sm text-muted">Xác nhận dữ liệu trước khi đồng bộ lên hệ thống công ty.</p>
        </div>
      </template>

      <div class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Ngày đồng bộ</span>
          <Calendar
            v-model="syncDate"
            dateFormat="yy-mm-dd"
            showIcon
            class="mt-1 w-44"
            inputClass="h-10 text-sm font-bold"
          />
        </label>

        <div v-if="syncResult" :class="['rounded-md border px-4 py-3', syncResult.success ? 'border-brand/30 bg-brand/10' : 'border-red-500/30 bg-red-500/10']">
          <div class="flex items-center gap-2">
            <i :class="['pi text-sm', syncResult.success ? 'pi-check-circle text-brand' : 'pi-times-circle text-red-500']" />
            <span :class="['text-sm font-bold', syncResult.success ? 'text-brand' : 'text-red-500']">{{ syncResult.message }}</span>
          </div>
        </div>

        <div v-if="syncRows.length === 0" class="rounded-md border border-divider bg-canvas px-4 py-8 text-center">
          <i class="pi pi-inbox mb-2 text-3xl text-muted" />
          <p class="text-sm text-muted">Không có dữ liệu cho ngày này.</p>
          <p class="mt-1 text-xs text-muted">Chọn ngày khác hoặc kiểm tra daily report đã nhập giờ chưa.</p>
        </div>

        <div v-else class="overflow-hidden rounded-md border border-divider">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-divider bg-canvas text-left">
                <th class="px-3 py-2 font-bold text-muted">Dự án</th>
                <th class="px-3 py-2 font-bold text-muted">Task</th>
                <th class="px-3 py-2 text-center font-bold text-muted">Giờ</th>
                <th class="px-3 py-2 font-bold text-muted">Ghi chú</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(row, idx) in syncRows"
                :key="idx"
                :class="idx % 2 === 0 ? 'bg-panel' : 'bg-canvas'"
                class="border-b border-divider last:border-b-0"
              >
                <td class="max-w-[140px] truncate px-3 py-2 font-semibold text-ink" :title="`${row.projectCode} - ${row.projectName}`">
                  {{ row.projectCode }}
                </td>
                <td class="max-w-[200px] truncate px-3 py-2 text-secondary">
                  <span v-if="row.categoryLabel" class="text-muted">【{{ row.categoryLabel }}】</span>{{ row.taskName }}
                </td>
                <td class="px-3 py-2 text-center font-bold tabular-nums text-brand">{{ formatHoursDisplay(row.hour) }}h</td>
                <td class="max-w-[160px] truncate px-3 py-2 text-muted" :title="row.comment">{{ row.comment || "—" }}</td>
              </tr>
            </tbody>
            <tfoot>
              <tr class="border-t-2 border-brand bg-brand/10">
                <td class="px-3 py-2 font-bold text-ink" colspan="2">Tổng cộng ({{ syncRows.length }} mục)</td>
                <td class="px-3 py-2 text-center font-extrabold tabular-nums text-brand">{{ formatHoursDisplay(syncTotalHours) }}h</td>
                <td class="px-3 py-2"></td>
              </tr>
            </tfoot>
          </table>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Đóng" severity="secondary" @click="isSyncDialogOpen = false" />
          <Button :icon="isSyncing ? 'pi pi-spinner pi-spin' : 'pi pi-sync'" :label="isSyncing ? 'Đang đồng bộ…' : 'Đồng bộ'" :disabled="syncRows.length === 0 || isSyncing" @click="executeSyncDailyReport" />
        </div>
      </template>
    </Dialog>
  </section>
</template>
