use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub bucket: String,
    pub endpoint_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Object {
    pub key: String,
    pub display_name: String,
    pub size: u64,
    pub last_modified: String,
    pub is_folder: bool,
    pub etag: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3ListResult {
    pub objects: Vec<S3Object>,
    pub current_prefix: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3OperationResult {
    pub success: bool,
    pub message: String,
    pub processed: u32,
    pub failed: u32,
}
