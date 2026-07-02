use crate::app::error::AppError;
use crate::app::result::AppResult;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> AppResult<Self> {
        let client = Client::builder()
            .build()
            .map_err(|error| AppError::new(format!("Failed to create HTTP client: {error}")))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    pub fn with_timeout(base_url: &str, timeout: std::time::Duration) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|error| AppError::new(format!("Failed to create HTTP client: {error}")))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> AppResult<T> {
        let url = self.url(path);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|error| AppError::new(format!("GET {url} failed: {error}")))?;

        parse_response(response, &url).await
    }

    pub async fn get_with_headers<T: DeserializeOwned>(
        &self,
        path: &str,
        headers: HeaderMap,
    ) -> AppResult<T> {
        let url = self.url(path);
        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| AppError::new(format!("GET {url} failed: {error}")))?;

        parse_response(response, &url).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> AppResult<T> {
        let url = self.url(path);
        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|error| AppError::new(format!("POST {url} failed: {error}")))?;

        parse_response(response, &url).await
    }

    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> AppResult<T> {
        let url = self.url(path);
        let response = self
            .client
            .put(&url)
            .json(body)
            .send()
            .await
            .map_err(|error| AppError::new(format!("PUT {url} failed: {error}")))?;

        parse_response(response, &url).await
    }

    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> AppResult<T> {
        let url = self.url(path);
        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|error| AppError::new(format!("DELETE {url} failed: {error}")))?;

        parse_response(response, &url).await
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }
}

pub fn bearer_auth(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) {
        headers.insert(AUTHORIZATION, value);
    }
    headers
}

pub fn api_key_header(key_name: &str, key_value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let (Ok(name), Ok(value)) = (
        HeaderName::from_bytes(key_name.as_bytes()),
        HeaderValue::from_str(key_value),
    ) {
        headers.insert(name, value);
    }
    headers
}

async fn parse_response<T: DeserializeOwned>(response: Response, url: &str) -> AppResult<T> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::new(format!(
            "{} {url}: {body}",
            status.as_u16()
        )));
    }

    response
        .json::<T>()
        .await
        .map_err(|error| AppError::new(format!("Failed to parse response from {url}: {error}")))
}
