//! Service preview và so sánh dữ liệu CSV báo cáo tháng.
//!
//! Cung cấp chức năng preview (xem trước) file CSV trước khi import
//! và so sánh dữ liệu CSV với lịch làm việc.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::services::csv_reader_service;
use crate::models::import_csv::{ImportCsvPreviewResult, ImportPreviewRow, WorkRecord};
use crate::models::monthly_report::{CompareRow, CompareStatus, CsvDetail, ScheduleDetail};
use crate::services::schedule_service;
use crate::utils::app_config;
use ini::Ini;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

fn load_config() -> AppResult<(String, String, String)> {
    let path = app_config::config_path();
    let ini = Ini::load_from_file(&path)
        .map_err(|e| AppError::new(format!("Cannot read config.ini: {e}")))?;
    let section = ini.section(Some("INTERNAL"))
        .ok_or_else(|| AppError::new("Missing [INTERNAL] section in config.ini"))?;
    let base_url = section.get("BASE_URL")
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::new("Missing BASE_URL in [INTERNAL] config"))?
        .to_string();
    let username = section.get("USERNAME")
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::new("Missing USERNAME in [INTERNAL] config"))?
        .to_string();
    let password = section.get("PASSWORD")
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::new("Missing PASSWORD in [INTERNAL] config"))?
        .to_string();
    Ok((base_url, username, password))
}

pub async fn fetch_csv_from_url(
    date_from: &str,
    date_to: &str,
    staff: &str,
) -> AppResult<String> {
    let (base_url, username, password) = load_config()?;

    let jar = Arc::new(reqwest::cookie::Jar::default());
    let client = reqwest::Client::builder()
        .cookie_provider(jar)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| AppError::new(format!("Failed to build HTTP client: {e}")))?;

    let login_url = format!("{base_url}/login");
    let login_resp = client
        .post(&login_url)
        .form(&[("txtUserName", username.as_str()), ("txtPass", password.as_str())])
        .send()
        .await
        .map_err(|e| AppError::new(format!("Login failed: {e}")))?;

    if !login_resp.status().is_success() && !login_resp.status().is_redirection() {
        return Err(AppError::new(format!(
            "Login returned status {}",
            login_resp.status()
        )));
    }

    let csv_url = format!("{base_url}/pjjyujidatacsv");
    let params: Vec<(&str, &str)> = vec![
        ("txtExtractionPeriodFrom", date_from),
        ("txtExtractionPeriodTo", date_to),
        ("cmbOutputCsvMethod", "1"),
        ("handle", "exportCSV"),
        ("projectCd", ""),
        ("shm001", staff),
    ];
    let response = client
        .post(&csv_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::new(format!("Failed to fetch CSV: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::new(format!(
            "Server returned status {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::new(format!("Failed to read response: {e}")))?;

    if bytes.is_empty() {
        return Err(AppError::new("Server returned empty response"));
    }

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("pjjyuji_auto_fetch.csv");
    std::fs::write(&temp_path, &bytes)?;

    Ok(temp_path.to_string_lossy().to_string())
}

/// Preview nội dung file CSV: parse dữ liệu và trả về kết quả tóm tắt.
/// Frontend sử dụng kết quả này để hiển thị bảng xác nhận trước khi import.
pub fn preview_csv(path: &str) -> AppResult<ImportCsvPreviewResult> {
    let data = csv_reader_service::read_csv_import_data(path)?;

    Ok(ImportCsvPreviewResult {
        source_path: data.source_path,
        source_file_name: data.source_file_name,
        row_count: data.records.len(),
        total_minutes: data.total_minutes,
        preview_rows: build_preview_rows(&data.records),
        raw_headers: data.raw_csv.headers,
        raw_rows: data.raw_csv.rows,
        minute_column_indexes: data.raw_csv.minute_column_indexes,
    })
}

/// Chuyển danh sách `WorkRecord` thành danh sách `ImportPreviewRow`.
/// Tính tổng phút cho mỗi dòng thay vì giữ chi tiết từng loại giờ.
fn build_preview_rows(records: &[WorkRecord]) -> Vec<ImportPreviewRow> {
    records
        .iter()
        .map(|record| ImportPreviewRow {
            date: record.date.clone(),
            project_code: record.project_code.clone(),
            project_name: record.project_name.clone(),
            process_code: record.process_code.clone(),
            phase_name: record.phase_name.clone(),
            work_content: record.work_content.clone(),
            total_minutes: record.totals.total_minutes(),
        })
        .collect()
}

fn normalize_date(value: &str) -> String {
    let parts: Vec<&str> = value.split(|c: char| !c.is_ascii_digit()).filter(|s| !s.is_empty()).collect();
    if parts.len() >= 3 {
        format!("{}-{:0>2}-{:0>2}", parts[0], parts[1], parts[2])
    } else {
        value.trim().to_string()
    }
}

fn map_process_code_to_phase(code: &str) -> Option<&'static str> {
    match code.trim() {
        "43" => Some("Bug"),
        "45" => Some("Thay đổi quy cách"),
        _ => None,
    }
}

fn normalize_schedule_phase(phase: &str) -> &str {
    if phase.contains("Bug") || phase.contains("BUG") {
        "Bug"
    } else if phase.contains("Thay đổi quy cách") {
        "Thay đổi quy cách"
    } else {
        phase
    }
}

fn extract_job_id(work_content: &str) -> String {
    if let Some(start) = work_content.find("[No_") {
        let after = &work_content[start + 1..];
        if let Some(end) = after.find(']') {
            return after[..end].to_string();
        }
    }
    String::new()
}

#[derive(Default)]
struct CsvPhaseGroup {
    total_hours: f64,
    process_codes: BTreeSet<String>,
    project_names: BTreeSet<String>,
    entries: Vec<CsvDetail>,
}

#[derive(Default)]
struct SchedulePhaseGroup {
    total_hours: f64,
    details: Vec<ScheduleDetail>,
}

pub fn compare_csv_with_schedule(
    csv_path: &str,
    schedule_path: &str,
    target_year: i32,
    target_month: u32,
    user_filter: &str,
) -> AppResult<Vec<CompareRow>> {
    let csv_data = csv_reader_service::read_csv_import_data(csv_path)?;
    let preview_rows = build_preview_rows(&csv_data.records);

    let schedule = schedule_service::read_schedule(schedule_path, target_year, target_month, user_filter)?;

    let mut csv_mapped: HashMap<(String, String), CsvPhaseGroup> = HashMap::new();
    let mut csv_unmapped: HashMap<(String, String), CsvPhaseGroup> = HashMap::new();

    for row in &preview_rows {
        let date = normalize_date(&row.date);
        let hours = row.total_minutes as f64 / 60.0;

        let job_id = extract_job_id(&row.work_content);
        let detail = CsvDetail {
            job_id,
            work_content: row.work_content.clone(),
            hours,
        };

        if let Some(phase) = map_process_code_to_phase(&row.process_code) {
            let key = (date, phase.to_string());
            let group = csv_mapped.entry(key).or_default();
            group.total_hours += hours;
            group.process_codes.insert(row.process_code.clone());
            group.project_names.insert(row.project_name.clone());
            group.entries.push(detail);
        } else {
            let key = (date, row.phase_name.clone());
            let group = csv_unmapped.entry(key).or_default();
            group.total_hours += hours;
            group.process_codes.insert(row.process_code.clone());
            group.project_names.insert(row.project_name.clone());
            group.entries.push(detail);
        }
    }

    let mm = format!("{target_month:02}");
    let yyyy = format!("{target_year}");
    let mut schedule_grouped: HashMap<(String, String), SchedulePhaseGroup> = HashMap::new();

    for worker in &schedule.workers {
        for day in &worker.days {
            let date = normalize_date(&format!("{yyyy}/{mm}/{}", day.day));
            for entry in &day.entries {
                let phase = normalize_schedule_phase(&entry.phase).to_string();
                let key = (date.clone(), phase);
                let group = schedule_grouped.entry(key).or_default();
                group.total_hours += entry.hours;
                group.details.push(ScheduleDetail {
                    job_id: entry.job_id.clone(),
                    job_name: entry.job_name.clone(),
                    sheet_name: entry.sheet_name.clone(),
                    hours: entry.hours,
                });
            }
        }
    }

    let mut all_keys: BTreeSet<(String, String)> = BTreeSet::new();
    all_keys.extend(csv_mapped.keys().cloned());
    all_keys.extend(schedule_grouped.keys().cloned());

    let mut rows: Vec<CompareRow> = Vec::new();

    for (date, phase) in all_keys {
        let csv = csv_mapped.get(&(date.clone(), phase.clone()));
        let sched = schedule_grouped.get(&(date.clone(), phase.clone()));

        let csv_hours = csv.map(|g| g.total_hours).unwrap_or(0.0);
        let schedule_hours = sched.map(|g| g.total_hours).unwrap_or(0.0);
        let diff_hours = csv_hours - schedule_hours;

        let status = match (csv.is_some(), sched.is_some()) {
            (true, false) => CompareStatus::CsvOnly,
            (false, true) => CompareStatus::ScheduleOnly,
            _ if diff_hours.abs() < 0.01 => CompareStatus::Match,
            _ => CompareStatus::Mismatch,
        };

        rows.push(CompareRow {
            date,
            phase,
            process_code: csv.map(|g| g.process_codes.iter().cloned().collect::<Vec<_>>().join(", ")).unwrap_or_default(),
            project_name: csv.map(|g| g.project_names.iter().cloned().collect::<Vec<_>>().join(", ")).unwrap_or_default(),
            csv_hours,
            schedule_hours,
            diff_hours,
            status,
            csv_details: csv.map(|g| g.entries.clone()).unwrap_or_default(),
            schedule_details: sched.map(|g| g.details.clone()).unwrap_or_default(),
        });
    }

    for ((date, phase), group) in csv_unmapped {
        rows.push(CompareRow {
            date,
            phase,
            process_code: group.process_codes.into_iter().collect::<Vec<_>>().join(", "),
            project_name: group.project_names.into_iter().collect::<Vec<_>>().join(", "),
            csv_hours: group.total_hours,
            schedule_hours: 0.0,
            diff_hours: group.total_hours,
            status: CompareStatus::CsvOnlyWarning,
            csv_details: group.entries,
            schedule_details: Vec::new(),
        });
    }

    rows.sort_by(|a, b| a.date.cmp(&b.date).then(a.phase.cmp(&b.phase)));

    Ok(rows)
}
