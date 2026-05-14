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
    pub fn total_minutes(&self) -> i64 {
        self.regular_minutes
            + self.normal_overtime_minutes
            + self.legal_holiday_overtime_minutes
            + self.legal_public_holiday_overtime_minutes
            + self.late_night_overtime_minutes
    }
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
