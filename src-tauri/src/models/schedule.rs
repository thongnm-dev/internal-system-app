use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkInfoEntry {
    pub phase: String,
    pub job_id: String,
    pub job_name: String,
    pub hours: f64,
    pub sheet_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayWork {
    pub day: String,
    pub entries: Vec<WorkInfoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerSchedule {
    pub worker_name: String,
    pub days: Vec<DayWork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResult {
    pub file_path: String,
    pub target_month: String,
    pub workers: Vec<WorkerSchedule>,
}
