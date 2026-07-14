import { safeInvoke } from "./_base";
import type {
  CreateDailyNoteRequest,
  DailyNoteDateCountResult,
  DailyWorkNoteResult,
} from "@/_/types/daily-note";

export function createDailyNote(username: string, request: CreateDailyNoteRequest) {
  return safeInvoke<DailyWorkNoteResult>("create_daily_note", { username, request });
}

export function getDailyNotesByDate(username: string, noteDate: string) {
  return safeInvoke<DailyWorkNoteResult[]>("get_daily_notes_by_date", { username, noteDate });
}

export function getDailyNotesByMonth(username: string, year: number, month: number) {
  return safeInvoke<DailyWorkNoteResult[]>("get_daily_notes_by_month", { username, year, month });
}

export function getDailyNoteCounts(username: string, year: number, month: number) {
  return safeInvoke<DailyNoteDateCountResult[]>("get_daily_note_counts", { username, year, month });
}

export function updateDailyNoteContent(id: number, username: string, content: string) {
  return safeInvoke<DailyWorkNoteResult>("update_daily_note_content", { id, username, content });
}

export function updateDailyNoteStatus(id: number, username: string, status: string) {
  return safeInvoke<DailyWorkNoteResult>("update_daily_note_status", { id, username, status });
}

export function deleteDailyNote(id: number, username: string) {
  return safeInvoke<void>("delete_daily_note", { id, username });
}
