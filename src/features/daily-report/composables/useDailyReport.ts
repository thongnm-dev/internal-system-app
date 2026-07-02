import { computed, ref, watch } from "vue";

export type DailyReportTask = {
  id: string;
  code: string;
  name: string;
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

const assignedProjects: DailyReportProject[] = [
  {
    id: "pj-yuji",
    code: "YUJI",
    name: "PJ Yuji Internal Tool",
    client: "Internal",
    tasks: [
      { id: "yuji-planning", code: "PLAN", name: "Planning and daily coordination" },
      { id: "yuji-frontend", code: "FE", name: "Frontend implementation" },
      { id: "yuji-review", code: "REV", name: "Code review and QA support" },
    ],
  },
  {
    id: "hr-portal",
    code: "HRP",
    name: "HR Portal",
    client: "Operations",
    tasks: [
      { id: "hrp-maintenance", code: "MNT", name: "Maintenance requests" },
      { id: "hrp-bugfix", code: "BUG", name: "Bug fixing" },
    ],
  },
  {
    id: "sales-dashboard",
    code: "SALE",
    name: "Sales Dashboard",
    client: "Business",
    tasks: [
      { id: "sale-reporting", code: "RPT", name: "Report screen updates" },
      { id: "sale-data", code: "DATA", name: "Data reconciliation" },
      { id: "sale-meeting", code: "MTG", name: "Stakeholder meeting" },
    ],
  },
];

const optionalProjects: DailyReportProject[] = [
  {
    id: "mobile-app",
    code: "MOB",
    name: "Mobile Companion App",
    client: "Product",
    tasks: [
      { id: "mob-api", code: "API", name: "API integration" },
      { id: "mob-test", code: "TEST", name: "Device testing" },
    ],
  },
  {
    id: "infra-support",
    code: "OPS",
    name: "Infrastructure Support",
    client: "Platform",
    tasks: [
      { id: "ops-monitoring", code: "MON", name: "Monitoring and alert handling" },
      { id: "ops-release", code: "REL", name: "Release support" },
    ],
  },
];

export function useDailyReport(username?: string) {
  const visibleProjectIds = ref<string[]>(assignedProjects.map((p) => p.id));
  const selectedProjectId = ref(assignedProjects[0]?.id ?? "");
  const now = new Date();
  const currentMonth = startOfMonth(now);
  const selectedMonth = ref(startOfMonth(now));
  const entries = ref<DailyReportEntries>(loadEntries(username, startOfMonth(now)));

  const allProjects = [...assignedProjects, ...optionalProjects];

  const projects = computed(() =>
    allProjects
      .filter((p) => visibleProjectIds.value.includes(p.id))
      .map((p) => ({
        ...p,
        isUserAdded: optionalProjects.some((op) => op.id === p.id),
      })),
  );

  const selectedProject = computed(
    () => projects.value.find((p) => p.id === selectedProjectId.value) ?? projects.value[0] ?? null,
  );

  const availableProjects = computed(() =>
    optionalProjects.filter((p) => !visibleProjectIds.value.includes(p.id)),
  );

  const year = computed(() => selectedMonth.value.getFullYear());
  const monthIndex = computed(() => selectedMonth.value.getMonth());

  const days = computed<DailyReportDay[]>(() => getMonthDays(year.value, monthIndex.value));

  const canGoNextMonth = computed(() => selectedMonth.value < currentMonth);

  const monthLabel = computed(() =>
    selectedMonth.value.toLocaleDateString("en-US", { month: "long", year: "numeric" }),
  );

  const monthValue = computed(() => formatMonthValue(selectedMonth.value));

  const maxMonthValue = computed(() => formatMonthValue(currentMonth));

  const storageKeyValue = computed(() => dailyReportStorageKey(username, selectedMonth.value));

  watch([selectedMonth, () => username], () => {
    entries.value = loadEntries(username, selectedMonth.value);
  });

  function addProject(projectId: string) {
    if (!visibleProjectIds.value.includes(projectId)) {
      visibleProjectIds.value = [...visibleProjectIds.value, projectId];
    }
    selectedProjectId.value = projectId;
  }

  function removeProject(projectId: string) {
    visibleProjectIds.value = visibleProjectIds.value.filter((id) => id !== projectId);
    if (selectedProjectId.value === projectId) {
      selectedProjectId.value = visibleProjectIds.value[0] ?? "";
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

  function updateEntry(taskId: string, day: number, value: DailyReportEntry | null) {
    const normalized = value ? normalizeEntry(value) : null;
    const key = entryKey(taskId, day);
    if (!normalized) {
      const { [key]: _removed, ...rest } = entries.value;
      entries.value = rest;
    } else {
      entries.value = { ...entries.value, [key]: normalized };
    }
    window.localStorage.setItem(storageKeyValue.value, JSON.stringify(entries.value));
  }

  function totalHours(taskId?: string): number {
    const prefix = taskId ? `${taskId}:` : "";
    return Object.entries(entries.value).reduce((total, [key, value]) => {
      if (taskId && !key.startsWith(prefix)) return total;
      return total + entryHour(value);
    }, 0);
  }

  return {
    addProject,
    availableProjects,
    canGoNextMonth,
    days,
    entries,
    maxMonthValue,
    monthLabel,
    monthValue,
    nextMonth: nextMonthFn,
    previousMonth,
    projects,
    removeProject,
    selectMonth,
    selectedProject,
    selectedProjectId,
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

function parseMonthValue(value: string) {
  const [y, m] = value.split("-").map(Number);
  if (!Number.isInteger(y) || !Number.isInteger(m) || m < 1 || m > 12) return null;
  return new Date(y, m - 1, 1);
}

function dailyReportStorageKey(username: string | undefined, month: Date) {
  return `pjjyuji.daily-report.${username || "guest"}.${formatMonthValue(month)}`;
}

function loadEntries(username: string | undefined, month: Date): DailyReportEntries {
  try {
    const saved = window.localStorage.getItem(dailyReportStorageKey(username, month));
    return saved ? (JSON.parse(saved) as DailyReportEntries) : {};
  } catch {
    return {};
  }
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
