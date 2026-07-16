import { safeInvoke } from "./_base";
import type { XlsxMarkdownResult } from "@/_/types/excel2md";

export function convertXlsxSpecToMarkdown(inputPath: string, outputPath: string | null) {
  return safeInvoke<XlsxMarkdownResult>("excel2md", { inputPath, outputPath });
}
