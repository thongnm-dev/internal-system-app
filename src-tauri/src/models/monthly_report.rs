//! Các kiểu dữ liệu cho module so sánh báo cáo tháng.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompareStatus {
    #[serde(rename = "match")]
    Match,
    #[serde(rename = "mismatch")]
    Mismatch,
    #[serde(rename = "csv-only")]
    CsvOnly,
    #[serde(rename = "schedule-only")]
    ScheduleOnly,
    #[serde(rename = "csv-only-warning")]
    CsvOnlyWarning,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScheduleDetail {
    pub job_id: String,
    pub job_name: String,
    pub sheet_name: String,
    pub hours: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CsvDetail {
    pub job_id: String,
    pub work_content: String,
    pub hours: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompareRow {
    pub date: String,
    pub phase: String,
    pub process_code: String,
    pub project_name: String,
    pub csv_hours: f64,
    pub schedule_hours: f64,
    pub diff_hours: f64,
    pub status: CompareStatus,
    pub csv_details: Vec<CsvDetail>,
    pub schedule_details: Vec<ScheduleDetail>,
}
