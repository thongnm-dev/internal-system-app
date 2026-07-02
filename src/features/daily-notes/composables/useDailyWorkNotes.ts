import { computed, ref, watch } from "vue";

export type DailyWorkStatus = "completed" | "incomplete" | "reserved";

export type DailyWorkNote = {
  id: string;
  content: string;
  date: string;
  status: DailyWorkStatus;
  createdAt: string;
};

export type DailyWorkCalendarDay = {
  date: string;
  day: number;
  isCurrentMonth: boolean;
  isFutureDisabled: boolean;
  isSelected: boolean;
  isToday: boolean;
  taskCount: number;
};

export type DailyWorkNoteDraft = {
  content: string;
  date: Date | null;
  status: DailyWorkStatus;
};

const statusOrder: DailyWorkStatus[] = ["completed", "incomplete", "reserved"];

export function useDailyWorkNotes(username?: string) {
  const today = startOfDay(new Date());
  const maxEntryDate = addDays(today, 7);
  const selectedDate = ref(formatDate(today));
  const visibleMonth = ref(startOfMonth(today));
  const notes = ref<DailyWorkNote[]>(loadNotes(username));
  const statusFilter = ref<DailyWorkStatus>("completed");

  watch(
    () => username,
    () => {
      notes.value = loadNotes(username);
    },
  );

  const calendarDays = computed(() =>
    buildCalendarDays(visibleMonth.value, selectedDate.value, notes.value, maxEntryDate),
  );

  const selectedDateNotes = computed(() =>
    notes.value
      .filter((n) => n.date === selectedDate.value)
      .sort((a, b) => b.createdAt.localeCompare(a.createdAt)),
  );

  const filteredNotes = computed(() =>
    selectedDateNotes.value.filter((n) => n.status === statusFilter.value),
  );

  const statusCounts = computed(() =>
    statusOrder.reduce<Record<DailyWorkStatus, number>>(
      (counts, status) => ({
        ...counts,
        [status]: selectedDateNotes.value.filter((n) => n.status === status).length,
      }),
      { completed: 0, incomplete: 0, reserved: 0 },
    ),
  );

  const monthLabel = computed(() =>
    visibleMonth.value.toLocaleDateString("vi-VN", { month: "long", year: "numeric" }),
  );

  const monthValue = computed(() => formatMonthValue(visibleMonth.value));

  const selectedDateLabel = computed(() => formatDisplayDate(selectedDate.value));

  const totalSelectedDateNotes = computed(() => selectedDateNotes.value.length);

  function selectDate(date: string) {
    selectedDate.value = date;
    const nextDate = parseDateValue(date);
    if (nextDate) {
      visibleMonth.value = startOfMonth(nextDate);
    }
  }

  function selectMonth(value: string) {
    const nextMonth = parseMonthValue(value);
    if (nextMonth) {
      visibleMonth.value = nextMonth;
    }
  }

  function previousMonth() {
    const m = visibleMonth.value;
    visibleMonth.value = new Date(m.getFullYear(), m.getMonth() - 1, 1);
  }

  function nextMonthFn() {
    const m = visibleMonth.value;
    visibleMonth.value = new Date(m.getFullYear(), m.getMonth() + 1, 1);
  }

  function addNote(draft: DailyWorkNoteDraft): boolean {
    if (!draft.date) return false;
    const noteDate = startOfDay(draft.date);
    if (noteDate > maxEntryDate) return false;
    const trimmedContent = draft.content.trim();
    if (!trimmedContent) return false;

    const nextNote: DailyWorkNote = {
      id: makeNoteId(),
      content: trimmedContent,
      date: formatDate(noteDate),
      status: draft.status,
      createdAt: new Date().toISOString(),
    };

    notes.value = [nextNote, ...notes.value];
    saveNotes(username, notes.value);
    selectDate(nextNote.date);
    statusFilter.value = nextNote.status;
    return true;
  }

  function updateNoteStatus(noteId: string, status: DailyWorkStatus) {
    notes.value = notes.value.map((n) => (n.id === noteId ? { ...n, status } : n));
    saveNotes(username, notes.value);
  }

  function removeNote(noteId: string) {
    notes.value = notes.value.filter((n) => n.id !== noteId);
    saveNotes(username, notes.value);
  }

  return {
    addNote,
    calendarDays,
    filteredNotes,
    maxEntryDate,
    monthLabel,
    monthValue,
    nextMonth: nextMonthFn,
    previousMonth,
    removeNote,
    selectDate,
    selectMonth,
    selectedDate,
    selectedDateLabel,
    statusCounts,
    statusFilter,
    today,
    totalSelectedDateNotes,
    updateNoteStatus,
  };
}

function buildCalendarDays(
  month: Date,
  selectedDate: string,
  notes: DailyWorkNote[],
  maxEntryDate: Date,
): DailyWorkCalendarDay[] {
  const firstDay = startOfMonth(month);
  const calendarStart = addDays(firstDay, -firstDay.getDay());
  const today = startOfDay(new Date());

  return Array.from({ length: 42 }, (_, index) => {
    const date = addDays(calendarStart, index);
    const dateValue = formatDate(date);
    return {
      date: dateValue,
      day: date.getDate(),
      isCurrentMonth: date.getMonth() === month.getMonth() && date.getFullYear() === month.getFullYear(),
      isFutureDisabled: date > maxEntryDate,
      isSelected: dateValue === selectedDate,
      isToday: isSameDate(date, today),
      taskCount: notes.filter((n) => n.date === dateValue).length,
    };
  });
}

function storageKey(username: string | undefined) {
  return `pjjyuji.daily-work-notes.${username || "guest"}`;
}

function loadNotes(username: string | undefined): DailyWorkNote[] {
  try {
    const saved = window.localStorage.getItem(storageKey(username));
    return saved ? (JSON.parse(saved) as DailyWorkNote[]) : [];
  } catch {
    return [];
  }
}

function saveNotes(username: string | undefined, notes: DailyWorkNote[]) {
  window.localStorage.setItem(storageKey(username), JSON.stringify(notes));
}

function makeNoteId() {
  return `note-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function startOfDay(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

function startOfMonth(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

function addDays(date: Date, days: number) {
  const nextDate = new Date(date);
  nextDate.setDate(date.getDate() + days);
  return startOfDay(nextDate);
}

function isSameDate(left: Date, right: Date) {
  return (
    left.getFullYear() === right.getFullYear() &&
    left.getMonth() === right.getMonth() &&
    left.getDate() === right.getDate()
  );
}

function formatDate(date: Date) {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

function formatMonthValue(date: Date) {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
}

function parseDateValue(value: string) {
  const [y, m, d] = value.split("-").map(Number);
  if (!Number.isInteger(y) || !Number.isInteger(m) || !Number.isInteger(d)) return null;
  return new Date(y, m - 1, d);
}

function parseMonthValue(value: string) {
  const [y, m] = value.split("-").map(Number);
  if (!Number.isInteger(y) || !Number.isInteger(m) || m < 1 || m > 12) return null;
  return new Date(y, m - 1, 1);
}

function formatDisplayDate(value: string) {
  const date = parseDateValue(value);
  return date
    ? date.toLocaleDateString("vi-VN", { weekday: "long", day: "2-digit", month: "2-digit", year: "numeric" })
    : value;
}
