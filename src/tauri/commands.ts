import { invoke } from "@tauri-apps/api/core";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import type { SystemInfo } from "@/shared/types/system";
import type { ImportBatchSummary, ImportCsvPreviewResult, ImportCsvResult } from "@/shared/types/import-csv";
import type { XlsxMarkdownResult } from "@/shared/types/excel2md";

type TauriWindow = Window & {
  __TAURI_INTERNALS__?: { invoke?: unknown };
};

export function canUseTauriRuntime(): boolean {
  return typeof window !== "undefined" && typeof (window as TauriWindow).__TAURI_INTERNALS__?.invoke === "function";
}

export function friendlyError(error: unknown): string {
  const text = String(error);
  if (text.includes("__TAURI_INTERNALS__") || text.includes("reading 'invoke'")) {
    return tauriRuntimeMessage;
  }
  return text;
}

async function safeInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!canUseTauriRuntime()) {
    throw new Error(tauriRuntimeMessage);
  }
  return invoke<T>(command, args);
}

// --- System ---

export function getSystemInfo() {
  return safeInvoke<SystemInfo>("get_system_info");
}

export function checkInternetConnection() {
  return safeInvoke<boolean>("check_internet_connection");
}

// --- Import CSV ---

export function listImportBatches() {
  return safeInvoke<ImportBatchSummary[]>("list_import_batches");
}

export function previewMonthlyReportCsv(path: string) {
  return safeInvoke<ImportCsvPreviewResult>("preview_monthly_report_csv", { path });
}

export function importMonthlyReportCsv(path: string, reportName: string, note: string) {
  return safeInvoke<ImportCsvResult>("import_monthly_report_csv", { path, reportName, note });
}

// --- Xlsx → Markdown ---

export function convertXlsxSpecToMarkdown(inputPath: string, outputPath: string | null) {
  return safeInvoke<XlsxMarkdownResult>("excel2md", { inputPath, outputPath });
}

// --- Projects ---

export type ProjectMember = { username: string; name: string };

export type ProjectDetailResult = {
  id: number;
  code: string;
  name: string;
  client: string;
  backlog_key: string;
  backlog_url: string;
  backlog_space: string;
  is_active: boolean;
  members: ProjectMember[];
  created_at: string;
  updated_at: string;
};

export type BacklogProjectLookup = {
  project_id: string | number;
  project_key: string;
  project_name: string;
};

export type CreateProjectRequest = {
  code: string;
  name: string;
  client?: string;
  backlog_key?: string;
  backlog_url?: string;
  backlog_space?: string;
  members: ProjectMember[];
};

export type ProjectSummaryResult = {
  id: number;
  code: string;
  name: string;
  client: string;
  is_active: boolean;
  member_count: number;
  created_at: string;
};

export function createProject(request: CreateProjectRequest) {
  return safeInvoke<ProjectDetailResult>("create_project", { request });
}

export function updateProject(projectId: number, request: CreateProjectRequest) {
  return safeInvoke<ProjectDetailResult>("update_project", { projectId, request });
}

export function getProjectDetail(projectId: number) {
  return safeInvoke<ProjectDetailResult>("get_project_detail", { projectId });
}

export function listProjects() {
  return safeInvoke<ProjectSummaryResult[]>("list_projects");
}

export function deleteProject(projectId: number) {
  return safeInvoke<void>("delete_project", { projectId });
}

export function getBacklogProjectByKey(projectKey: string) {
  return safeInvoke<BacklogProjectLookup>("get_backlog_project_by_key", { projectKey });
}

// --- Daily Work Notes ---

export type DailyWorkNoteResult = {
  id: number;
  username: string;
  content: string;
  note_date: string;
  status: string;
  created_at: string;
};

export type CreateDailyNoteRequest = {
  content: string;
  note_date: string;
  status: string;
};

export type DailyNoteDateCountResult = {
  note_date: string;
  note_count: number;
};

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

export function updateDailyNoteStatus(id: number, username: string, status: string) {
  return safeInvoke<DailyWorkNoteResult>("update_daily_note_status", { id, username, status });
}

export function deleteDailyNote(id: number, username: string) {
  return safeInvoke<void>("delete_daily_note", { id, username });
}
