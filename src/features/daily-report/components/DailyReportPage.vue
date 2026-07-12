<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import Dialog from "primevue/dialog";
import Checkbox from "primevue/checkbox";
import { useAuthStore } from "@/app/stores/auth";
import {
  dayTotal,
  emptyEntry,
  entryHour,
  entryKey,
  formatHoursDisplay,
  normalizeEntryForForm,
  projectDayTotal,
  projectTotal,
  TASK_CATEGORIES,
  useDailyReport,
  type DailyReportDay,
  type DailyReportEntries,
  type DailyReportEntry,
  type DailyReportProject,
  type DailyReportTask,
  type NewTaskInput,
  type TaskCategory,
} from "../composables/useDailyReport";

const auth = useAuthStore();
const router = useRouter();
const ctrl = useDailyReport(auth.user?.username);

// --- Editing cell dialog ---
type EditingCell = {
  day: DailyReportDay;
  entry: DailyReportEntry;
  task: DailyReportTask;
};

const editingCell = ref<EditingCell | null>(null);
const editForm = ref<DailyReportEntry>(emptyEntry());

function openEditDialog(task: DailyReportTask, day: DailyReportDay) {
  if (!ctrl.isEditable.value) return;
  const entry = ctrl.entries.value[entryKey(task.id, day.day)];
  const normalized = normalizeEntryForForm(entry);
  editingCell.value = { day, entry: normalized, task };
  editForm.value = { ...normalized };
}

function saveEntry() {
  if (!editingCell.value) return;
  ctrl.updateEntry(editingCell.value.task.id, editingCell.value.day.day, editForm.value);
  editingCell.value = null;
}

function clearEntry() {
  if (!editingCell.value) return;
  ctrl.updateEntry(editingCell.value.task.id, editingCell.value.day.day, null);
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

// --- New task dialog ---
const isAddingTask = ref(false);
const taskTargetProject = ref<DailyReportProject | null>(null);
const taskForm = ref<NewTaskInput>(emptyTaskForm());

function emptyTaskForm(): NewTaskInput {
  return {
    shortName: "",
    description: "",
    categories: [],
    assignee: auth.user?.username ?? "",
    estimateHour: "",
    dueDate: "",
    issueKey: "",
  };
}

function openTaskDialog() {
  if (!contextMenu.value) return;
  taskTargetProject.value = contextMenu.value.project;
  taskForm.value = emptyTaskForm();
  contextMenu.value = null;
  isAddingTask.value = true;
}

function toggleTaskCategory(category: TaskCategory) {
  const list = taskForm.value.categories;
  taskForm.value.categories = list.includes(category)
    ? list.filter((c) => c !== category)
    : [...list, category];
}

const canSaveTask = computed(() => taskForm.value.shortName.trim().length > 0);

function saveTask() {
  if (!taskTargetProject.value || !canSaveTask.value) return;
  ctrl.addTask(taskTargetProject.value.id, taskForm.value);
  isAddingTask.value = false;
  taskTargetProject.value = null;
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
        <button
          class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-brand text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
          type="button"
          title="Add project"
          :disabled="ctrl.availableProjects.value.length === 0"
          @click="openProjectPicker"
        >
          <i class="pi pi-plus" />
        </button>
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
          <button
            class="flex h-9 w-9 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
            type="button"
            title="Previous month"
            @click="ctrl.previousMonth()"
          >
            <i class="pi pi-chevron-left" />
          </button>
          <input
            class="h-9 w-32 rounded-md border border-divider bg-panel px-3 text-center text-sm font-bold text-secondary outline-none hover:border-brand focus:border-brand focus:ring-2 focus:ring-emerald-100"
            type="month"
            :max="ctrl.maxMonthValue.value"
            :value="parseMonth(ctrl.monthValue.value)"
            @change="ctrl.selectMonth(($event.target as HTMLInputElement).value)"
          />
          <button
            class="flex h-9 w-9 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            title="Next month"
            :disabled="!ctrl.canGoNextMonth.value"
            @click="ctrl.nextMonth()"
          >
            <i class="pi pi-chevron-right" />
          </button>
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
              <!-- Task rows -->
              <div
                v-for="task in project.tasks"
                :key="task.id"
                class="flex h-14 items-center border-b border-divider bg-panel px-4"
              >
                <div class="min-w-0 flex-1 pl-3">
                  <strong class="block truncate text-sm text-ink">{{ task.name }}</strong>
                  <span class="mt-1 block truncate text-xs font-semibold text-muted">{{ task.code }}</span>
                </div>
                <span class="ml-3 shrink-0 rounded-md bg-canvas px-2 py-1 text-xs font-bold text-secondary">
                  {{ formatHoursDisplay(ctrl.totalHours(task.id)) }}h
                </span>
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
              <div v-for="task in project.tasks" :key="task.id" class="flex h-14 border-b border-divider">
                <div
                  v-for="day in ctrl.days.value"
                  :key="`${task.id}-${day.day}`"
                  :class="[
                    'flex h-14 w-12 shrink-0 items-center justify-center border-r border-divider px-1',
                    day.isWeekend ? 'bg-canvas' : 'bg-panel',
                  ]"
                >
                  <button
                    :class="[
                      'flex h-9 w-10 items-center justify-center rounded-md border text-sm font-bold tabular-nums outline-none transition focus:ring-2 focus:ring-emerald-100',
                      entryHour(ctrl.entries.value[entryKey(task.id, day.day)]) > 0
                        ? 'border-brand bg-emerald-50 text-brand'
                        : 'border-divider bg-panel text-muted',
                      ctrl.isEditable.value
                        ? entryHour(ctrl.entries.value[entryKey(task.id, day.day)]) > 0
                          ? 'hover:bg-emerald-100'
                          : 'hover:border-brand hover:text-brand'
                        : 'cursor-not-allowed opacity-60',
                    ]"
                    type="button"
                    :disabled="!ctrl.isEditable.value"
                    :title="ctrl.isEditable.value
                      ? `${task.name} - ${day.label} ${day.weekday}`
                      : `${task.name} - ${day.label} ${day.weekday} (read-only)`"
                    @click="openEditDialog(task, day)"
                  >
                    <template v-if="entryHour(ctrl.entries.value[entryKey(task.id, day.day)]) > 0">
                      {{ formatHoursDisplay(entryHour(ctrl.entries.value[entryKey(task.id, day.day)])) }}
                    </template>
                    <i v-else-if="ctrl.isEditable.value" class="pi pi-plus" />
                    <span v-else class="text-muted">-</span>
                  </button>
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
            {{ editingCell.task.name }} - {{ editingCell.day.label }} {{ editingCell.day.weekday }}
          </p>
        </div>
      </template>

      <div class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Hour</span>
          <input
            v-model="editForm.hour"
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            inputmode="decimal"
            max="24"
            min="0"
            step="0.25"
            type="number"
          />
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
            <input
              v-model="editForm.regularOt"
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              inputmode="decimal"
              max="24"
              min="0"
              step="0.25"
              type="number"
            />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Midnight OT</span>
            <input
              v-model="editForm.midnightOt"
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              inputmode="decimal"
              max="24"
              min="0"
              step="0.25"
              type="number"
            />
          </label>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-between gap-3">
          <button
            class="h-10 rounded-md border border-red-200 bg-panel px-4 text-sm font-bold text-red-600 hover:bg-red-50"
            type="button"
            @click="clearEntry"
          >
            Clear
          </button>
          <div class="flex items-center gap-2">
            <button
              class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
              type="button"
              @click="editingCell = null"
            >
              Cancel
            </button>
            <button
              class="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
              type="button"
              @click="saveEntry"
            >
              Save
            </button>
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
          <input
            v-model="projectKeyword"
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
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
              'flex cursor-pointer items-center gap-3 rounded-md border px-4 py-3 hover:border-brand hover:bg-emerald-50',
              selectedProjectIds.includes(project.id) ? 'border-brand bg-emerald-50' : 'border-divider bg-panel',
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
            <button
              class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
              type="button"
              @click="isAddingProject = false"
            >
              Cancel
            </button>
            <button
              class="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
              type="button"
              :disabled="selectedProjectIds.length === 0"
              @click="confirmAddProjects"
            >
              Add projects
            </button>
          </div>
        </div>
      </template>
    </Dialog>

    <!-- New task dialog -->
    <Dialog
      :visible="isAddingTask"
      class="w-full max-w-lg rounded-lg bg-panel shadow-xl"
      :style="{ maxHeight: '90vh' }"
      modal
      @update:visible="isAddingTask = $event"
    >
      <template #header>
        <div class="min-w-0">
          <h3 class="truncate font-bold text-ink">New Task</h3>
          <p v-if="taskTargetProject" class="mt-1 truncate text-sm text-muted">
            {{ taskTargetProject.code }} - {{ taskTargetProject.name }}
          </p>
        </div>
      </template>

      <div class="space-y-4">
        <!-- Short name -->
        <label class="block">
          <span class="text-xs font-bold text-muted">Short name <span class="text-red-500">*</span></span>
          <input
            v-model="taskForm.shortName"
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
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

        <!-- Task category (multi-select) -->
        <div>
          <span class="text-xs font-bold text-muted">Phân loại task</span>
          <div class="mt-1 flex flex-wrap gap-2">
            <label
              v-for="category in TASK_CATEGORIES"
              :key="category"
              :class="[
                'flex cursor-pointer items-center gap-2 rounded-md border px-3 py-2 text-sm font-semibold transition',
                taskForm.categories.includes(category)
                  ? 'border-brand bg-emerald-50 text-brand'
                  : 'border-divider bg-panel text-secondary hover:border-brand',
              ]"
            >
              <Checkbox
                :model-value="taskForm.categories.includes(category)"
                :binary="true"
                class="h-4 w-4 accent-brand"
                @update:model-value="toggleTaskCategory(category)"
              />
              {{ category }}
            </label>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <!-- Assignee -->
          <label class="block">
            <span class="text-xs font-bold text-muted">Assignee</span>
            <input
              v-model="taskForm.assignee"
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Username"
            />
          </label>

          <!-- Estimate hour -->
          <label class="block">
            <span class="text-xs font-bold text-muted">Estimate Hour</span>
            <input
              v-model="taskForm.estimateHour"
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              inputmode="decimal"
              min="0"
              step="0.25"
              type="number"
              placeholder="0"
            />
          </label>

          <!-- Due date -->
          <label class="block">
            <span class="text-xs font-bold text-muted">Due Date</span>
            <input
              v-model="taskForm.dueDate"
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              type="date"
            />
          </label>

          <!-- Backlog issue key -->
          <label class="block">
            <span class="text-xs font-bold text-muted">Link Issue Backlog</span>
            <input
              v-model="taskForm.issueKey"
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Issue Key"
            />
          </label>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <button
            class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            type="button"
            @click="isAddingTask = false"
          >
            Cancel
          </button>
          <button
            class="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            :disabled="!canSaveTask"
            @click="saveTask"
          >
            Add task
          </button>
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
      <button
        class="flex w-full items-center gap-2 px-3 py-2 text-left font-semibold hover:bg-canvas"
        type="button"
        @click="openTaskDialog"
      >
        <i class="pi pi-plus" />
        <span class="min-w-0 truncate">Thêm task</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-3 py-2 text-left font-semibold hover:bg-canvas"
        type="button"
        @click="openBacklog"
      >
        <i class="pi pi-folder-open" />
        <span class="min-w-0 truncate">Xem backlog</span>
      </button>
      <button class="flex w-full items-center gap-2 px-3 py-2 text-left font-semibold hover:bg-canvas" type="button">
        <i class="pi pi-upload" />
        <span class="min-w-0 truncate">Import task</span>
      </button>
      <template v-if="contextMenu.canDelete">
        <div class="my-1 border-t border-divider" />
        <button
          class="flex w-full items-center gap-2 px-3 py-2 text-left font-semibold text-red-600 hover:bg-red-50"
          type="button"
          @click="deleteProject"
        >
          <i class="pi pi-trash" />
          <span class="min-w-0 truncate">Xóa project</span>
        </button>
      </template>
    </div>
  </section>
</template>
