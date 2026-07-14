import { safeInvoke } from "./_base";
import type {
  CreateDailyReportTaskRequest,
  DailyReportEntryResult,
  DailyReportPhaseResult,
  DailyReportProjectResult,
  DailyReportTaskHoursResult,
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

export function getDailyReportTasks(username: string, year: number, month: number) {
  return safeInvoke<DailyReportUserTaskResult[]>("get_daily_report_tasks", { username, year, month });
}

export function getDailyReportTaskHours(username: string) {
  return safeInvoke<DailyReportTaskHoursResult[]>("get_daily_report_task_hours", { username });
}

export function setDailyReportTaskCompleted(username: string, taskId: string, isCompleted: boolean) {
  return safeInvoke<DailyReportUserTaskResult>("set_daily_report_task_completed", {
    username,
    taskId,
    isCompleted,
  });
}

export function setProjectTaskCompleted(taskId: string, isCompleted: boolean) {
  return safeInvoke<boolean>("set_project_task_completed", { taskId, isCompleted });
}

export function getDailyReportPhases() {
  return safeInvoke<DailyReportPhaseResult[]>("get_daily_report_phases", {});
}

export function deleteDailyReportTask(username: string, taskId: string) {
  return safeInvoke<void>("delete_daily_report_task", { username, taskId });
}

export function getDailyReportProjects(username: string) {
  return safeInvoke<DailyReportProjectResult[]>("get_daily_report_projects", { username });
}
