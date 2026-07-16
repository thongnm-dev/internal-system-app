export type MinuteTotals = {
  regular_minutes: number;
  normal_overtime_minutes: number;
  legal_holiday_overtime_minutes: number;
  legal_public_holiday_overtime_minutes: number;
  late_night_overtime_minutes: number;
};

export type WorkDetail = {
  date: string;
  work_content: string;
  total_minutes: number;
};

export type PhaseSummary = {
  process_code: string;
  phase_name: string;
  totals: MinuteTotals;
  row_count: number;
  details: WorkDetail[];
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

export type SelectedPhaseDetail = {
  project_code: string;
  project_name: string;
  phase: PhaseSummary;
};
