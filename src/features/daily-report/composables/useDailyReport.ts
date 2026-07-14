import { computed, ref, watch } from "vue";
import { friendlyError } from "@/tauri/commands/_base";
import {
  clearDailyReportEntry,
  createDailyReportTask,
  getDailyReportEntries,
  getDailyReportPhases,
  getDailyReportProjects,
  getDailyReportTaskHours,
  getDailyReportTasks,
  saveDailyReportEntry,
  setDailyReportTaskCompleted,
  setProjectTaskCompleted,
} from "@/tauri/commands/daily-report";
import type {
  DailyReportEntryResult,
  DailyReportPhaseResult,
  DailyReportProjectResult,
  DailyReportUserTaskResult,
} from "@/_/types/daily-report";

export type TaskCategory = "PG" | "Review PG" | "UT" | "Review UT" | "Other";

export const TASK_CATEGORIES: TaskCategory[] = ["PG", "Review PG", "UT", "Review UT", "Other"];

export type DailyReportTask = {
  id: string;
  code: string;
  name: string;
  description?: string;
  categories?: TaskCategory[];
  assignee?: string;
  estimateHour?: string;
  dueDate?: string;
  issueKey?: string;
  isUserAdded?: boolean;
  isCompleted?: boolean;
  completedAt?: string;
};

export type NewTaskInput = {
  shortName: string;
  description: string;
  categories: TaskCategory[];
  assignee: string;
  estimateHour: string;
  dueDate: string;
  issueKey: string;
};

export type DailyReportProject = {
  id: string;
  code: string;
  name: string;
  client: string;
  isUserAdded?: boolean;
  tasks: DailyReportTask[];
};

export type DailyReportEntry = {
  comment: string;
  hour: string;
  isOt: boolean;
  midnightOt: string;
  phase: string;
  regularOt: string;
};

export type DailyReportEntries = Record<string, DailyReportEntry | string>;

export type DailyReportDay = {
  day: number;
  label: string;
  weekday: string;
  isWeekend: boolean;
  isToday: boolean;
};

export function useDailyReport(username?: string) {
  const assignedProjects = ref<DailyReportProjectResult[]>([]);
  const optionalProjects = ref<DailyReportProjectResult[]>([]);
  const visibleOptionalIds = ref<string[]>([]);
  const selectedProjectId = ref("");
  const now = new Date();
  const currentMonth = startOfMonth(now);
  const selectedMonth = ref(startOfMonth(now));
  const entries = ref<DailyReportEntries>({});
  const userTasks = ref<Record<string, DailyReportTask[]>>({});
  // Tổng giờ tích luỹ (mọi tháng) theo task_id — cho badge tiến độ so với estimate.
  const taskHoursTotal = ref<Record<string, number>>({});
  // Danh sách công đoạn (phase) cho dropdown khi nhập giờ. Mỗi phần tử là process_code (tên phase).
  const phases = ref<string[]>([]);
  const isLoading = ref(false);
  const error = ref("");

  function projectIdStr(p: DailyReportProjectResult): string {
    return String(p.id);
  }

  function toFrontendProject(p: DailyReportProjectResult, isUserAdded: boolean): DailyReportProject {
    const id = projectIdStr(p);
    return {
      id,
      code: p.code,
      name: p.name,
      client: p.client,
      isUserAdded,
      tasks: userTasks.value[id] ?? [],
    };
  }

  const projects = computed<DailyReportProject[]>(() => {
    const assigned = assignedProjects.value.map((p) => toFrontendProject(p, false));
    const optional = optionalProjects.value
      .filter((p) => visibleOptionalIds.value.includes(projectIdStr(p)))
      .map((p) => toFrontendProject(p, true));
    return [...assigned, ...optional];
  });

  const selectedProject = computed(
    () => projects.value.find((p) => p.id === selectedProjectId.value) ?? projects.value[0] ?? null,
  );

  const availableProjects = computed(() =>
    optionalProjects.value
      .filter((p) => !visibleOptionalIds.value.includes(projectIdStr(p)))
      .map((p) => ({ id: projectIdStr(p), code: p.code, name: p.name, client: p.client })),
  );

  const year = computed(() => selectedMonth.value.getFullYear());
  const monthIndex = computed(() => selectedMonth.value.getMonth());

  const days = computed<DailyReportDay[]>(() => getMonthDays(year.value, monthIndex.value));

  const canGoNextMonth = computed(() => selectedMonth.value < currentMonth);

  const editableFromMonth = new Date(currentMonth.getFullYear(), currentMonth.getMonth() - 1, 1);
  const isEditable = computed(() => selectedMonth.value >= editableFromMonth);

  const monthLabel = computed(() =>
    selectedMonth.value.toLocaleDateString("en-US", { month: "long", year: "numeric" }),
  );

  const monthValue = computed(() => formatMonthValue(selectedMonth.value));

  const maxMonthValue = computed(() => formatMonthValue(currentMonth));

  function projectIdOfTask(taskId: string): string {
    for (const project of projects.value) {
      if (project.tasks.some((t) => t.id === taskId)) return project.id;
    }
    return "";
  }

  async function loadProjects() {
    if (!username) return;
    try {
      const rows = await getDailyReportProjects(username);
      assignedProjects.value = rows.filter((p) => p.is_member);
      optionalProjects.value = rows.filter((p) => !p.is_member);
      if (!selectedProjectId.value && assignedProjects.value.length > 0) {
        selectedProjectId.value = projectIdStr(assignedProjects.value[0]);
      }
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  async function loadEntries() {
    entries.value = {};
    if (!username) return;
    isLoading.value = true;
    error.value = "";
    try {
      const rows = await getDailyReportEntries(username, year.value, monthIndex.value + 1);
      entries.value = rows.reduce<DailyReportEntries>((acc, row) => {
        acc[entryKey(row.task_id, dayOfDate(row.entry_date))] = toFrontendEntry(row);
        return acc;
      }, {});
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadUserTasks() {
    userTasks.value = {};
    if (!username) return;
    try {
      const rows = await getDailyReportTasks(username, year.value, monthIndex.value + 1);
      userTasks.value = rows.reduce<Record<string, DailyReportTask[]>>((acc, row) => {
        const list = acc[row.project_id] ?? (acc[row.project_id] = []);
        list.push(toFrontendTask(row));
        return acc;
      }, {});
      for (const projectId of Object.keys(userTasks.value)) {
        if (
          optionalProjects.value.some((p) => projectIdStr(p) === projectId) &&
          !visibleOptionalIds.value.includes(projectId)
        ) {
          visibleOptionalIds.value = [...visibleOptionalIds.value, projectId];
        }
      }
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  async function loadTaskHours() {
    if (!username) return;
    try {
      const rows = await getDailyReportTaskHours(username);
      taskHoursTotal.value = rows.reduce<Record<string, number>>((acc, row) => {
        acc[row.task_id] = row.total_hour;
        return acc;
      }, {});
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  async function loadPhases() {
    if (!username) return;
    try {
      const rows: DailyReportPhaseResult[] = await getDailyReportPhases();
      phases.value = rows.map((r) => r.process_code);
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  loadProjects().then(() => {
    loadUserTasks();
    loadEntries();
    loadTaskHours();
  });
  loadPhases();

  watch(selectedMonth, () => {
    loadUserTasks();
    loadEntries();
    loadTaskHours();
  });

  function addProject(projectId: string) {
    if (!visibleOptionalIds.value.includes(projectId)) {
      visibleOptionalIds.value = [...visibleOptionalIds.value, projectId];
    }
    selectedProjectId.value = projectId;
  }

  async function addTask(projectId: string, input: NewTaskInput): Promise<DailyReportTask | null> {
    error.value = "";
    const shortName = input.shortName.trim();
    if (!shortName) return null;
    const categories = input.categories.slice();
    const task: DailyReportTask = {
      id: `${projectId}-task-${Date.now()}`,
      code: categories[0] ?? "TASK",
      name: shortName,
      description: input.description.trim(),
      categories,
      assignee: input.assignee.trim() || (username ?? ""),
      estimateHour: input.estimateHour.trim(),
      dueDate: input.dueDate,
      issueKey: input.issueKey.trim(),
      isUserAdded: true,
    };
    const existing = userTasks.value[projectId] ?? [];
    userTasks.value = { ...userTasks.value, [projectId]: [...existing, task] };
    if (!visibleOptionalIds.value.includes(projectId) && optionalProjects.value.some((p) => projectIdStr(p) === projectId)) {
      visibleOptionalIds.value = [...visibleOptionalIds.value, projectId];
    }

    if (username) {
      try {
        const saved = await createDailyReportTask(username, {
          task_id: task.id,
          project_id: projectId,
          code: task.code,
          name: task.name,
          description: task.description ?? "",
          categories,
          assignee: task.assignee ?? "",
          estimate_hour: task.estimateHour ?? "",
          due_date: task.dueDate ?? "",
          issue_key: task.issueKey ?? "",
        });
        // Reconcile the optimistic entry with the persisted record so the grid
        // always reflects exactly what the backend stored (id, code, created_at…).
        const persisted = toFrontendTask(saved);
        userTasks.value = {
          ...userTasks.value,
          [projectId]: (userTasks.value[projectId] ?? []).map((t) => (t.id === task.id ? persisted : t)),
        };
        return persisted;
      } catch (e) {
        error.value = friendlyError(e);
        userTasks.value = {
          ...userTasks.value,
          [projectId]: (userTasks.value[projectId] ?? []).filter((t) => t.id !== task.id),
        };
        return null;
      }
    }
    return task;
  }

  async function setTaskCompleted(taskId: string, isCompleted: boolean) {
    error.value = "";
    const projectId = projectIdOfTask(taskId);
    if (!projectId) return;
    const task = (userTasks.value[projectId] ?? []).find((t) => t.id === taskId);
    const isUserAdded = task?.isUserAdded ?? true;

    const applyFlag = (flag: boolean) => {
      userTasks.value = {
        ...userTasks.value,
        [projectId]: (userTasks.value[projectId] ?? []).map((t) =>
          t.id === taskId ? { ...t, isCompleted: flag } : t,
        ),
      };
    };

    // Optimistic toggle; task chỉ thực sự biến mất khỏi carry-over ở lần load sau.
    applyFlag(isCompleted);
    if (!username) return;

    try {
      // Task user tự thêm -> daily_report_tasks; task được giao -> project_tasks.
      if (isUserAdded) {
        await setDailyReportTaskCompleted(username, taskId, isCompleted);
      } else {
        await setProjectTaskCompleted(taskId, isCompleted);
      }
    } catch (e) {
      error.value = friendlyError(e);
      applyFlag(!isCompleted);
    }
  }

  function removeProject(projectId: string) {
    visibleOptionalIds.value = visibleOptionalIds.value.filter((id) => id !== projectId);
    if (selectedProjectId.value === projectId) {
      selectedProjectId.value = projects.value[0]?.id ?? "";
    }
  }

  function selectMonth(value: string) {
    const next = parseMonthValue(value);
    if (!next || next > currentMonth) return;
    selectedMonth.value = next;
  }

  function previousMonth() {
    const m = selectedMonth.value;
    selectedMonth.value = new Date(m.getFullYear(), m.getMonth() - 1, 1);
  }

  function nextMonthFn() {
    const m = selectedMonth.value;
    const next = new Date(m.getFullYear(), m.getMonth() + 1, 1);
    if (next <= currentMonth) selectedMonth.value = next;
  }

  // Cập nhật tổng giờ tích luỹ theo delta để badge luôn khớp mà không cần query lại.
  function bumpTaskHours(taskId: string, delta: number) {
    if (delta === 0) return;
    const next = (taskHoursTotal.value[taskId] ?? 0) + delta;
    taskHoursTotal.value = { ...taskHoursTotal.value, [taskId]: next < 0 ? 0 : next };
  }

  async function updateEntry(taskId: string, day: number, value: DailyReportEntry | null) {
    if (!isEditable.value) return;
    const normalized = value ? normalizeEntry(value) : null;
    const key = entryKey(taskId, day);
    const entryDate = formatDate(year.value, monthIndex.value, day);
    const previousHour = entryHour(entries.value[key]);

    if (!normalized) {
      const { [key]: _removed, ...rest } = entries.value;
      entries.value = rest;
      bumpTaskHours(taskId, -previousHour);
      if (username) {
        try {
          await clearDailyReportEntry(username, taskId, entryDate);
        } catch (e) {
          error.value = friendlyError(e);
        }
      }
      return;
    }

    entries.value = { ...entries.value, [key]: normalized };
    bumpTaskHours(taskId, (Number(normalized.hour) || 0) - previousHour);
    if (username) {
      try {
        await saveDailyReportEntry(username, {
          task_id: taskId,
          project_id: projectIdOfTask(taskId),
          entry_date: entryDate,
          comment: normalized.comment,
          hour: Number(normalized.hour) || 0,
          is_ot: normalized.isOt,
          regular_ot: Number(normalized.regularOt) || 0,
          midnight_ot: Number(normalized.midnightOt) || 0,
          phase: normalized.phase,
        });
      } catch (e) {
        error.value = friendlyError(e);
      }
    }
  }

  function totalHours(taskId?: string): number {
    const prefix = taskId ? `${taskId}:` : "";
    return Object.entries(entries.value).reduce((total, [key, value]) => {
      if (taskId && !key.startsWith(prefix)) return total;
      return total + entryHour(value);
    }, 0);
  }

  /// Tổng giờ tích luỹ (mọi tháng) của một task, cho badge tiến độ.
  function cumulativeHours(taskId: string): number {
    return taskHoursTotal.value[taskId] ?? 0;
  }

  return {
    addProject,
    addTask,
    availableProjects,
    canGoNextMonth,
    cumulativeHours,
    days,
    entries,
    error,
    isEditable,
    isLoading,
    maxMonthValue,
    monthLabel,
    monthValue,
    nextMonth: nextMonthFn,
    phases,
    previousMonth,
    projects,
    removeProject,
    selectMonth,
    selectedProject,
    selectedProjectId,
    setTaskCompleted,
    totalHours,
    updateEntry,
  };
}

export function entryKey(taskId: string, day: number) {
  return `${taskId}:${day}`;
}

export function entryHour(entry: DailyReportEntry | string | undefined): number {
  if (!entry) return 0;
  return Number(typeof entry === "string" ? entry : entry.hour) || 0;
}

function toFrontendEntry(row: DailyReportEntryResult): DailyReportEntry {
  return {
    comment: row.comment ?? "",
    hour: numToStr(row.hour),
    isOt: row.is_ot,
    midnightOt: row.is_ot ? numToStr(row.midnight_ot) : "",
    phase: row.phase ?? "",
    regularOt: row.is_ot ? numToStr(row.regular_ot) : "",
  };
}

function toFrontendTask(row: DailyReportUserTaskResult): DailyReportTask {
  return {
    id: row.task_id,
    code: row.code,
    name: row.name,
    description: row.description,
    categories: (row.categories ?? []) as TaskCategory[],
    assignee: row.assignee,
    estimateHour: row.estimate_hour,
    dueDate: row.due_date,
    issueKey: row.issue_key,
    isUserAdded: row.is_user_added,
    isCompleted: row.is_completed,
    completedAt: row.completed_at,
  };
}

function numToStr(value: number): string {
  if (!Number.isFinite(value) || value === 0) return "";
  return String(value);
}

function dayOfDate(value: string): number {
  return Number(value.slice(8, 10));
}

function getMonthDays(year: number, monthIndex: number): DailyReportDay[] {
  const lastDay = new Date(year, monthIndex + 1, 0).getDate();
  const today = new Date();
  return Array.from({ length: lastDay }, (_, i) => {
    const date = new Date(year, monthIndex, i + 1);
    return {
      day: i + 1,
      label: String(i + 1).padStart(2, "0"),
      weekday: date.toLocaleDateString("en-US", { weekday: "short" }),
      isWeekend: date.getDay() === 0 || date.getDay() === 6,
      isToday: isSameDate(date, today),
    };
  });
}

function isSameDate(left: Date, right: Date) {
  return (
    left.getFullYear() === right.getFullYear() &&
    left.getMonth() === right.getMonth() &&
    left.getDate() === right.getDate()
  );
}

function startOfMonth(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

function formatMonthValue(date: Date) {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
}

function formatDate(year: number, monthIndex: number, day: number) {
  return `${year}-${String(monthIndex + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
}

function parseMonthValue(value: string) {
  const [y, m] = value.split("-").map(Number);
  if (!Number.isInteger(y) || !Number.isInteger(m) || m < 1 || m > 12) return null;
  return new Date(y, m - 1, 1);
}

function normalizeEntry(value: DailyReportEntry): DailyReportEntry | null {
  const trimmed = value.hour.trim();
  if (!trimmed) return null;
  const numeric = Number(trimmed);
  if (!Number.isFinite(numeric) || numeric < 0) return null;
  const isOt = value.isOt;
  return {
    comment: value.comment.trim(),
    hour: String(Math.min(numeric, 24)),
    isOt,
    midnightOt: isOt ? normalizeOptionalHour(value.midnightOt) : "",
    phase: value.phase.trim(),
    regularOt: isOt ? normalizeOptionalHour(value.regularOt) : "",
  };
}

function normalizeOptionalHour(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return "";
  const numeric = Number(trimmed);
  if (!Number.isFinite(numeric) || numeric < 0) return "";
  return String(Math.min(numeric, 24));
}

export function normalizeEntryForForm(entry: DailyReportEntry | string | undefined): DailyReportEntry {
  if (!entry) return emptyEntry();
  if (typeof entry === "string") return { ...emptyEntry(), hour: entry };
  return entry;
}

export function emptyEntry(): DailyReportEntry {
  return { comment: "", hour: "", isOt: false, midnightOt: "", phase: "", regularOt: "" };
}

export function formatHoursDisplay(value: number): string {
  return Number.isInteger(value) ? value.toLocaleString("en-US") : value.toFixed(2);
}

export function projectDayTotal(project: DailyReportProject, entries: DailyReportEntries, day: number): number {
  return project.tasks.reduce((t, task) => t + entryHour(entries[entryKey(task.id, day)]), 0);
}

export function projectTotal(project: DailyReportProject, entries: DailyReportEntries): number {
  return project.tasks.reduce(
    (t, task) =>
      t +
      Object.entries(entries).reduce(
        (tt, [key, val]) => (key.startsWith(`${task.id}:`) ? tt + entryHour(val) : tt),
        0,
      ),
    0,
  );
}

export function dayTotal(projects: DailyReportProject[], entries: DailyReportEntries, day: number): number {
  return projects.reduce((t, p) => t + projectDayTotal(p, entries, day), 0);
}
