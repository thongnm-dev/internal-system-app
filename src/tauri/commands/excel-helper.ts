import { safeInvoke } from "./_base";
import type { ExcelHelperOptions, ExcelHelperResult } from "@/_/types/excel-helper";

export function listExcelSheetNames(inputPath: string) {
  return safeInvoke<string[]>("list_excel_sheet_names", { inputPath });
}

export function resizeExcelImages(
  inputPath: string,
  outputPath: string,
  widthCm: number | null,
  heightCm: number | null,
  options: ExcelHelperOptions,
  selectedSheets: string[] | null,
) {
  return safeInvoke<ExcelHelperResult>("resize_excel_images", {
    inputPath,
    outputPath,
    widthCm,
    heightCm,
    options,
    selectedSheets,
  });
}
