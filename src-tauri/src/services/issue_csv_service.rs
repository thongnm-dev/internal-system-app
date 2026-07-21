use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::issue_csv::IssueCsvRow;
use crate::utils::csv_reader::CsvReader;
use std::path::Path;

const EXPECTED_HEADERS: [&str; 16] = [
    "Subject",
    "Description",
    "Issue Type",
    "Assignee",
    "Start Date",
    "Due Date",
    "Estimated Hours",
    "Actual Hours",
    "Categories",
    "Version",
    "Milestones",
    "Priority",
    "Parent issue",
    "Bug Types",
    "Bug severity levels",
    "Test Phase",
];

pub fn parse_issue_csv(path: &Path) -> AppResult<Vec<IssueCsvRow>> {
    let csv = CsvReader::from_path(path)?;

    let headers = csv.headers();
    if headers.len() < EXPECTED_HEADERS.len() {
        return Err(AppError::new("CSV headers do not match the issue import template."));
    }
    for (i, expected) in EXPECTED_HEADERS.iter().enumerate() {
        if headers[i] != *expected {
            return Err(AppError::new("CSV headers do not match the issue import template."));
        }
    }

    let subject_idx = csv.column_index(EXPECTED_HEADERS[0])?;
    let description_idx = csv.column_index(EXPECTED_HEADERS[1])?;
    let issue_type_idx = csv.column_index(EXPECTED_HEADERS[2])?;
    let assignee_idx = csv.column_index(EXPECTED_HEADERS[3])?;
    let start_date_idx = csv.column_index(EXPECTED_HEADERS[4])?;
    let due_date_idx = csv.column_index(EXPECTED_HEADERS[5])?;
    let estimated_hours_idx = csv.column_index(EXPECTED_HEADERS[6])?;
    let actual_hours_idx = csv.column_index(EXPECTED_HEADERS[7])?;
    let categories_idx = csv.column_index(EXPECTED_HEADERS[8])?;
    let version_idx = csv.column_index(EXPECTED_HEADERS[9])?;
    let milestones_idx = csv.column_index(EXPECTED_HEADERS[10])?;
    let priority_idx = csv.column_index(EXPECTED_HEADERS[11])?;
    let parent_issue_idx = csv.column_index(EXPECTED_HEADERS[12])?;
    let bug_types_idx = csv.column_index(EXPECTED_HEADERS[13])?;
    let bug_severity_levels_idx = csv.column_index(EXPECTED_HEADERS[14])?;
    let test_phase_idx = csv.column_index(EXPECTED_HEADERS[15])?;

    let rows = csv
        .rows()
        .iter()
        .map(|row| IssueCsvRow {
            subject: csv.get_value(row, subject_idx).to_string(),
            description: csv.get_value(row, description_idx).to_string(),
            issue_type: csv.get_value(row, issue_type_idx).to_string(),
            assignee: csv.get_value(row, assignee_idx).to_string(),
            start_date: csv.get_value(row, start_date_idx).to_string(),
            due_date: csv.get_value(row, due_date_idx).to_string(),
            estimated_hours: csv.get_value(row, estimated_hours_idx).to_string(),
            actual_hours: csv.get_value(row, actual_hours_idx).to_string(),
            categories: csv.get_value(row, categories_idx).to_string(),
            version: csv.get_value(row, version_idx).to_string(),
            milestones: csv.get_value(row, milestones_idx).to_string(),
            priority: csv.get_value(row, priority_idx).to_string(),
            parent_issue: csv.get_value(row, parent_issue_idx).to_string(),
            bug_types: csv.get_value(row, bug_types_idx).to_string(),
            bug_severity_levels: csv.get_value(row, bug_severity_levels_idx).to_string(),
            test_phase: csv.get_value(row, test_phase_idx).to_string(),
        })
        .collect();

    Ok(rows)
}
