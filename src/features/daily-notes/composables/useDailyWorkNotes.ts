import { computed, ref, watch } from "vue";
import { friendlyError } from "@/tauri/commands/_base";
import {
  createDailyNote,
  deleteDailyNote,
  getDailyNoteCounts,
  getDailyNotesByDate,
  updateDailyNoteContent,
  updateDailyNoteStatus,
} from "@/tauri/commands/daily-note";
import type { DailyWorkNoteResult, DailyNoteDateCountResult } from "@/_/types/daily-note";

export type DailyWorkStatus = "completed" | "incomplete" | "reserved";

export type DailyWorkNote = {
  id: number;
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

function toNote(r: DailyWorkNoteResult): DailyWorkNote {
  return {
    id: r.id,
    content: r.content,
    date: r.note_date,
    status: r.status as DailyWorkStatus,
    createdAt: r.created_at,
  };
}

export function useDailyWorkNotes(username?: string) {
  const today = startOfDay(new Date());
  const maxEntryDate = addDays(today, 7);
  const selectedDate = ref(formatDate(today));
  const visibleMonth = ref(startOfMonth(today));
  const notes = ref<DailyWorkNote[]>([]);
  const dateCounts = ref<DailyNoteDateCountResult[]>([]);
  const statusFilter = ref<DailyWorkStatus>("completed");
  const isLoading = ref(false);
  const isSaving = ref(false);
  const error = ref("");

  async function loadNotesByDate() {
    if (!username) return;
    isLoading.value = true;
    error.value = "";
    try {
      const results = await getDailyNotesByDate(username, selectedDate.value);
      notes.value = results.map(toNote);
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadDateCounts() {
    if (!username) return;
    try {
      const m = visibleMonth.value;
      dateCounts.value = await getDailyNoteCounts(username, m.getFullYear(), m.getMonth() + 1);
    } catch {
      dateCounts.value = [];
    }
  }

  loadNotesByDate();
  loadDateCounts();

  watch(selectedDate, () => {
    loadNotesByDate();
  });

  watch(visibleMonth, () => {
    loadDateCounts();
  });

  const calendarDays = computed(() =>
    buildCalendarDays(visibleMonth.value, selectedDate.value, dateCounts.value, maxEntryDate),
  );

  const selectedDateNotes = computed(() =>
    notes.value.slice().sort((a, b) => b.createdAt.localeCompare(a.createdAt)),
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

  async function addNote(draft: DailyWorkNoteDraft): Promise<boolean> {
    if (!username || !draft.date) return false;
    const noteDate = startOfDay(draft.date);
    if (noteDate > maxEntryDate) return false;
    const trimmedContent = draft.content.trim();
    if (!trimmedContent) return false;

    isSaving.value = true;
    error.value = "";
    try {
      await createDailyNote(username, {
        content: trimmedContent,
        note_date: formatDate(noteDate),
        status: draft.status,
      });
      selectDate(formatDate(noteDate));
      statusFilter.value = draft.status;
      await loadNotesByDate();
      await loadDateCounts();
      return true;
    } catch (e) {
      error.value = friendlyError(e);
      return false;
    } finally {
      isSaving.value = false;
    }
  }

  async function updateNoteContent(noteId: number, content: string): Promise<boolean> {
    if (!username) return false;
    const trimmed = content.trim();
    if (!trimmed) return false;
    error.value = "";
    try {
      await updateDailyNoteContent(noteId, username, trimmed);
      notes.value = notes.value.map((n) => (n.id === noteId ? { ...n, content: trimmed } : n));
      return true;
    } catch (e) {
      error.value = friendlyError(e);
      return false;
    }
  }

  async function updateNoteStatus(noteId: number, status: DailyWorkStatus) {
    if (!username) return;
    error.value = "";
    try {
      await updateDailyNoteStatus(noteId, username, status);
      notes.value = notes.value.map((n) => (n.id === noteId ? { ...n, status } : n));
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  async function removeNote(noteId: number) {
    if (!username) return;
    error.value = "";
    try {
      await deleteDailyNote(noteId, username);
      notes.value = notes.value.filter((n) => n.id !== noteId);
      await loadDateCounts();
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  return {
    addNote,
    calendarDays,
    error,
    filteredNotes,
    isLoading,
    isSaving,
    maxEntryDate,
    monthLabel,
    monthValue,
    nextMonth: nextMonthFn,
    previousMonth,
    removeNote,
    selectDate,
    updateNoteContent,
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
  dateCounts: DailyNoteDateCountResult[],
  maxEntryDate: Date,
): DailyWorkCalendarDay[] {
  const firstDay = startOfMonth(month);
  const calendarStart = addDays(firstDay, -firstDay.getDay());
  const today = startOfDay(new Date());
  const countMap = new Map(dateCounts.map((c) => [c.note_date, c.note_count]));

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
      taskCount: countMap.get(dateValue) ?? 0,
    };
  });
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
