import type { CompareResult } from "@/_/types/file-compare";
import { safeInvoke } from "./_base";

/** So sánh 2 file (text/markdown/word/excel). 2 file phải cùng loại. */
export function fileCompareRun(fileA: string, fileB: string) {
  return safeInvoke<CompareResult>("file_compare_run", { fileA, fileB });
}

/** Xuất kết quả so sánh ra file Excel (.xlsx) tại `outputPath`. */
export function fileCompareExport(fileA: string, fileB: string, outputPath: string) {
  return safeInvoke<string>("file_compare_export", { fileA, fileB, outputPath });
}
