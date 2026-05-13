use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::domain::statistics::models::{
    AnalysisResult, MinuteTotals, PhaseAccumulator, PhaseSummary, ProjectAccumulator,
    ProjectSummary, WorkDetail,
};
use crate::domain::statistics::phase::phase_label;
use crate::infrastructure::csv::reader::{get_required_index, parse_minutes, read_shift_jis_csv};
use crate::infrastructure::file_system::{display_path, resolve_input_path};
use std::collections::BTreeMap;

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

pub fn analyze_csv(path: &str) -> AppResult<AnalysisResult> {
    let input_path = resolve_input_path(path)?;
    let content = read_shift_jis_csv(&input_path)?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());

    let headers = reader.headers()?.clone();
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
        let record = record?;
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
        append_record(&mut projects, project_code, project_name, process_code, phase_name, totals, detail);
    }

    Ok(AnalysisResult {
        source_path: display_path(&input_path),
        row_count,
        grand_total,
        projects: build_project_summaries(projects),
    })
}

fn append_record(
    projects: &mut BTreeMap<String, ProjectAccumulator>,
    project_code: &str,
    project_name: &str,
    process_code: &str,
    phase_name: String,
    totals: MinuteTotals,
    detail: WorkDetail,
) {
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

fn build_project_summaries(projects: BTreeMap<String, ProjectAccumulator>) -> Vec<ProjectSummary> {
    projects
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
        .collect()
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        AppError::new(value)
    }
}
