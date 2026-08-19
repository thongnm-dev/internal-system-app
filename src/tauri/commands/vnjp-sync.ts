import type {
  AnalyzeAndApplyResult,
  CleanupResult,
  ConfirmedInsert,
  RedCell,
  RedCellVerificationReport,
  RowAlignmentReport,
  RowInsertResult,
  SyncAnalysis,
} from "@/_/types/vnjp-sync";
import { safeInvoke } from "./_base";

/** Gộp "Phân tích" + "Áp dụng" thành 1 lệnh gọi backend duy nhất — xem `vnjp_sync_analyze_and_apply`. */
export function vnjpSyncAnalyzeAndApply(
  vnPath: string,
  jpPath: string,
): Promise<AnalyzeAndApplyResult> {
  return safeInvoke<AnalyzeAndApplyResult>("vnjp_sync_analyze_and_apply", {
    vnPath,
    jpPath,
  });
}

/** Đường dẫn thư mục Temp — nơi lưu file kết quả của "Áp dụng" — để liệt kê danh sách file đã tạo. */
export function vnjpSyncTempDir(): Promise<string> {
  return safeInvoke<string>("vnjp_sync_temp_dir");
}

export function vnjpSyncExportReport(
  analysis: SyncAnalysis,
  outputPath: string,
): Promise<string> {
  return safeInvoke<string>("vnjp_sync_export_report", { analysis, outputPath });
}

export function vnjpSyncCleanup(
  jpPath: string,
  outputPath: string,
): Promise<CleanupResult> {
  return safeInvoke<CleanupResult>("vnjp_sync_cleanup", { jpPath, outputPath });
}

export function vnjpSyncAnalyzeRowAlignment(
  vnPath: string,
  jpPath: string,
): Promise<RowAlignmentReport> {
  return safeInvoke<RowAlignmentReport>("vnjp_sync_analyze_row_alignment", {
    vnPath,
    jpPath,
  });
}

export function vnjpSyncInsertRows(
  jpPath: string,
  vnPath: string,
  outputPath: string,
  inserts: ConfirmedInsert[],
): Promise<RowInsertResult> {
  return safeInvoke<RowInsertResult>("vnjp_sync_insert_rows", {
    jpPath,
    vnPath,
    outputPath,
    inserts,
  });
}

export function vnjpSyncVerifyRedCellsAi(
  jpPath: string,
  redCells: RedCell[],
  provider: string,
  model: string,
): Promise<RedCellVerificationReport> {
  return safeInvoke<RedCellVerificationReport>("vnjp_sync_verify_red_cells_ai", {
    jpPath,
    redCells,
    provider,
    model,
  });
}
