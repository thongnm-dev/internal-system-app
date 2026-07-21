use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCsvRow {
    pub subject: String,
    pub description: String,
    pub issue_type: String,
    pub assignee: String,
    pub start_date: String,
    pub due_date: String,
    pub estimated_hours: String,
    pub actual_hours: String,
    pub categories: String,
    pub version: String,
    pub milestones: String,
    pub priority: String,
    pub parent_issue: String,
    pub bug_types: String,
    pub bug_severity_levels: String,
    pub test_phase: String,
}
