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
  deleteDailyReportTask,
  setDailyReportTaskCompleted,
  setProjectTaskCompleted,
} from "@/tauri/commands/daily-report";
import type {
  DailyReportEntryResult,
  DailyReportPhaseResult,
  DailyReportProjectResult,
  DailyReportUserTaskResult,
} from "@/_/types/daily-report";

export type TaskCategory = string;

export type DailyReportTask = {
  id: string;
  dbId: number;
  categoryId: number;
  name: string;
  description?: string;
  assignee?: string;
  estimateHour?: string;
  dueDate?: string;
  issueKey?: string;
  isUserAdded?: boolean;
  isCompleted?: boolean;
  completedAt?: string;
};

export type DailyReportTaskRow = DailyReportTask & {
  rowId: string;
  category: string;
  categoryLabel: string;
};

export type NewTaskInput = {
  shortName: string;
  description: string;
  category: TaskCategory;
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
  tasks: DailyReportTaskRow[];
};

export type DailyReportEntry = {
  comment: string;
  hour: string;
  isOt: boolean;
  midnightOt: string;
  categoryId: number;
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

export function makeRowId(taskId: string, category: string): string {
  return `${taskId}|${category}`;
}

function parseRowId(rowId: string): { taskId: string; category: string } {
  const idx = rowId.lastIndexOf("|");
  if (idx < 0) return { taskId: rowId, category: "" };
  return { taskId: rowId.substring(0, idx), category: rowId.substring(idx + 1) };
}

function taskToRow(
  task: DailyReportTask,
  phaseMap: Map<string, string>,
  idToCodeMap: Map<number, string>,
): DailyReportTaskRow {
  const code = idToCodeMap.get(task.categoryId) ?? "";
  return {
    ...task,
    rowId: makeRowId(task.id, code),
    category: code,
    categoryLabel: phaseMap.get(code) ?? code,
  };
}

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
  const taskHoursTotal = ref<Record<string, number>>({});
  const phases = ref<{ id: number; code: string; name: string; shortName: string }[]>([]);
  const categoryIdToCode = computed(() => new Map(phases.value.map((p) => [p.id, p.code])));
  const categoryCodeToId = computed(() => new Map(phases.value.map((p) => [p.code, p.id])));
  const isLoading = ref(false);
  const error = ref("");

  function projectIdStr(p: DailyReportProjectResult): string {
    return String(p.id);
  }

  function toFrontendProject(p: DailyReportProjectResult, isUserAdded: boolean): DailyReportProject {
    const id = projectIdStr(p);
    const rawTasks = userTasks.value[id] ?? [];
    const phaseMap = new Map(phases.value.map((ph) => [ph.code, ph.shortName]));
    return {
      id,
      code: p.code,
      name: p.name,
      client: p.client,
      isUserAdded,
      tasks: rawTasks.map((t) => taskToRow(t, phaseMap, categoryIdToCode.value)),
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
        const rowId = makeRowId(row.task_id, categoryIdToCode.value.get(row.category_id) ?? "");
        acc[entryKey(rowId, dayOfDate(row.entry_date))] = toFrontendEntry(row);
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
      phases.value = rows.map((r) => ({ id: r.id, code: r.process_code, name: r.process_name, shortName: r.short_name }));
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

    if (!username) return null;

    try {
      const saved = await createDailyReportTask(username, {
        project_id: projectId,
        name: shortName,
        description: input.description.trim(),
        category_id: categoryCodeToId.value.get(input.category) ?? 0,
        assignee: input.assignee.trim() || username,
        estimate_hour: String(input.estimateHour ?? "").trim(),
        due_date: input.dueDate,
        issue_key: input.issueKey.trim(),
      });
      const persisted = toFrontendTask(saved);
      const existing = userTasks.value[projectId] ?? [];
      userTasks.value = { ...userTasks.value, [projectId]: [...existing, persisted] };
      if (!visibleOptionalIds.value.includes(projectId) && optionalProjects.value.some((p) => projectIdStr(p) === projectId)) {
        visibleOptionalIds.value = [...visibleOptionalIds.value, projectId];
      }
      return persisted;
    } catch (e) {
      error.value = friendlyError(e);
      return null;
    }
  }

  async function setTaskCompleted(taskId: string, isCompleted: boolean) {
    error.value = "";
    const projectId = projectIdOfTask(taskId);
    if (!projectId) return;
    const rawTask = (userTasks.value[projectId] ?? []).find((t) => t.id === taskId);
    const isUserAdded = rawTask?.isUserAdded ?? true;

    const applyFlag = (flag: boolean) => {
      userTasks.value = {
        ...userTasks.value,
        [projectId]: (userTasks.value[projectId] ?? []).map((t) =>
          t.id === taskId ? { ...t, isCompleted: flag } : t,
        ),
      };
    };

    applyFlag(isCompleted);
    if (!username) return;

    try {
      if (isUserAdded) {
        await setDailyReportTaskCompleted(username, rawTask!.dbId, isCompleted);
      } else {
        await setProjectTaskCompleted(taskId, isCompleted);
      }
    } catch (e) {
      error.value = friendlyError(e);
      applyFlag(!isCompleted);
    }
  }

  function migrateEntryKeys(taskId: string, oldCategory: string, newCategory: string) {
    if (oldCategory === newCategory) return;
    const oldRowId = makeRowId(taskId, oldCategory);
    const newRowId = makeRowId(taskId, newCategory);
    const updated = { ...entries.value };
    const oldPrefix = `${oldRowId}:`;
    for (const key of Object.keys(updated)) {
      if (key.startsWith(oldPrefix)) {
        const day = key.substring(oldPrefix.length);
        updated[`${newRowId}:${day}`] = updated[key];
        delete updated[key];
      }
    }
    entries.value = updated;
  }

  async function updateTask(projectId: string, taskId: string, input: NewTaskInput): Promise<DailyReportTask | null> {
    error.value = "";
    const shortName = input.shortName.trim();
    if (!shortName) return null;
    const tasks = userTasks.value[projectId] ?? [];
    const existing = tasks.find((t) => t.id === taskId);
    if (!existing) return null;
    const oldCategory = categoryIdToCode.value.get(existing.categoryId) ?? "";
    const newCategory = input.category ?? "";
    const newCategoryId = categoryCodeToId.value.get(newCategory) ?? 0;
    const updated: DailyReportTask = {
      ...existing,
      name: shortName,
      description: input.description.trim(),
      categoryId: newCategoryId,
      assignee: input.assignee.trim() || existing.assignee || (username ?? ""),
      estimateHour: String(input.estimateHour ?? "").trim(),
      dueDate: input.dueDate,
      issueKey: input.issueKey.trim(),
    };
    migrateEntryKeys(taskId, oldCategory, newCategory);
    userTasks.value = {
      ...userTasks.value,
      [projectId]: tasks.map((t) => (t.id === taskId ? updated : t)),
    };
    if (username && existing.isUserAdded) {
      try {
        const saved = await createDailyReportTask(username, {
          id: existing.dbId,
          project_id: projectId,
          name: updated.name,
          description: updated.description ?? "",
          category_id: newCategoryId,
          assignee: updated.assignee ?? "",
          estimate_hour: updated.estimateHour ?? "",
          due_date: updated.dueDate ?? "",
          issue_key: updated.issueKey ?? "",
        });
        const persisted = toFrontendTask(saved);
        userTasks.value = {
          ...userTasks.value,
          [projectId]: (userTasks.value[projectId] ?? []).map((t) => (t.id === taskId ? persisted : t)),
        };
        return persisted;
      } catch (e) {
        migrateEntryKeys(taskId, newCategory, oldCategory);
        userTasks.value = { ...userTasks.value, [projectId]: tasks };
        error.value = friendlyError(e);
        throw new Error(error.value);
      }
    }
    return updated;
  }

  async function removeTask(projectId: string, taskId: string) {
    const tasks = userTasks.value[projectId] ?? [];
    const task = tasks.find((t) => t.id === taskId);
    if (!task) throw new Error("Task không tồn tại.");
    if (!task.isUserAdded) throw new Error("Chỉ có thể xóa task được thêm nhanh.");
    if (totalHours(taskId) > 0) throw new Error("Không thể xóa task đã phát sinh số giờ.");
    error.value = "";
    try {
      if (username) await deleteDailyReportTask(username, task.dbId);
      userTasks.value = {
        ...userTasks.value,
        [projectId]: tasks.filter((t) => t.id !== taskId),
      };
    } catch (e) {
      const msg = friendlyError(e);
      error.value = msg;
      throw new Error(msg);
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

  function bumpTaskHours(taskId: string, delta: number) {
    if (delta === 0) return;
    const next = (taskHoursTotal.value[taskId] ?? 0) + delta;
    taskHoursTotal.value = { ...taskHoursTotal.value, [taskId]: next < 0 ? 0 : next };
  }

  async function updateEntry(rowId: string, day: number, value: DailyReportEntry | null) {
    if (!isEditable.value) return;
    const { taskId, category } = parseRowId(rowId);
    const normalized = value ? normalizeEntry(value) : null;
    const key = entryKey(rowId, day);
    const entryDate = formatDate(year.value, monthIndex.value, day);
    const previousHour = entryHour(entries.value[key]);

    if (!normalized) {
      const { [key]: _removed, ...rest } = entries.value;
      entries.value = rest;
      bumpTaskHours(taskId, -previousHour);
      if (username) {
        try {
          await clearDailyReportEntry(username, taskId, entryDate, categoryCodeToId.value.get(category) ?? 0);
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
          category_id: categoryCodeToId.value.get(category) ?? 0,
        });
      } catch (e) {
        error.value = friendlyError(e);
      }
    }
  }

  function totalHours(rowId?: string): number {
    const prefix = rowId ? `${rowId}:` : "";
    return Object.entries(entries.value).reduce((total, [key, value]) => {
      if (rowId && !key.startsWith(prefix)) return total;
      return total + entryHour(value);
    }, 0);
  }

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
    removeTask,
    selectMonth,
    updateTask,
    selectedProject,
    selectedProjectId,
    setTaskCompleted,
    totalHours,
    updateEntry,
  };
}

export function entryKey(rowId: string, day: number) {
  return `${rowId}:${day}`;
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
    categoryId: row.category_id ?? 0,
    regularOt: row.is_ot ? numToStr(row.regular_ot) : "",
  };
}

function toFrontendTask(row: DailyReportUserTaskResult): DailyReportTask {
  return {
    id: row.task_id,
    dbId: row.id,
    categoryId: row.category_id ?? 0,
    name: row.name,
    description: row.description,
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
  const hourStr = String(value.hour ?? "").trim();
  if (!hourStr) return null;
  const numeric = Number(hourStr);
  if (!Number.isFinite(numeric) || numeric < 0) return null;
  const isOt = value.isOt;
  return {
    comment: String(value.comment ?? "").trim(),
    hour: String(Math.min(numeric, 24)),
    isOt,
    midnightOt: isOt ? normalizeOptionalHour(String(value.midnightOt ?? "")) : "",
    categoryId: value.categoryId ?? 0,
    regularOt: isOt ? normalizeOptionalHour(String(value.regularOt ?? "")) : "",
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
  return { comment: "", hour: "", isOt: false, midnightOt: "", categoryId: 0, regularOt: "" };
}

export function formatHoursDisplay(value: number): string {
  return Number.isInteger(value) ? value.toLocaleString("en-US") : String(parseFloat(value.toFixed(2)));
}

export function projectDayTotal(project: DailyReportProject, entries: DailyReportEntries, day: number): number {
  return project.tasks.reduce((t, task) => t + entryHour(entries[entryKey(task.rowId, day)]), 0);
}

export function projectTotal(project: DailyReportProject, entries: DailyReportEntries): number {
  return project.tasks.reduce(
    (t, task) =>
      t +
      Object.entries(entries).reduce(
        (tt, [key, val]) => (key.startsWith(`${task.rowId}:`) ? tt + entryHour(val) : tt),
        0,
      ),
    0,
  );
}

export function dayTotal(projects: DailyReportProject[], entries: DailyReportEntries, day: number): number {
  return projects.reduce((t, p) => t + projectDayTotal(p, entries, day), 0);
}
