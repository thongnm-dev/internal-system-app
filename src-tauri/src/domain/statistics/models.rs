use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct MinuteTotals {
    pub regular_minutes: i64,
    pub normal_overtime_minutes: i64,
    pub legal_holiday_overtime_minutes: i64,
    pub legal_public_holiday_overtime_minutes: i64,
    pub late_night_overtime_minutes: i64,
}

impl MinuteTotals {
    pub fn add(&mut self, other: &MinuteTotals) {
        self.regular_minutes += other.regular_minutes;
        self.normal_overtime_minutes += other.normal_overtime_minutes;
        self.legal_holiday_overtime_minutes += other.legal_holiday_overtime_minutes;
        self.legal_public_holiday_overtime_minutes += other.legal_public_holiday_overtime_minutes;
        self.late_night_overtime_minutes += other.late_night_overtime_minutes;
    }

    pub fn total_minutes(&self) -> i64 {
        self.regular_minutes
            + self.normal_overtime_minutes
            + self.legal_holiday_overtime_minutes
            + self.legal_public_holiday_overtime_minutes
            + self.late_night_overtime_minutes
    }
}

#[derive(Clone, Serialize)]
pub struct WorkDetail {
    pub date: String,
    pub work_content: String,
    pub total_minutes: i64,
}

#[derive(Clone, Serialize)]
pub struct PhaseSummary {
    pub process_code: String,
    pub phase_name: String,
    pub totals: MinuteTotals,
    pub row_count: usize,
    pub details: Vec<WorkDetail>,
}

#[derive(Clone, Serialize)]
pub struct ProjectSummary {
    pub project_code: String,
    pub project_name: String,
    pub totals: MinuteTotals,
    pub row_count: usize,
    pub phases: Vec<PhaseSummary>,
}

#[derive(Serialize)]
pub struct AnalysisResult {
    pub source_path: String,
    pub row_count: usize,
    pub grand_total: MinuteTotals,
    pub projects: Vec<ProjectSummary>,
}

#[derive(Clone)]
pub struct WorkRecord {
    pub date: String,
    pub project_code: String,
    pub project_name: String,
    pub process_code: String,
    pub process_name: String,
    pub phase_name: String,
    pub work_content: String,
    pub totals: MinuteTotals,
}

#[derive(Default)]
pub struct PhaseAccumulator {
    pub process_code: String,
    pub phase_name: String,
    pub totals: MinuteTotals,
    pub row_count: usize,
    pub details: Vec<WorkDetail>,
}

#[derive(Default)]
pub struct ProjectAccumulator {
    pub project_code: String,
    pub project_name: String,
    pub totals: MinuteTotals,
    pub row_count: usize,
    pub phases: std::collections::BTreeMap<String, PhaseAccumulator>,
}
