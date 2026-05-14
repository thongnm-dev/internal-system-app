export type MinuteTotals = {
  regular_minutes: number;
  normal_overtime_minutes: number;
  legal_holiday_overtime_minutes: number;
  legal_public_holiday_overtime_minutes: number;
  late_night_overtime_minutes: number;
};

export type PhaseSummary = {
  process_code: string;
  phase_name: string;
  totals: MinuteTotals;
  row_count: number;
  details: WorkDetail[];
};

export type WorkDetail = {
  date: string;
  work_content: string;
  total_minutes: number;
};

export type ProjectSummary = {
  project_code: string;
  project_name: string;
  totals: MinuteTotals;
  row_count: number;
  phases: PhaseSummary[];
};

export type AnalysisResult = {
  source_path: string;
  row_count: number;
  grand_total: MinuteTotals;
  projects: ProjectSummary[];
};

export type SystemInfo = {
  username: string;
  timestamp: string;
  ip_address: string;
  version: string;
};

export type MessageMode = "info" | "error";

export type MenuKey = "overview" | "projects" | "phases" | "importCsv" | "settings";

export type SummaryMetric = {
  label: string;
  value: string;
};

export type SelectedPhaseDetail = {
  project_code: string;
  project_name: string;
  phase: PhaseSummary;
};

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
  row_count: number;
  total_minutes: number;
};
