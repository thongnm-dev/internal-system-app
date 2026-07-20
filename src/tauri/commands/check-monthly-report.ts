import { safeInvoke } from "./_base";
import type { ImportCsvPreviewResult, ImportCsvResult } from "@/_/types/check-monthly-report";

export function previewMonthlyReportCsv(path: string) {
  return safeInvoke<ImportCsvPreviewResult>("preview_monthly_report_csv", { path });
}

export function importMonthlyReportCsv(path: string, reportName: string, note: string) {
  return safeInvoke<ImportCsvResult>("import_monthly_report_csv", { path, reportName, note });
}
