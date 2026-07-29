import { safeInvoke } from "./_base";
import type { EvidenceResizeOptions, EvidenceResizeResult } from "@/_/types/evidence-resize";

export function resizeEvidenceImages(
  inputPath: string,
  outputPath: string,
  widthCm: number | null,
  heightCm: number | null,
  options: EvidenceResizeOptions,
) {
  return safeInvoke<EvidenceResizeResult>("resize_evidence_images", {
    inputPath,
    outputPath,
    widthCm,
    heightCm,
    options,
  });
}
