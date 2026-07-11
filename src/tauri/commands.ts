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
  project_id: string | number;
  project_code: string;
  project_name: string;
  backlog_project_id?: string | number;
  backlog_project_key?: string;
  backlog_project_name?: string;
  members: ProjectMember[];
};

export type BacklogProjectLookup = {
  project_id: string | number;
  project_key: string;
  project_name: string;
};

export function getProjectDetail(projectId: string) {
  return safeInvoke<ProjectDetailResult>("get_project_detail", { projectId });
}

export function getBacklogProjectByKey(projectKey: string) {
  return safeInvoke<BacklogProjectLookup>("get_backlog_project_by_key", { projectKey });
}
