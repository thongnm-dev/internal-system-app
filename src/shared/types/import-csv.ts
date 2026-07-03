export type ImportPreviewRow = {
  date: string;
  project_code: string;
  project_name: string;
  process_code: string;
  phase_name: string;
  work_content: string;
  total_minutes: number;
};

export type ImportCsvResult = {
  batch_id: number;
  source_path: string;
  source_file_name: string;
  imported_at: string;
  row_count: number;
  total_minutes: number;
  preview_rows: ImportPreviewRow[];
  raw_headers: string[];
  raw_rows: string[][];
  minute_column_indexes: number[];
};

export type ImportCsvPreviewResult = {
  source_path: string;
  source_file_name: string;
  row_count: number;
  total_minutes: number;
  preview_rows: ImportPreviewRow[];
  raw_headers: string[];
  raw_rows: string[][];
  minute_column_indexes: number[];
};

export type ImportBatchSummary = {
  id: number;
  source_file_name: string;
  imported_at: string;
  report_name: string;
  note: string;
  target_month_from: string;
  target_month_to: string;
  imported_by: string;
  row_count: number;
  total_minutes: number;
};
