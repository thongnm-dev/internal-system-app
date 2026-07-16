use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SyncDailyReportRequest {
    pub date: String,
    pub entries: Vec<SyncEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SyncEntry {
    pub project_code: String,
    pub project_name: String,
    pub task_name: String,
    pub process_code: String,
    pub category_label: String,
    pub hour: f64,
    pub comment: String,
    pub regular_ot: f64,
    pub midnight_ot: f64,
}

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub synced_count: usize,
    pub total_count: usize,
}
