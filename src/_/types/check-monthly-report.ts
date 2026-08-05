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

export type CompareStatus = "match" | "mismatch" | "csv-only" | "schedule-only" | "csv-only-warning";

export type CsvDetail = {
  job_id: string;
  work_content: string;
  hours: number;
};

export type ScheduleDetail = {
  job_id: string;
  job_name: string;
  sheet_name: string;
  hours: number;
};

export type CompareRow = {
  date: string;
  phase: string;
  process_code: string;
  project_name: string;
  csv_hours: number;
  schedule_hours: number;
  diff_hours: number;
  status: CompareStatus;
  csv_details: CsvDetail[];
  schedule_details: ScheduleDetail[];
};

export type MinuteTotals = {
  regular_minutes: number;
  normal_overtime_minutes: number;
  legal_holiday_overtime_minutes: number;
  legal_public_holiday_overtime_minutes: number;
  late_night_overtime_minutes: number;
};

export type ReportWorkDetail = {
  date: string;
  work_content: string;
  total_minutes: number;
};

export type ReportPhaseSummary = {
  process_code: string;
  phase_name: string;
  totals: MinuteTotals;
  row_count: number;
  details: ReportWorkDetail[];
};

export type ReportProjectSummary = {
  project_code: string;
  project_name: string;
  totals: MinuteTotals;
  row_count: number;
  phases: ReportPhaseSummary[];
};

/** Tổng hợp project/phase/work-detail được build ở frontend từ dữ liệu CSV import preview. */
export type MonthlyReportSummary = {
  source_path: string;
  row_count: number;
  grand_total: MinuteTotals;
  projects: ReportProjectSummary[];
};

export type SelectedReportPhaseDetail = {
  project_code: string;
  project_name: string;
  phase: ReportPhaseSummary;
};
