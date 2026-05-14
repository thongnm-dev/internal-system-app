import { useEffect, useMemo, useState } from "react";

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

export function useDailyReportController(username?: string) {
  const [visibleProjectIds, setVisibleProjectIds] = useState(() => assignedProjects.map((project) => project.id));
  const [selectedProjectId, setSelectedProjectId] = useState(assignedProjects[0]?.id ?? "");
  const now = useMemo(() => new Date(), []);
  const [selectedMonth, setSelectedMonth] = useState(() => startOfMonth(now));
  const [entries, setEntries] = useState<DailyReportEntries>(() => loadEntries(username, startOfMonth(now)));

  const currentMonth = startOfMonth(now);
  const year = selectedMonth.getFullYear();
  const monthIndex = selectedMonth.getMonth();
  const storageKey = dailyReportStorageKey(username, selectedMonth);

  const allProjects = useMemo(() => [...assignedProjects, ...optionalProjects], []);
  const projects = allProjects.filter((project) => visibleProjectIds.includes(project.id));
  const selectedProject = projects.find((project) => project.id === selectedProjectId) ?? projects[0] ?? null;
  const availableProjects = optionalProjects.filter((project) => !visibleProjectIds.includes(project.id));
  const days = useMemo(() => getMonthDays(year, monthIndex), [monthIndex, year]);

  useEffect(() => {
    setEntries(loadEntries(username, selectedMonth));
  }, [selectedMonth, username]);

  const addProject = (projectId: string) => {
    setVisibleProjectIds((current) => (current.includes(projectId) ? current : [...current, projectId]));
    setSelectedProjectId(projectId);
  };

  const selectMonth = (value: string) => {
    const nextMonth = parseMonthValue(value);
    if (!nextMonth || nextMonth > currentMonth) {
      return;
    }

    setSelectedMonth(nextMonth);
  };

  const previousMonth = () => {
    setSelectedMonth((current) => new Date(current.getFullYear(), current.getMonth() - 1, 1));
  };

  const nextMonth = () => {
    setSelectedMonth((current) => {
      const next = new Date(current.getFullYear(), current.getMonth() + 1, 1);
      return next > currentMonth ? current : next;
    });
  };

  const updateEntry = (taskId: string, day: number, value: DailyReportEntry | null) => {
    const normalized = value ? normalizeEntry(value) : null;
    const key = entryKey(taskId, day);
    setEntries((current) => {
      let nextEntries: DailyReportEntries;
      if (!normalized) {
        const { [key]: _removed, ...rest } = current;
        nextEntries = rest;
      } else {
        nextEntries = {
          ...current,
          [key]: normalized,
        };
      }

      window.localStorage.setItem(storageKey, JSON.stringify(nextEntries));
      return nextEntries;
    });
  };

  const totalHours = (taskId?: string) => {
    const prefix = taskId ? `${taskId}:` : "";
    return Object.entries(entries).reduce((total, [key, value]) => {
      if (taskId && !key.startsWith(prefix)) {
        return total;
      }

      return total + entryHour(value);
    }, 0);
  };

  return {
    addProject,
    availableProjects,
    canGoNextMonth: selectedMonth < currentMonth,
    days,
    entries,
    maxMonthValue: formatMonthValue(currentMonth),
    monthLabel: selectedMonth.toLocaleDateString("en-US", { month: "long", year: "numeric" }),
    monthValue: formatMonthValue(selectedMonth),
    nextMonth,
    previousMonth,
    projects,
    selectMonth,
    selectedProject,
    selectedProjectId,
    setSelectedProjectId,
    totalHours,
    updateEntry,
  };
}

export function entryKey(taskId: string, day: number) {
  return `${taskId}:${day}`;
}

function getMonthDays(year: number, monthIndex: number) {
  const lastDay = new Date(year, monthIndex + 1, 0).getDate();
  return Array.from({ length: lastDay }, (_, index) => {
    const date = new Date(year, monthIndex, index + 1);
    return {
      day: index + 1,
      label: String(index + 1).padStart(2, "0"),
      weekday: date.toLocaleDateString("en-US", { weekday: "short" }),
      isWeekend: date.getDay() === 0 || date.getDay() === 6,
      isToday: isSameDate(date, new Date()),
    };
  });
}

function isSameDate(left: Date, right: Date) {
  return left.getFullYear() === right.getFullYear() && left.getMonth() === right.getMonth() && left.getDate() === right.getDate();
}

function startOfMonth(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

function formatMonthValue(date: Date) {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
}

function parseMonthValue(value: string) {
  const [yearText, monthText] = value.split("-");
  const year = Number(yearText);
  const month = Number(monthText);
  if (!Number.isInteger(year) || !Number.isInteger(month) || month < 1 || month > 12) {
    return null;
  }

  return new Date(year, month - 1, 1);
}

function dailyReportStorageKey(username: string | undefined, month: Date) {
  return `pjjyuji.daily-report.${username || "guest"}.${formatMonthValue(month)}`;
}

function loadEntries(username: string | undefined, month: Date): DailyReportEntries {
  try {
    const saved = window.localStorage.getItem(dailyReportStorageKey(username, month));
    return saved ? JSON.parse(saved) as DailyReportEntries : {};
  } catch {
    return {};
  }
}

export function entryHour(entry: DailyReportEntry | string | undefined) {
  if (!entry) {
    return 0;
  }

  return Number(typeof entry === "string" ? entry : entry.hour) || 0;
}

function normalizeEntry(value: DailyReportEntry) {
  const trimmed = value.hour.trim();
  if (!trimmed) {
    return null;
  }

  const numeric = Number(trimmed);
  if (!Number.isFinite(numeric) || numeric < 0) {
    return null;
  }

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
  if (!trimmed) {
    return "";
  }

  const numeric = Number(trimmed);
  if (!Number.isFinite(numeric) || numeric < 0) {
    return "";
  }

  return String(Math.min(numeric, 24));
}
