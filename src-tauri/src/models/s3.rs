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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwsStorage {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub name_alias: String,
    pub subscribe: String,
    pub is_upload: bool,
    pub is_download: bool,
    pub file_only: bool,
    pub link_available: Vec<String>,
    pub exclude_subscribe: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileRequest {
    pub parent_name: String,
    pub name: String,
    pub local_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedFile {
    pub parent_name: String,
    pub name: String,
    pub file_path: String,
    pub full_path: String,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteUploadedItem {
    pub aws_cd: String,
    pub bug_no: String,
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
