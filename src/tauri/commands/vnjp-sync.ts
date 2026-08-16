import type { ApplyResult, SyncAnalysis } from "@/_/types/vnjp-sync";
import { safeInvoke } from "./_base";

export function vnjpSyncAnalyze(
  vnPath: string,
  jpPath: string,
): Promise<SyncAnalysis> {
  return safeInvoke<SyncAnalysis>("vnjp_sync_analyze", { vnPath, jpPath });
}

export function vnjpSyncExportReport(
  analysis: SyncAnalysis,
  outputPath: string,
): Promise<string> {
  return safeInvoke<string>("vnjp_sync_export_report", { analysis, outputPath });
}

export function vnjpSyncApply(
  vnPath: string,
  jpPath: string,
  outputPath: string,
): Promise<ApplyResult> {
  return safeInvoke<ApplyResult>("vnjp_sync_apply", { vnPath, jpPath, outputPath });
}
