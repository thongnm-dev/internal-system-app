export type ExcelHelperOptions = {
  pageBreakPreview: boolean;
  zoomPercent: number | null;
  startColumn: string | null;
  fontName: string | null;
  fontSize: number | null;
  resetActiveCell: boolean;
  avoidCoveringContent: boolean;
};

export type ExcelHelperResult = {
  source_path: string;
  output_path: string;
  source_file_name: string;
  output_file_name: string;
  images_resized: number;
  drawings_processed: number;
  warnings: string[];
};
