import { safeInvoke } from "./_base";
import type {
  CreateDailyReportTaskRequest,
  DailyReportEntryResult,
  DailyReportUserTaskResult,
  SaveDailyReportEntryRequest,
} from "@/_/types/daily-report";

export function saveDailyReportEntry(username: string, request: SaveDailyReportEntryRequest) {
  return safeInvoke<DailyReportEntryResult>("save_daily_report_entry", { username, request });
}

export function clearDailyReportEntry(username: string, taskId: string, entryDate: string) {
  return safeInvoke<void>("clear_daily_report_entry", { username, taskId, entryDate });
}

export function getDailyReportEntries(username: string, year: number, month: number) {
  return safeInvoke<DailyReportEntryResult[]>("get_daily_report_entries", { username, year, month });
}

export function createDailyReportTask(username: string, request: CreateDailyReportTaskRequest) {
  return safeInvoke<DailyReportUserTaskResult>("create_daily_report_task", { username, request });
}

export function getDailyReportTasks(username: string) {
  return safeInvoke<DailyReportUserTaskResult[]>("get_daily_report_tasks", { username });
}

export function deleteDailyReportTask(username: string, taskId: string) {
  return safeInvoke<void>("delete_daily_report_task", { username, taskId });
}
