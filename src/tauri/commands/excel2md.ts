import { safeInvoke } from "./_base";
import type { XlsxMarkdownResult } from "@/shared/types/excel2md";

export function convertXlsxSpecToMarkdown(inputPath: string, outputPath: string | null) {
  return safeInvoke<XlsxMarkdownResult>("excel2md", { inputPath, outputPath });
}
