import { safeInvoke } from "./_base";
import type { ImportBatchSummary, ImportCsvPreviewResult, ImportCsvResult } from "@/_/types/import-csv";

export function listImportBatches() {
  return safeInvoke<ImportBatchSummary[]>("list_import_batches");
}

export function previewMonthlyReportCsv(path: string) {
  return safeInvoke<ImportCsvPreviewResult>("preview_monthly_report_csv", { path });
}

export function importMonthlyReportCsv(path: string, reportName: string, note: string) {
  return safeInvoke<ImportCsvResult>("import_monthly_report_csv", { path, reportName, note });
}
