import { safeInvoke } from "./_base";
import type { CompareRow, ImportCsvPreviewResult, ImportCsvResult } from "@/_/types/check-monthly-report";

export function previewMonthlyReportCsv(path: string) {
  return safeInvoke<ImportCsvPreviewResult>("preview_monthly_report_csv", { path });
}

export function importMonthlyReportCsv(path: string, reportName: string, note: string) {
  return safeInvoke<ImportCsvResult>("import_monthly_report_csv", { path, reportName, note });
}

export function compareMonthlyReport(csvPath: string, schedulePath: string, targetYear: number, targetMonth: number, userFilter?: string) {
  return safeInvoke<CompareRow[]>("compare_monthly_report", { csvPath, schedulePath, targetYear, targetMonth, userFilter });
}

export function fetchCsvFromSystem(dateFrom: string, dateTo: string, staff: string) {
  return safeInvoke<string>("fetch_csv_from_system", { dateFrom, dateTo, staff });
}
