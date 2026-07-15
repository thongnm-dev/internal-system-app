//! Business logic cho việc gọi Backlog API.
//!
//! Sử dụng reqwest HTTP client để gọi Backlog REST API v2.
//! Xác thực bằng API key qua query parameter `apiKey`.
//! Thông tin kết nối được đọc từ bảng `api_keys` trong database
//! với `name = 'ALX_BACKLOG'`.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::api_key_store;
use crate::models::backlog::*;
use crate::utils::pgsql_connect;
use ini::Ini;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;

const BACKLOG_NAME: &str = "ALX_BACKLOG";
const BACKLOG_URL_LABEL: &str = "ALX_BACKLOG_URL";
const BACKLOG_APIKEY_LABEL: &str = "ALX_BACKLOG_APIKEY";

struct BacklogClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl BacklogClient {
    async fn resolve() -> AppResult<Self> {
        if let Ok(client) = Self::from_db().await {
            return Ok(client);
        }
        Self::from_ini()
    }

    async fn from_db() -> AppResult<Self> {
        let base_url = api_key_store::get_value(BACKLOG_NAME, BACKLOG_URL_LABEL).await?;
        let api_key = api_key_store::get_value(BACKLOG_NAME, BACKLOG_APIKEY_LABEL).await?;
        Self::build(&base_url, &api_key)
    }

    fn from_ini() -> AppResult<Self> {
        let path = pgsql_connect::config_path();
        let ini = Ini::load_from_file(&path).map_err(|e| {
            AppError::new(format!("Failed to load config.ini at {}: {e}", path.display()))
        })?;

        let section = ini
            .section(Some("backlog"))
            .ok_or_else(|| AppError::new(
                "Backlog chưa được cấu hình. Thêm section [backlog] trong config.ini hoặc cấu hình trong Settings → API Keys."
            ))?;

        let base_url = section.get(BACKLOG_URL_LABEL).unwrap_or("").to_string();
        let api_key = section.get(BACKLOG_APIKEY_LABEL).unwrap_or("").to_string();
        Self::build(&base_url, &api_key)
    }

    fn build(base_url: &str, api_key: &str) -> AppResult<Self> {
        let base_url = base_url
            .trim()
            .trim_end_matches('/')
            .trim_end_matches("/api/v2")
            .trim_end_matches('/')
            .to_string();
        let api_key = api_key.trim().to_string();

        if base_url.is_empty() {
            return Err(AppError::new(
                "Backlog URL chưa được cấu hình.",
            ));
        }
        if api_key.is_empty() {
            return Err(AppError::new(
                "Backlog API Key chưa được cấu hình.",
            ));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::new(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> AppResult<T> {
        let url = format!("{}/api/v2/{}", self.base_url, path.trim_start_matches('/'));

        let mut params: Vec<(&str, &str)> = vec![("apiKey", &self.api_key)];
        params.extend_from_slice(query);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| AppError::new(format!("Request to Backlog failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::new(format!(
                "Backlog API error {}: {body}",
                status.as_u16()
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| AppError::new(format!("Failed to parse Backlog response: {e}")))
    }
}

pub async fn get_space() -> AppResult<BacklogSpace> {
    let client = BacklogClient::resolve().await?;
    client.get("space", &[]).await
}

pub async fn get_project(project_key: &str) -> AppResult<BacklogProject> {
    let client = BacklogClient::resolve().await?;
    client
        .get(&format!("projects/{project_key}"), &[])
        .await
}

pub async fn list_issue_types(project_key: &str) -> AppResult<Vec<BacklogIssueType>> {
    let client = BacklogClient::resolve().await?;
    client
        .get(&format!("projects/{project_key}/issueTypes"), &[])
        .await
}

pub async fn list_statuses(project_key: &str) -> AppResult<Vec<BacklogStatus>> {
    let client = BacklogClient::resolve().await?;
    client
        .get(&format!("projects/{project_key}/statuses"), &[])
        .await
}

pub async fn list_categories(project_key: &str) -> AppResult<Vec<BacklogCategory>> {
    let client = BacklogClient::resolve().await?;
    client
        .get(&format!("projects/{project_key}/categories"), &[])
        .await
}

pub async fn list_milestones(project_key: &str) -> AppResult<Vec<BacklogMilestone>> {
    let client = BacklogClient::resolve().await?;
    client
        .get(&format!("projects/{project_key}/versions"), &[])
        .await
}

pub async fn list_project_users(project_key: &str) -> AppResult<Vec<BacklogUser>> {
    let client = BacklogClient::resolve().await?;
    client
        .get(&format!("projects/{project_key}/users"), &[])
        .await
}

pub async fn list_issues(query: BacklogIssueQuery) -> AppResult<BacklogIssueList> {
    let client = BacklogClient::resolve().await?;

    let count_str = query.count.to_string();
    let offset_str = query.offset.to_string();

    let mut params: Vec<(String, String)> = vec![
        ("projectId[]".to_string(), query.project_key.clone()),
        ("count".to_string(), count_str),
        ("offset".to_string(), offset_str),
    ];

    if let Some(ref ids) = query.status_ids {
        for id in ids {
            params.push(("statusId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref ids) = query.assignee_ids {
        for id in ids {
            params.push(("assigneeId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref ids) = query.issue_type_ids {
        for id in ids {
            params.push(("issueTypeId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref ids) = query.category_ids {
        for id in ids {
            params.push(("categoryId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref ids) = query.milestone_ids {
        for id in ids {
            params.push(("milestoneId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref kw) = query.keyword {
        if !kw.is_empty() {
            params.push(("keyword".to_string(), kw.clone()));
        }
    }
    if let Some(ref sort) = query.sort {
        params.push(("sort".to_string(), sort.clone()));
    }
    if let Some(ref order) = query.order {
        params.push(("order".to_string(), order.clone()));
    }
    if let Some(parent_id) = query.parent_issue_id {
        params.push(("parentIssueId[]".to_string(), parent_id.to_string()));
    }

    let query_refs: Vec<(&str, &str)> = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let issues: Vec<BacklogIssue> = client.get("issues", &query_refs).await?;

    let mut count_params: Vec<(String, String)> =
        vec![("projectId[]".to_string(), query.project_key.clone())];
    if let Some(ref ids) = query.status_ids {
        for id in ids {
            count_params.push(("statusId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref ids) = query.assignee_ids {
        for id in ids {
            count_params.push(("assigneeId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref ids) = query.issue_type_ids {
        for id in ids {
            count_params.push(("issueTypeId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref ids) = query.category_ids {
        for id in ids {
            count_params.push(("categoryId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref ids) = query.milestone_ids {
        for id in ids {
            count_params.push(("milestoneId[]".to_string(), id.to_string()));
        }
    }
    if let Some(ref kw) = query.keyword {
        if !kw.is_empty() {
            count_params.push(("keyword".to_string(), kw.clone()));
        }
    }
    if let Some(parent_id) = query.parent_issue_id {
        count_params.push(("parentIssueId[]".to_string(), parent_id.to_string()));
    }

    let count_refs: Vec<(&str, &str)> = count_params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let count_result: BacklogIssueCount = client.get("issues/count", &count_refs).await?;

    Ok(BacklogIssueList {
        issues,
        total_count: count_result.count,
    })
}

pub async fn get_issue(issue_key: &str) -> AppResult<BacklogIssue> {
    let client = BacklogClient::resolve().await?;
    client.get(&format!("issues/{issue_key}"), &[]).await
}

pub async fn get_project_lookup(project_key: &str) -> AppResult<BacklogProjectLookup> {
    let project = get_project(project_key).await?;
    Ok(BacklogProjectLookup {
        project_id: project.id.to_string(),
        project_key: project.project_key,
        project_name: project.name,
    })
}
