import { useEffect, useMemo, useState } from "react";

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

export function useDailyWorkNotesController(username?: string) {
  const today = useMemo(() => startOfDay(new Date()), []);
  const maxEntryDate = useMemo(() => addDays(today, 7), [today]);
  const [selectedDate, setSelectedDate] = useState(() => formatDate(today));
  const [visibleMonth, setVisibleMonth] = useState(() => startOfMonth(today));
  const [notes, setNotes] = useState<DailyWorkNote[]>(() => loadNotes(username));
  const [statusFilter, setStatusFilter] = useState<DailyWorkStatus>("completed");

  useEffect(() => {
    setNotes(loadNotes(username));
  }, [username]);

  const calendarDays = useMemo(
    () => buildCalendarDays(visibleMonth, selectedDate, notes, maxEntryDate),
    [maxEntryDate, notes, selectedDate, visibleMonth],
  );

  const selectedDateNotes = useMemo(
    () => notes.filter((note) => note.date === selectedDate).sort((left, right) => right.createdAt.localeCompare(left.createdAt)),
    [notes, selectedDate],
  );

  const filteredNotes = useMemo(
    () => selectedDateNotes.filter((note) => note.status === statusFilter),
    [selectedDateNotes, statusFilter],
  );

  const statusCounts = useMemo(() => {
    return statusOrder.reduce<Record<DailyWorkStatus, number>>(
      (counts, status) => ({
        ...counts,
        [status]: selectedDateNotes.filter((note) => note.status === status).length,
      }),
      { completed: 0, incomplete: 0, reserved: 0 },
    );
  }, [selectedDateNotes]);

  const selectDate = (date: string) => {
    setSelectedDate(date);
    const nextDate = parseDateValue(date);
    if (nextDate) {
      setVisibleMonth(startOfMonth(nextDate));
    }
  };

  const selectMonth = (value: string) => {
    const nextMonth = parseMonthValue(value);
    if (nextMonth) {
      setVisibleMonth(nextMonth);
    }
  };

  const previousMonth = () => {
    setVisibleMonth((current) => new Date(current.getFullYear(), current.getMonth() - 1, 1));
  };

  const nextMonth = () => {
    setVisibleMonth((current) => new Date(current.getFullYear(), current.getMonth() + 1, 1));
  };

  const addNote = (draft: DailyWorkNoteDraft) => {
    if (!draft.date) {
      return false;
    }

    const noteDate = startOfDay(draft.date);
    if (noteDate > maxEntryDate) {
      return false;
    }

    const trimmedContent = draft.content.trim();
    if (!trimmedContent) {
      return false;
    }

    const nextNote: DailyWorkNote = {
      id: makeNoteId(),
      content: trimmedContent,
      date: formatDate(noteDate),
      status: draft.status,
      createdAt: new Date().toISOString(),
    };

    setNotes((current) => {
      const nextNotes = [nextNote, ...current];
      saveNotes(username, nextNotes);
      return nextNotes;
    });
    selectDate(nextNote.date);
    setStatusFilter(nextNote.status);
    return true;
  };

  const updateNoteStatus = (noteId: string, status: DailyWorkStatus) => {
    setNotes((current) => {
      const nextNotes = current.map((note) => note.id === noteId ? { ...note, status } : note);
      saveNotes(username, nextNotes);
      return nextNotes;
    });
  };

  const removeNote = (noteId: string) => {
    setNotes((current) => {
      const nextNotes = current.filter((note) => note.id !== noteId);
      saveNotes(username, nextNotes);
      return nextNotes;
    });
  };

  return {
    addNote,
    calendarDays,
    filteredNotes,
    maxEntryDate,
    monthLabel: visibleMonth.toLocaleDateString("vi-VN", { month: "long", year: "numeric" }),
    monthValue: formatMonthValue(visibleMonth),
    nextMonth,
    previousMonth,
    removeNote,
    selectDate,
    selectMonth,
    selectedDate,
    selectedDateLabel: formatDisplayDate(selectedDate),
    statusCounts,
    statusFilter,
    setStatusFilter,
    today,
    totalSelectedDateNotes: selectedDateNotes.length,
    updateNoteStatus,
  };
}

function buildCalendarDays(month: Date, selectedDate: string, notes: DailyWorkNote[], maxEntryDate: Date): DailyWorkCalendarDay[] {
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
      taskCount: notes.filter((note) => note.date === dateValue).length,
    };
  });
}

function storageKey(username: string | undefined) {
  return `pjjyuji.daily-work-notes.${username || "guest"}`;
}

function loadNotes(username: string | undefined) {
  try {
    const saved = window.localStorage.getItem(storageKey(username));
    return saved ? JSON.parse(saved) as DailyWorkNote[] : [];
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
  return left.getFullYear() === right.getFullYear() && left.getMonth() === right.getMonth() && left.getDate() === right.getDate();
}

function formatDate(date: Date) {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

function formatMonthValue(date: Date) {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
}

function parseDateValue(value: string) {
  const [yearText, monthText, dayText] = value.split("-");
  const year = Number(yearText);
  const month = Number(monthText);
  const day = Number(dayText);
  if (!Number.isInteger(year) || !Number.isInteger(month) || !Number.isInteger(day)) {
    return null;
  }

  return new Date(year, month - 1, day);
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

function formatDisplayDate(value: string) {
  const date = parseDateValue(value);
  return date ? date.toLocaleDateString("vi-VN", { weekday: "long", day: "2-digit", month: "2-digit", year: "numeric" }) : value;
}
