import { safeInvoke } from "./_base";
import type { EvidenceResizeOptions, EvidenceResizeResult } from "@/_/types/evidence-resize";

export function listExcelSheetNames(inputPath: string) {
  return safeInvoke<string[]>("list_excel_sheet_names", { inputPath });
}

export function resizeEvidenceImages(
  inputPath: string,
  outputPath: string,
  widthCm: number | null,
  heightCm: number | null,
  options: EvidenceResizeOptions,
  selectedSheets: string[] | null,
) {
  return safeInvoke<EvidenceResizeResult>("resize_evidence_images", {
    inputPath,
    outputPath,
    widthCm,
    heightCm,
    options,
    selectedSheets,
  });
}
