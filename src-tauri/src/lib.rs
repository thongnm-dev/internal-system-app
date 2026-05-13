use chrono::Local;
use encoding_rs::SHIFT_JIS;
use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::net::UdpSocket;
use std::path::{Path, PathBuf};

const DATE: &str = "日付";
const PROJECT_CODE: &str = "プロジェクトコード";
const PROJECT_NAME: &str = "プロジェクト名";
const PROCESS_CODE: &str = "プロセスコード";
const PROCESS_NAME: &str = "プロセス";
const WORK_CONTENT: &str = "作業内容";
const REGULAR_MINUTES: &str = "時間内(分)";
const NORMAL_OVERTIME: &str = "普通残業時間(分)";
const LEGAL_HOLIDAY_OVERTIME: &str = "法定休日残業(分)";
const LEGAL_PUBLIC_HOLIDAY_OVERTIME: &str = "法定祝日残業時間(分)";
const LATE_NIGHT_OVERTIME: &str = "深夜残業(分)";

#[derive(Default, Clone, Serialize)]
struct MinuteTotals {
    regular_minutes: i64,
    normal_overtime_minutes: i64,
    legal_holiday_overtime_minutes: i64,
    legal_public_holiday_overtime_minutes: i64,
    late_night_overtime_minutes: i64,
}

impl MinuteTotals {
    fn add(&mut self, other: &MinuteTotals) {
        self.regular_minutes += other.regular_minutes;
        self.normal_overtime_minutes += other.normal_overtime_minutes;
        self.legal_holiday_overtime_minutes += other.legal_holiday_overtime_minutes;
        self.legal_public_holiday_overtime_minutes += other.legal_public_holiday_overtime_minutes;
        self.late_night_overtime_minutes += other.late_night_overtime_minutes;
    }

    fn total_minutes(&self) -> i64 {
        self.regular_minutes
            + self.normal_overtime_minutes
            + self.legal_holiday_overtime_minutes
            + self.legal_public_holiday_overtime_minutes
            + self.late_night_overtime_minutes
    }
}

#[derive(Clone, Serialize)]
struct WorkDetail {
    date: String,
    work_content: String,
    total_minutes: i64,
}

#[derive(Clone, Serialize)]
struct PhaseSummary {
    process_code: String,
    phase_name: String,
    totals: MinuteTotals,
    row_count: usize,
    details: Vec<WorkDetail>,
}

#[derive(Clone, Serialize)]
struct ProjectSummary {
    project_code: String,
    project_name: String,
    totals: MinuteTotals,
    row_count: usize,
    phases: Vec<PhaseSummary>,
}

#[derive(Serialize)]
struct AnalysisResult {
    source_path: String,
    row_count: usize,
    grand_total: MinuteTotals,
    projects: Vec<ProjectSummary>,
}

#[derive(Serialize)]
struct SystemInfo {
    username: String,
    timestamp: String,
    ip_address: String,
    version: String,
}

#[derive(Default)]
struct PhaseAccumulator {
    process_code: String,
    phase_name: String,
    totals: MinuteTotals,
    row_count: usize,
    details: Vec<WorkDetail>,
}

#[derive(Default)]
struct ProjectAccumulator {
    project_code: String,
    project_name: String,
    totals: MinuteTotals,
    row_count: usize,
    phases: BTreeMap<String, PhaseAccumulator>,
}

fn phase_label(code: &str, fallback: &str) -> String {
    match code.trim() {
        "10" => "PG".to_string(),
        "11" => "UT".to_string(),
        "48" => "Review PG".to_string(),
        "49" => "Review UT".to_string(),
        "43" => "Bug".to_string(),
        "45" => "Thay doi qui cach".to_string(),
        "59" => "Trong tay".to_string(),
        "24" => "Delivery".to_string(),
        other if fallback.trim().is_empty() => format!("Other ({other})"),
        _ => fallback.to_string(),
    }
}

fn parse_minutes(value: Option<&str>) -> i64 {
    value
        .unwrap_or_default()
        .trim()
        .replace(',', "")
        .parse::<i64>()
        .unwrap_or(0)
}

fn get_required_index(headers: &csv::StringRecord, name: &str) -> Result<usize, String> {
    headers
        .iter()
        .position(|header| header == name)
        .ok_or_else(|| format!("Required column was not found: {name}"))
}

fn resolve_input_path(path: &str) -> Result<PathBuf, String> {
    let raw = path.trim();
    let candidate = if raw.is_empty() {
        PathBuf::from("pjjyuji_data_csv_20260513.csv")
    } else {
        PathBuf::from(raw)
    };

    if candidate.is_absolute() && candidate.exists() {
        return Ok(candidate);
    }

    let current = env::current_dir().map_err(|err| err.to_string())?;
    let possible_paths = [
        current.join(&candidate),
        current.join("..").join(&candidate),
        current.join("..").join("..").join(&candidate),
    ];

    possible_paths
        .into_iter()
        .find(|path| path.exists())
        .ok_or_else(|| format!("CSV file was not found: {}", candidate.display()))
}

#[tauri::command]
fn analyze_csv(path: String) -> Result<AnalysisResult, String> {
    let input_path = resolve_input_path(&path)?;
    let bytes = fs::read(&input_path).map_err(|err| err.to_string())?;
    let (content, _, _) = SHIFT_JIS.decode(&bytes);

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());

    let headers = reader.headers().map_err(|err| err.to_string())?.clone();
    let date_idx = get_required_index(&headers, DATE)?;
    let project_code_idx = get_required_index(&headers, PROJECT_CODE)?;
    let project_name_idx = get_required_index(&headers, PROJECT_NAME)?;
    let process_code_idx = get_required_index(&headers, PROCESS_CODE)?;
    let process_name_idx = get_required_index(&headers, PROCESS_NAME)?;
    let work_content_idx = get_required_index(&headers, WORK_CONTENT)?;
    let regular_idx = get_required_index(&headers, REGULAR_MINUTES)?;
    let normal_overtime_idx = get_required_index(&headers, NORMAL_OVERTIME)?;
    let legal_holiday_idx = get_required_index(&headers, LEGAL_HOLIDAY_OVERTIME)?;
    let legal_public_holiday_idx = get_required_index(&headers, LEGAL_PUBLIC_HOLIDAY_OVERTIME)?;
    let late_night_idx = get_required_index(&headers, LATE_NIGHT_OVERTIME)?;

    let mut projects: BTreeMap<String, ProjectAccumulator> = BTreeMap::new();
    let mut grand_total = MinuteTotals::default();
    let mut row_count = 0;

    for record in reader.records() {
        let record = record.map_err(|err| err.to_string())?;
        row_count += 1;

        let date = record.get(date_idx).unwrap_or_default().trim();
        let project_code = record.get(project_code_idx).unwrap_or_default().trim();
        let project_name = record.get(project_name_idx).unwrap_or_default().trim();
        let process_code = record.get(process_code_idx).unwrap_or_default().trim();
        let process_name = record.get(process_name_idx).unwrap_or_default().trim();
        let work_content = record.get(work_content_idx).unwrap_or_default().trim();
        let phase_name = phase_label(process_code, process_name);
        let totals = MinuteTotals {
            regular_minutes: parse_minutes(record.get(regular_idx)),
            normal_overtime_minutes: parse_minutes(record.get(normal_overtime_idx)),
            legal_holiday_overtime_minutes: parse_minutes(record.get(legal_holiday_idx)),
            legal_public_holiday_overtime_minutes: parse_minutes(record.get(legal_public_holiday_idx)),
            late_night_overtime_minutes: parse_minutes(record.get(late_night_idx)),
        };
        let detail = WorkDetail {
            date: date.to_string(),
            work_content: work_content.to_string(),
            total_minutes: totals.total_minutes(),
        };

        grand_total.add(&totals);

        let project = projects.entry(project_code.to_string()).or_insert_with(|| ProjectAccumulator {
            project_code: project_code.to_string(),
            project_name: project_name.to_string(),
            ..Default::default()
        });
        if project.project_name.is_empty() {
            project.project_name = project_name.to_string();
        }
        project.row_count += 1;
        project.totals.add(&totals);

        let phase = project.phases.entry(process_code.to_string()).or_insert_with(|| PhaseAccumulator {
            process_code: process_code.to_string(),
            phase_name,
            ..Default::default()
        });
        phase.row_count += 1;
        phase.totals.add(&totals);
        phase.details.push(detail);
    }

    let projects = projects
        .into_values()
        .map(|project| {
            let mut phases: Vec<PhaseSummary> = project
                .phases
                .into_values()
                .map(|mut phase| {
                    phase.details.sort_by(|a, b| a.date.cmp(&b.date));
                    PhaseSummary {
                        process_code: phase.process_code,
                        phase_name: phase.phase_name,
                        totals: phase.totals,
                        row_count: phase.row_count,
                        details: phase.details,
                    }
                })
                .collect();
            phases.sort_by(|a, b| b.totals.total_minutes().cmp(&a.totals.total_minutes()));

            ProjectSummary {
                project_code: project.project_code,
                project_name: project.project_name,
                totals: project.totals,
                row_count: project.row_count,
                phases,
            }
        })
        .collect();

    Ok(AnalysisResult {
        source_path: display_path(&input_path),
        row_count,
        grand_total,
        projects,
    })
}

#[tauri::command]
fn get_system_info() -> SystemInfo {
    SystemInfo {
        username: env::var("USERNAME")
            .or_else(|_| env::var("USER"))
            .unwrap_or_else(|_| "unknown".to_string()),
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        ip_address: local_ip_address(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

fn local_ip_address() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn display_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![analyze_csv, get_system_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
