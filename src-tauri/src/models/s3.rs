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
pub struct LocalFileEntry {
    pub name: String,
    pub relative_path: String,
    pub full_path: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadAvailability {
    pub download_available: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3OperationResult {
    pub success: bool,
    pub message: String,
    pub processed: u32,
    pub failed: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadByStorageResult {
    pub success: bool,
    pub message: String,
    pub processed: u32,
    pub failed: u32,
    pub sync_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadHistorySearchParams {
    pub from_date: String,
    pub to_date: String,
    pub aws_cd: String,
    pub bug_no: String,
    pub is_moved_at_local: bool,
    pub is_moved_at_s3: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadHistorySearchItem {
    pub id: i32,
    pub download_ymd: String,
    pub aws_cd: String,
    pub aws_name: String,
    pub is_moved_at_local: bool,
    pub bug_no: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadHistoryItem {
    pub id: i32,
    pub download_ymd: String,
    pub download_hms: String,
    pub sync_path: String,
    pub download_count: i32,
    pub is_moved_at_local: bool,
    pub aws_name: String,
    pub aws_name_alias: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadHistorySearchParams {
    pub from_date: String,
    pub to_date: String,
    pub aws_cd: String,
    pub bug_no: String,
    pub is_moved_at_s3: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBugFolders {
    pub storage: AwsStorage,
    pub bugs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BugFolderItem {
    pub bug_no: String,
    pub in_subscribe: bool,
    pub last_modified: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BugFolderTab {
    pub name: String,
    pub name_alias: String,
    pub items: Vec<BugFolderItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadHistorySearchItem {
    pub upload_ymd: String,
    pub aws_name: String,
    pub is_moved_at_s3: bool,
    pub bug_no: String,
    pub att_files: String,
}
