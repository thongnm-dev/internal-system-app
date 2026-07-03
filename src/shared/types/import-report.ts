import type { ImportPreviewRow } from "./import-csv";

export type ImportReportSearchCriteria = {
  target_month_from: string;
  target_month_to: string;
  report_name: string;
  keyword: string;
};

export type ImportReportListItem = {
  id: number;
  report_name: string;
  note: string;
  source_file_name: string;
  imported_at: string;
  imported_by: string;
  target_month_from: string;
  target_month_to: string;
  row_count: number;
  total_minutes: number;
};

export type ImportReportDetail = ImportReportListItem & {
  source_path: string;
  preview_rows: ImportPreviewRow[];
};
