//! Các kiểu dữ liệu (model) cho Backlog API responses.

use serde::{Deserialize, Serialize};
/// Thông tin dự án từ Backlog API.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogProject {
    pub id: i64,
    pub project_key: String,
    pub name: String,
    pub archived: Option<bool>,
    pub text_formatting_rule: Option<String>,
    pub display_order: Option<i64>,
}

/// Loại issue trong dự án Backlog.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogIssueType {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub color: Option<String>,
    pub display_order: Option<i64>,
}

/// Trạng thái issue trong dự án Backlog.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogStatus {
    pub id: i64,
    pub project_id: Option<i64>,
    pub name: String,
    pub color: Option<String>,
    pub display_order: Option<i64>,
}

/// Danh mục (category) trong dự án Backlog.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogCategory {
    pub id: i64,
    pub name: String,
    pub display_order: Option<i64>,
}

/// Thông tin user trên Backlog.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogUser {
    pub id: i64,
    pub user_id: Option<String>,
    pub name: String,
    pub role_type: Option<i64>,
    pub lang: Option<String>,
    pub mail_address: Option<String>,
    pub keyword: Option<String>,
}

/// Priority của issue.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogPriority {
    pub id: i64,
    pub name: String,
}

/// Milestone / version trong dự án.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogMilestone {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub release_due_date: Option<String>,
    pub archived: Option<bool>,
    pub display_order: Option<i64>,
}

/// Thông tin issue từ Backlog API.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogIssue {
    pub id: i64,
    pub project_id: i64,
    pub issue_key: String,
    pub key_id: i64,
    pub issue_type: Option<BacklogIssueType>,
    pub summary: String,
    pub description: Option<String>,
    pub priority: Option<BacklogPriority>,
    pub status: Option<BacklogStatus>,
    pub assignee: Option<BacklogUser>,
    pub category: Option<Vec<BacklogCategory>>,
    pub milestone: Option<Vec<BacklogMilestone>>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub parent_issue_id: Option<i64>,
    pub created_user: Option<BacklogUser>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

/// Tham số lọc khi lấy danh sách issues.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogIssueQuery {
    pub project_key: String,
    #[serde(default = "default_count")]
    pub count: i32,
    #[serde(default)]
    pub offset: i32,
    pub status_ids: Option<Vec<i64>>,
    pub assignee_ids: Option<Vec<i64>>,
    pub issue_type_ids: Option<Vec<i64>>,
    pub category_ids: Option<Vec<i64>>,
    pub milestone_ids: Option<Vec<i64>>,
    pub priority_ids: Option<Vec<i64>>,
    pub keyword: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub parent_issue_id: Option<i64>,
}

fn default_count() -> i32 {
    100
}

/// Kết quả đếm issues.
#[derive(Debug, Serialize, Deserialize)]
pub struct BacklogIssueCount {
    pub count: i64,
}

/// Danh sách issues kèm tổng số.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogIssueList {
    pub issues: Vec<BacklogIssue>,
    pub total_count: i64,
}

/// Kết quả tra cứu dự án từ Backlog API theo project key.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklogProjectLookup {
    pub project_id: String,
    pub project_key: String,
    pub project_name: String,
}
