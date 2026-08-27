use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::s3::{AwsStorage, BugFolderItem, BugFolderTab, DeleteUploadedItem, DownloadAvailability, DownloadByStorageResult, DownloadHistoryItem, DownloadHistorySearchItem, DownloadHistorySearchParams, LocalFileEntry, S3Config, S3ListResult, S3Object, S3OperationResult, ScannedFile, StorageBugFolders, UploadFileRequest, UploadHistorySearchItem, UploadHistorySearchParams};
use crate::utils::app_config;

use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::retry::RetryConfig;
use aws_sdk_s3::config::timeout::TimeoutConfig;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::Client;
use futures_util::{stream, StreamExt};
use ini::Ini;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{OnceLock, RwLock as StdRwLock};
use std::time::Duration;

// Total time budget (including all internal SDK retries) allowed for a single S3
// request before it gives up. Bounds every S3 operation to at most this long when
// the network is unreachable, instead of hanging indefinitely.
const S3_OPERATION_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const S3_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

// Max number of S3 requests (upload/download/delete/copy) allowed in flight at
// once for a single multi-item operation. Bounds memory/socket usage while still
// giving a large speedup over one-at-a-time sequential transfers.
const S3_CONCURRENCY: usize = 8;

const S3_NETWORK_ERROR_MESSAGE: &str =
    "Lỗi không thể kết nối mạng. Vui lòng kiểm tra kết nối internet!";

/// True when an S3 SDK error means the request never reached AWS (timed out or
/// failed to dispatch), i.e. a real connectivity problem rather than a service-side
/// rejection (bad key, permission, etc.).
fn is_connectivity_error<E, R>(err: &SdkError<E, R>) -> bool {
    matches!(
        err,
        SdkError::TimeoutError(_) | SdkError::DispatchFailure(_)
    )
}

/// Converts an S3 SDK error into an [`AppError`], substituting the friendly
/// Vietnamese network-error message when the failure is a connectivity issue.
fn s3_error<E, R>(context: &str, err: SdkError<E, R>) -> AppError {
    if is_connectivity_error(&err) {
        AppError::new(S3_NETWORK_ERROR_MESSAGE)
    } else {
        AppError::new(format!("{context}: {err}"))
    }
}

/// Same as [`s3_error`] but returns plain text for accumulation into a per-item
/// `errors` list instead of an [`AppError`].
fn s3_error_text<E, R>(context: &str, err: &SdkError<E, R>) -> String {
    if is_connectivity_error(err) {
        S3_NETWORK_ERROR_MESSAGE.to_string()
    } else {
        format!("{context}: {err}")
    }
}

pub(crate) fn load_config_from_ini() -> AppResult<S3Config> {
    let path = app_config::config_path();
    let ini = Ini::load_from_file(&path).map_err(|e| {
        AppError::new(format!("Failed to load config.ini at {}: {e}", path.display()))
    })?;

    let section = ini.section(Some("S3 bucket")).ok_or_else(|| {
        AppError::new("Section [S3 bucket] not found in config.ini.")
    })?;

    Ok(S3Config {
        access_key_id: section.get("AWS_ACCESS_KEY_ID").unwrap_or("").to_string(),
        secret_access_key: section.get("AWS_SECRET_ACCESS_KEY").unwrap_or("").to_string(),
        region: section.get("AWS_REGION").unwrap_or("ap-northeast-1").to_string(),
        bucket: section.get("AWS_S3_BUCKET").unwrap_or("").to_string(),
        endpoint_url: section.get("AWS_ENDPOINT_URL").map(|s| s.to_string()),
    })
}

fn build_client(config: &S3Config) -> AppResult<(Client, String)> {
    let credentials = Credentials::new(
        &config.access_key_id,
        &config.secret_access_key,
        None,
        None,
        "s3-browser",
    );

    let region = Region::new(config.region.clone());
    let mut builder = aws_sdk_s3::config::Builder::new()
        .region(region)
        .credentials_provider(credentials)
        .behavior_version_latest()
        .retry_config(RetryConfig::standard().with_max_attempts(4))
        .timeout_config(
            TimeoutConfig::builder()
                .connect_timeout(S3_CONNECT_TIMEOUT)
                .operation_timeout(S3_OPERATION_TIMEOUT)
                .build(),
        );

    if let Some(ref endpoint) = config.endpoint_url {
        let ep = endpoint.trim();
        if !ep.is_empty() {
            builder = builder
                .endpoint_url(ep)
                .force_path_style(true);
        }
    }

    let client = Client::from_conf(builder.build());
    Ok((client, config.bucket.clone()))
}

/// Cache holding the last-built `Client` alongside the `S3Config` it was built
/// from. `Client` is cheap to clone (internally `Arc`-backed), so callers get a
/// shared, already-connected client instead of re-creating one (and redoing the
/// TLS handshake) on every single S3 command.
static CLIENT_CACHE: OnceLock<StdRwLock<Option<(S3Config, Client, String)>>> = OnceLock::new();

/// Returns a cached `(Client, bucket)` pair, rebuilding it only when
/// `config.ini` has changed since the last call.
fn get_or_build_client() -> AppResult<(Client, String)> {
    let config = load_config_from_ini()?;
    let cache = CLIENT_CACHE.get_or_init(|| StdRwLock::new(None));

    if let Some((cached_config, client, bucket)) = cache.read().unwrap().as_ref() {
        if cached_config == &config {
            return Ok((client.clone(), bucket.clone()));
        }
    }

    let (client, bucket) = build_client(&config)?;
    *cache.write().unwrap() = Some((config, client.clone(), bucket.clone()));
    Ok((client, bucket))
}

/// Downloads a single object to `local_path`, creating parent directories as
/// needed. Used as the per-item unit of work for concurrent batch downloads.
async fn download_one_object(
    client: &Client,
    bucket: &str,
    key: &str,
    local_path: &Path,
) -> Result<(), String> {
    if let Some(parent) = local_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("{key}: failed to create directory: {e}"))?;
    }

    let output = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| s3_error_text(&format!("{key}: download failed"), &e))?;

    let data = output
        .body
        .collect()
        .await
        .map_err(|e| format!("{key}: read stream failed: {e}"))?;

    tokio::fs::write(local_path, data.into_bytes())
        .await
        .map_err(|e| format!("{key}: write failed: {e}"))
}

/// Uploads a single local file to `s3_key`. Used as the per-item unit of work
/// for concurrent batch uploads.
async fn upload_one_file(
    client: &Client,
    bucket: &str,
    local_path: &Path,
    s3_key: &str,
) -> Result<(), String> {
    let body = tokio::fs::read(local_path)
        .await
        .map_err(|e| format!("read failed: {e}"))?;

    client
        .put_object()
        .bucket(bucket)
        .key(s3_key)
        .body(body.into())
        .send()
        .await
        .map_err(|e| s3_error_text("upload failed", &e))?;

    Ok(())
}

/// Deletes a single object. Used as the per-item unit of work for concurrent
/// batch deletes.
async fn delete_one_object(client: &Client, bucket: &str, key: &str) -> Result<(), String> {
    client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map(|_| ())
        .map_err(|e| s3_error_text(key, &e))
}

/// Server-side copies `key` to `target_key`, then deletes the source. Used as
/// the per-item unit of work for concurrent batch moves.
async fn move_one_object(
    client: &Client,
    bucket: &str,
    key: &str,
    target_key: &str,
) -> Result<(), String> {
    let encoded_key = key
        .split('/')
        .map(|seg| urlencoding::encode(seg).into_owned())
        .collect::<Vec<_>>()
        .join("/");
    let copy_source = format!("{}/{}", bucket, encoded_key);

    client
        .copy_object()
        .bucket(bucket)
        .copy_source(&copy_source)
        .key(target_key)
        .send()
        .await
        .map_err(|e| s3_error_text(&format!("{key}: copy failed"), &e))?;

    client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| s3_error_text(&format!("{key}: delete after copy failed"), &e))?;

    Ok(())
}

pub fn get_config() -> AppResult<S3Config> {
    load_config_from_ini()
}

pub fn save_config(config: &S3Config) -> AppResult<()> {
    let path = app_config::config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::new(format!("Failed to create config directory {}: {e}", parent.display()))
        })?;
    }

    let mut ini = if path.exists() {
        Ini::load_from_file(&path).unwrap_or_else(|_| Ini::new())
    } else {
        Ini::new()
    };

    let mut section = ini.with_section(Some("S3 bucket"));
    section
        .set("AWS_REGION", &config.region)
        .set("AWS_ACCESS_KEY_ID", &config.access_key_id)
        .set("AWS_SECRET_ACCESS_KEY", &config.secret_access_key)
        .set("AWS_S3_BUCKET", &config.bucket);

    if let Some(ref endpoint) = config.endpoint_url {
        section.set("AWS_ENDPOINT_URL", endpoint);
    }

    ini.write_to_file(&path).map_err(|e| {
        AppError::new(format!("Failed to write config.ini at {}: {e}", path.display()))
    })?;
    Ok(())
}

pub fn get_local_sync_workdir() -> AppResult<String> {
    let path = app_config::config_path();
    let ini = Ini::load_from_file(&path).map_err(|e| {
        AppError::new(format!("Failed to load config.ini at {}: {e}", path.display()))
    })?;
    let section = ini.section(Some("S3 bucket")).ok_or_else(|| {
        AppError::new("Section [S3 bucket] not found in config.ini.")
    })?;
    Ok(section.get("S3_LOCAL_SYNC_WORKDIR").unwrap_or("").to_string())
}

pub fn check_config() -> AppResult<()> {
    let config = load_config_from_ini()?;
    if config.access_key_id.is_empty()
        || config.secret_access_key.is_empty()
        || config.bucket.is_empty()
    {
        return Err(AppError::new("Thông tin cấu hình S3 chưa được thiết lập"));
    }
    Ok(())
}

pub async fn test_connection() -> AppResult<String> {
    let (client, bucket) = get_or_build_client()?;
    client
        .head_bucket()
        .bucket(&bucket)
        .send()
        .await
        .map_err(|e| s3_error("Connection failed", e))?;
    Ok(format!("Connected to bucket '{bucket}' successfully."))
}

pub async fn list_objects(prefix: String) -> AppResult<S3ListResult> {
    let (client, bucket) = get_or_build_client()?;

    let mut request = client
        .list_objects_v2()
        .bucket(&bucket)
        .delimiter("/");

    if !prefix.is_empty() {
        request = request.prefix(&prefix);
    }

    let output = request
        .send()
        .await
        .map_err(|e| s3_error("Failed to list objects", e))?;

    let mut objects: Vec<S3Object> = Vec::new();

    for p in output.common_prefixes() {
        if let Some(pref) = p.prefix() {
            let display = pref
                .strip_prefix(&prefix)
                .unwrap_or(pref)
                .trim_end_matches('/');
            if !display.is_empty() {
                objects.push(S3Object {
                    key: pref.to_string(),
                    display_name: display.to_string(),
                    size: 0,
                    last_modified: String::new(),
                    is_folder: true,
                    etag: String::new(),
                });
            }
        }
    }

    for obj in output.contents() {
        let key: &str = obj.key().unwrap_or_default();
        if key == prefix {
            continue;
        }
        let display = key.strip_prefix(&prefix).unwrap_or(key);
        if display.is_empty() || display.ends_with('/') {
            continue;
        }
        let last_modified = obj
            .last_modified()
            .map(|dt: &aws_sdk_s3::primitives::DateTime| {
                dt.fmt(aws_sdk_s3::primitives::DateTimeFormat::DateTime)
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        objects.push(S3Object {
            key: key.to_string(),
            display_name: display.to_string(),
            size: obj.size().unwrap_or(0) as u64,
            last_modified,
            is_folder: false,
            etag: obj.e_tag().unwrap_or_default().to_string(),
        });
    }

    Ok(S3ListResult {
        objects,
        current_prefix: prefix,
    })
}

pub async fn download_objects(
    keys: Vec<String>,
    destination_dir: String,
    strip_prefix: String,
) -> AppResult<S3OperationResult> {
    let (client, bucket) = get_or_build_client()?;
    let dest = Path::new(&destination_dir);
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    let (folder_keys, mut all_keys): (Vec<String>, Vec<String>) =
        keys.into_iter().partition(|k| k.ends_with('/'));

    // Expand folder prefixes to individual object keys — listing each folder is
    // metadata-only (cheap) but still a network round-trip, so run them all
    // concurrently instead of one folder at a time.
    if !folder_keys.is_empty() {
        let expansions: Vec<(String, AppResult<Vec<String>>)> =
            stream::iter(folder_keys.into_iter().map(|folder_prefix| {
                let client = client.clone();
                let bucket = bucket.clone();
                async move {
                    let r = list_all_objects_recursive(&client, &bucket, &folder_prefix).await;
                    (folder_prefix, r)
                }
            }))
            .buffer_unordered(S3_CONCURRENCY)
            .collect()
            .await;

        for (folder_prefix, r) in expansions {
            match r {
                Ok(child_keys) => {
                    all_keys.extend(child_keys.into_iter().filter(|k| !k.ends_with('/')));
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{folder_prefix}: {e}"));
                }
            }
        }
    }

    let targets: Vec<(String, std::path::PathBuf)> = all_keys
        .iter()
        .map(|key| {
            let relative = key.strip_prefix(strip_prefix.as_str()).unwrap_or(key.as_str());
            (key.clone(), dest.join(relative))
        })
        .collect();

    let results: Vec<Result<(), String>> =
        stream::iter(targets.into_iter().map(|(key, local_path)| {
            let client = client.clone();
            let bucket = bucket.clone();
            async move { download_one_object(&client, &bucket, &key, &local_path).await }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    for r in results {
        match r {
            Ok(()) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(e);
            }
        }
    }

    let message = if errors.is_empty() {
        format!("Downloaded {processed} file(s) successfully.")
    } else {
        format!(
            "Downloaded {processed} file(s), {failed} failed.\n{}",
            errors.join("\n")
        )
    };

    Ok(S3OperationResult {
        success: failed == 0,
        message,
        processed,
        failed,
    })
}

async fn list_all_objects_recursive(
    client: &Client,
    bucket: &str,
    prefix: &str,
) -> AppResult<Vec<String>> {
    let mut keys = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut request = client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(prefix);

        if let Some(ref token) = continuation_token {
            request = request.continuation_token(token);
        }

        let output = request
            .send()
            .await
            .map_err(|e| s3_error(&format!("Failed to list objects under '{prefix}'"), e))?;

        for obj in output.contents() {
            if let Some(key) = obj.key() {
                keys.push(key.to_string());
            }
        }

        if output.is_truncated() == Some(true) {
            continuation_token = output.next_continuation_token().map(String::from);
        } else {
            break;
        }
    }

    Ok(keys)
}

async fn list_subfolders_with_dates(
    client: &Client,
    bucket: &str,
    prefix: &str,
) -> HashMap<String, String> {
    let mut folder_dates: HashMap<String, String> = HashMap::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut request = client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(prefix);

        if let Some(ref token) = continuation_token {
            request = request.continuation_token(token);
        }

        let output = match request.send().await {
            Ok(o) => o,
            Err(_) => break,
        };

        for obj in output.contents() {
            let key = match obj.key() {
                Some(k) => k,
                None => continue,
            };
            let rest = match key.strip_prefix(prefix) {
                Some(r) => r,
                None => continue,
            };
            let folder_name = match rest.find('/') {
                Some(pos) => &rest[..pos],
                None => continue,
            };
            if folder_name.is_empty() {
                continue;
            }

            let last_modified = obj
                .last_modified()
                .map(|dt| {
                    dt.fmt(aws_sdk_s3::primitives::DateTimeFormat::DateTime)
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            folder_dates
                .entry(folder_name.to_string())
                .and_modify(|existing| {
                    if last_modified > *existing {
                        *existing = last_modified.clone();
                    }
                })
                .or_insert(last_modified);
        }

        if output.is_truncated() == Some(true) {
            continuation_token = output.next_continuation_token().map(String::from);
        } else {
            break;
        }
    }

    folder_dates
}

pub async fn upload_file(
    local_path: String,
    s3_key: String,
) -> AppResult<S3OperationResult> {
    let (client, bucket) = get_or_build_client()?;
    let path = Path::new(&local_path);

    if !path.exists() {
        return Err(AppError::new(format!("File not found: {local_path}")));
    }

    let body = tokio::fs::read(path)
        .await
        .map_err(|e| AppError::new(format!("Failed to read file: {e}")))?;

    client
        .put_object()
        .bucket(&bucket)
        .key(&s3_key)
        .body(body.into())
        .send()
        .await
        .map_err(|e| s3_error("Upload failed", e))?;

    Ok(S3OperationResult {
        success: true,
        message: format!("Uploaded '{s3_key}' successfully."),
        processed: 1,
        failed: 0,
    })
}

pub async fn delete_objects(
    keys: Vec<String>,
) -> AppResult<S3OperationResult> {
    let (client, bucket) = get_or_build_client()?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    // Expand folder prefixes — listing each folder is a separate network
    // round-trip, so run them all concurrently instead of one at a time.
    let (folder_keys, mut all_keys): (Vec<String>, Vec<String>) =
        keys.into_iter().partition(|k| k.ends_with('/'));

    if !folder_keys.is_empty() {
        let expansions: Vec<(String, AppResult<Vec<String>>)> =
            stream::iter(folder_keys.into_iter().map(|folder_key| {
                let client = client.clone();
                let bucket = bucket.clone();
                async move {
                    let r = list_all_objects_recursive(&client, &bucket, &folder_key).await;
                    (folder_key, r)
                }
            }))
            .buffer_unordered(S3_CONCURRENCY)
            .collect()
            .await;

        for (folder_key, r) in expansions {
            match r {
                Ok(mut child_keys) => {
                    if !child_keys.contains(&folder_key) {
                        child_keys.push(folder_key);
                    }
                    all_keys.extend(child_keys);
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{folder_key}: {e}"));
                }
            }
        }
    }

    let results: Vec<Result<(), String>> = stream::iter(all_keys.iter().cloned().map(|key| {
        let client = client.clone();
        let bucket = bucket.clone();
        async move { delete_one_object(&client, &bucket, &key).await }
    }))
    .buffer_unordered(S3_CONCURRENCY)
    .collect()
    .await;

    for r in results {
        match r {
            Ok(()) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(e);
            }
        }
    }

    let message = if errors.is_empty() {
        format!("Deleted {processed} object(s) successfully.")
    } else {
        format!(
            "Deleted {processed} object(s), {failed} failed.\n{}",
            errors.join("\n")
        )
    };

    Ok(S3OperationResult {
        success: failed == 0,
        message,
        processed,
        failed,
    })
}

pub async fn create_folder(
    prefix: String,
) -> AppResult<S3OperationResult> {
    let (client, bucket) = get_or_build_client()?;

    let folder_key = if prefix.ends_with('/') {
        prefix
    } else {
        format!("{prefix}/")
    };

    client
        .put_object()
        .bucket(&bucket)
        .key(&folder_key)
        .body(Vec::new().into())
        .send()
        .await
        .map_err(|e| s3_error("Failed to create folder", e))?;

    Ok(S3OperationResult {
        success: true,
        message: format!("Created folder '{folder_key}' successfully."),
        processed: 1,
        failed: 0,
    })
}

pub async fn list_upload_storages() -> AppResult<Vec<AwsStorage>> {
    crate::database::aws_storage_store::list_by_upload().await
}

pub async fn scan_upload_folder(dir_path: String) -> AppResult<Vec<ScannedFile>> {
    let root = Path::new(&dir_path);
    if !root.is_dir() {
        return Err(AppError::new(format!("Not a directory: {dir_path}")));
    }

    let mut files = Vec::new();
    let entries = std::fs::read_dir(root)
        .map_err(|e| AppError::new(format!("Failed to read directory: {e}")))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let parent_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            collect_bug_folder_files(&path, &parent_name, "", &mut files)?;
        }
    }

    Ok(files)
}

/// Quét đệ quy toàn bộ file bên trong một thư mục phiếu bug, kể cả khi phiếu bug có
/// subfolder (vd. bản backup/nháp) — `parent_name` luôn giữ nguyên là tên thư mục phiếu
/// bug gốc, không phải tên subfolder, để logic lọc same-name/bug-pattern ở
/// `scan_upload_folders` không bị ảnh hưởng bởi độ sâu thư mục. `rel_dir` theo dõi
/// đường dẫn subfolder tương đối so với gốc phiếu bug (rỗng ở cấp gốc) để giữ nguyên
/// cấu trúc thư mục con khi tải lên S3.
fn collect_bug_folder_files(
    dir: &Path,
    parent_name: &str,
    rel_dir: &str,
    files: &mut Vec<ScannedFile>,
) -> AppResult<()> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| AppError::new(format!("Failed to read sub-directory: {e}")))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let sub_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let next_rel_dir = if rel_dir.is_empty() {
                sub_name
            } else {
                format!("{rel_dir}/{sub_name}")
            };
            collect_bug_folder_files(&path, parent_name, &next_rel_dir, files)?;
        } else if path.is_file() {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
            let full_path = path.to_string_lossy().to_string();
            files.push(ScannedFile {
                parent_name: parent_name.to_string(),
                name,
                file_path: full_path.clone(),
                full_path,
                file_size,
                sub_path: rel_dir.to_string(),
            });
        }
    }

    Ok(())
}

pub async fn scan_upload_folders(dir_paths: Vec<String>) -> AppResult<Vec<ScannedFile>> {
    use regex::Regex;

    let bug_pattern = Regex::new(
        r"^F3\.1_バグ管理表_[A-Za-z]*\d+"
    ).unwrap();

    let paths = &dir_paths;
    if paths.is_empty() {
        return Ok(vec![]);
    }

    let raw_files = if paths.len() == 1 {
        let single_folder_name = Path::new(&paths[0])
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if bug_pattern.is_match(&single_folder_name) {
            let parent_path = Path::new(&paths[0])
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let files = scan_upload_folder(parent_path).await?;
            files.into_iter()
                .filter(|f| f.parent_name == single_folder_name)
                .collect::<Vec<_>>()
        } else {
            let files = scan_upload_folder(paths[0].clone()).await?;
            files.into_iter()
                .filter(|f| bug_pattern.is_match(&f.parent_name))
                .collect::<Vec<_>>()
        }
    } else {
        let folder_names: Vec<String> = paths.iter().map(|p| {
            Path::new(p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        }).collect();

        let invalid: Vec<&String> = folder_names.iter()
            .filter(|name| !bug_pattern.is_match(name))
            .collect();
        if !invalid.is_empty() {
            let names = invalid.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            return Err(AppError::new(
                format!("Thư mục không đúng định dạng F3.1_バグ管理表_XXXX: {names}")
            ));
        }

        let parent_path = Path::new(&paths[0])
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let files = scan_upload_folder(parent_path).await?;
        let selected: std::collections::HashSet<&str> = folder_names.iter().map(|s| s.as_str()).collect();
        files.into_iter()
            .filter(|f| selected.contains(f.parent_name.as_str()))
            .collect::<Vec<_>>()
    };

    let mut grouped: std::collections::HashMap<String, Vec<ScannedFile>> = std::collections::HashMap::new();
    for f in raw_files {
        grouped.entry(f.parent_name.clone()).or_default().push(f);
    }

    let mut result = Vec::new();
    for (folder_name, folder_files) in &grouped {
        // The same-name/bug-pattern naming convention only makes sense for files that
        // sit directly inside the bug folder. Files nested in a subfolder (e.g.
        // `OUTPUT/`) are deliberate work artifacts placed there by the user and don't
        // follow that naming convention — always include them.
        let (direct, nested): (Vec<&ScannedFile>, Vec<&ScannedFile>) =
            folder_files.iter().partition(|f| f.sub_path.is_empty());
        result.extend(nested.into_iter().cloned());

        let same_name: Vec<&ScannedFile> = direct.iter()
            .filter(|f| file_stem(&f.name).starts_with(folder_name.as_str()))
            .cloned()
            .collect();
        if !same_name.is_empty() {
            result.extend(same_name.into_iter().cloned());
        } else {
            let pattern_match: Vec<&ScannedFile> = direct.iter()
                .filter(|f| bug_pattern.is_match(&file_stem(&f.name)))
                .cloned()
                .collect();
            result.extend(pattern_match.into_iter().cloned());
        }
    }

    Ok(result)
}

fn file_stem(file_name: &str) -> String {
    match file_name.rfind('.') {
        Some(pos) if pos > 0 => file_name[..pos].to_string(),
        _ => file_name.to_string(),
    }
}

/// Cache for `get_work_folder` — the work-folder name rarely (if ever) changes
/// once configured, so avoid a DB round-trip (connect + stored procedure call)
/// on every single S3 command. Cached for the lifetime of the process; restart
/// the app to pick up a changed mapping.
static WORK_FOLDER_CACHE: OnceLock<StdRwLock<HashMap<String, String>>> = OnceLock::new();

pub async fn get_work_folder(folder_key: &str) -> AppResult<String> {
    let cache = WORK_FOLDER_CACHE.get_or_init(|| StdRwLock::new(HashMap::new()));

    if let Some(name) = cache.read().unwrap().get(folder_key) {
        return Ok(name.clone());
    }

    let name = crate::database::aws_storage_store::get_work_folder_name(folder_key).await?;
    cache.write().unwrap().insert(folder_key.to_string(), name.clone());
    Ok(name)
}

pub async fn upload_files(
    files: Vec<UploadFileRequest>,
    storage_name: String,
    subscribe: String,
    create_folder_same_name: bool,
    aws_cd: String,
    user_id: String,
) -> AppResult<S3OperationResult> {
    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;

    let (client, bucket) = get_or_build_client()?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    // {work_folder}/{storage_name}/{subscribe}/{parent_name}/{file_name}
    let base_prefix = format!("{}/{}/{}", work_folder, storage_name, subscribe);

    let mut targets: Vec<(String, String, String)> = Vec::new(); // (local_path, file_name, s3_key)
    for file in &files {
        // Preserve the local subfolder structure under the bug folder on S3
        // (e.g. a local `OUTPUT/` subfolder becomes `.../{parent_name}/OUTPUT/...`).
        let parent_prefix = if file.sub_path.is_empty() {
            file.parent_name.clone()
        } else {
            format!("{}/{}", file.parent_name, file.sub_path)
        };
        let s3_key = if create_folder_same_name {
            let stem = Path::new(&file.name)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| file.name.clone());
            format!("{}/{}/{}/{}", base_prefix, parent_prefix, stem, file.name)
        } else {
            format!("{}/{}/{}", base_prefix, parent_prefix, file.name)
        };

        if !Path::new(&file.local_path).exists() {
            failed += 1;
            errors.push(format!("{}: file not found", file.name));
            continue;
        }
        targets.push((file.local_path.clone(), file.name.clone(), s3_key));
    }

    let results: Vec<Result<(), String>> =
        stream::iter(targets.into_iter().map(|(local_path, file_name, s3_key)| {
            let client = client.clone();
            let bucket = bucket.clone();
            async move {
                upload_one_file(&client, &bucket, Path::new(&local_path), &s3_key)
                    .await
                    .map_err(|e| format!("{file_name}: {e}"))
            }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    for r in results {
        match r {
            Ok(()) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(e);
            }
        }
    }

    if processed > 0 {
        let now = chrono::Local::now();
        let date_ymd = now.format("%Y%m%d").to_string();
        let time_hms = now.format("%H%M%S").to_string();

        let details: Vec<crate::database::upload_store::UploadFileDetail> = files
            .iter()
            .map(|f| crate::database::upload_store::UploadFileDetail {
                bug_no: f.parent_name.clone(),
                file_name: f.name.clone(),
                file_path: f.local_path.clone(),
            })
            .collect();

        if let Err(e) = crate::database::upload_store::insert_upload(
            &aws_cd,
            &date_ymd,
            &time_hms,
            &user_id,
            create_folder_same_name,
            &details,
        )
        .await
        {
            log::error!("Failed to save upload history: {e}");
        }
    }

    let message = if errors.is_empty() {
        format!("Uploaded {processed} file(s) successfully.")
    } else {
        format!(
            "Uploaded {processed} file(s), {failed} failed.\n{}",
            errors.join("\n")
        )
    };

    Ok(S3OperationResult {
        success: failed == 0,
        message,
        processed,
        failed,
    })
}

pub async fn search_upload_history(
    params: UploadHistorySearchParams,
) -> AppResult<Vec<UploadHistorySearchItem>> {
    crate::database::upload_store::search_upload_history(&params).await
}

pub async fn scan_local_folder(folder_path: String) -> AppResult<Vec<LocalFileEntry>> {
    let root = Path::new(&folder_path);
    if !root.is_dir() {
        return Err(AppError::new(format!("Not a directory: {folder_path}")));
    }

    let mut files = Vec::new();
    scan_dir_recursive(root, root, &mut files)?;
    Ok(files)
}

fn scan_dir_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<LocalFileEntry>,
) -> AppResult<()> {
    let entries = std::fs::read_dir(current)
        .map_err(|e| AppError::new(format!("Failed to read directory: {e}")))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(root, &path, files)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            files.push(LocalFileEntry {
                name,
                relative_path: relative,
                full_path: path.to_string_lossy().to_string(),
                size,
            });
        }
    }
    Ok(())
}

pub async fn upload_folder(
    folder_path: String,
    s3_prefix: String,
) -> AppResult<S3OperationResult> {
    let files = scan_local_folder(folder_path.clone()).await?;

    if files.is_empty() {
        return Ok(S3OperationResult {
            success: true,
            message: "No files to upload.".to_string(),
            processed: 0,
            failed: 0,
        });
    }

    let (client, bucket) = get_or_build_client()?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    let root = Path::new(&folder_path);
    let folder_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let targets: Vec<(String, String, String)> = files // (full_path, relative_path, s3_key)
        .iter()
        .map(|file| {
            let s3_key = format!("{}{}/{}", s3_prefix, folder_name, file.relative_path);
            (file.full_path.clone(), file.relative_path.clone(), s3_key)
        })
        .collect();

    let results: Vec<Result<(), String>> =
        stream::iter(targets.into_iter().map(|(full_path, relative_path, s3_key)| {
            let client = client.clone();
            let bucket = bucket.clone();
            async move {
                upload_one_file(&client, &bucket, Path::new(&full_path), &s3_key)
                    .await
                    .map_err(|e| format!("{relative_path}: {e}"))
            }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    for r in results {
        match r {
            Ok(()) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(e);
            }
        }
    }

    let message = if errors.is_empty() {
        format!("Uploaded {processed} file(s) successfully.")
    } else {
        format!(
            "Uploaded {processed} file(s), {failed} failed.\n{}",
            errors.join("\n")
        )
    };

    Ok(S3OperationResult {
        success: failed == 0,
        message,
        processed,
        failed,
    })
}

pub async fn get_browser_allowed_prefixes() -> AppResult<Vec<String>> {
    crate::database::aws_storage_store::list_browser_allowed_prefixes("CORRECT_BUG_TEST").await
}

pub async fn list_download_storages() -> AppResult<Vec<AwsStorage>> {
    crate::database::aws_storage_store::list_by_download().await
}

pub async fn check_download_available(codes: Vec<String>) -> AppResult<HashMap<String, DownloadAvailability>> {
    let storages = crate::database::aws_storage_store::list_by_codes(&codes).await?;
    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;
    let (client, bucket) = get_or_build_client()?;

    let entries: Vec<(String, bool)> = stream::iter(storages.into_iter().map(|storage| {
        let client = client.clone();
        let bucket = bucket.clone();
        let work_folder = work_folder.clone();
        async move {
            let prefix = format!("{}/{}/{}/", work_folder, storage.name, storage.subscribe);
            let has_items = client
                .list_objects_v2()
                .bucket(&bucket)
                .prefix(&prefix)
                .delimiter("/")
                .send()
                .await
                .map(|out| !out.common_prefixes().is_empty())
                .unwrap_or(false);
            (storage.code, has_items)
        }
    }))
    .buffer_unordered(S3_CONCURRENCY)
    .collect()
    .await;

    let mut result = HashMap::new();
    for (code, has_items) in entries {
        result.insert(code, DownloadAvailability { download_available: has_items });
    }
    Ok(result)
}

pub async fn get_download_list(code: String) -> AppResult<Vec<String>> {
    let storages = crate::database::aws_storage_store::list_by_codes(&[code]).await?;
    let storage = storages
        .first()
        .ok_or_else(|| AppError::new("Storage not found".to_string()))?;

    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;
    let prefix = format!("{}/{}/{}/", work_folder, storage.name, storage.subscribe);

    let (client, bucket) = get_or_build_client()?;

    let output = client
        .list_objects_v2()
        .bucket(&bucket)
        .prefix(&prefix)
        .delimiter("/")
        .send()
        .await
        .map_err(|e| s3_error("Failed to list download items", e))?;

    let items: Vec<String> = output
        .common_prefixes()
        .iter()
        .filter_map(|p| p.prefix())
        .filter_map(|p| {
            p.strip_prefix(&prefix)
                .map(|s| s.trim_end_matches('/').to_string())
        })
        .filter(|s| !s.is_empty())
        .collect();

    Ok(items)
}

pub async fn download_by_storage(
    code: String,
    bug_list: Vec<String>,
    local_path: String,
    user_id: String,
) -> AppResult<DownloadByStorageResult> {
    let storages = crate::database::aws_storage_store::list_by_codes(&[code.clone()]).await?;
    let storage = storages
        .first()
        .ok_or_else(|| AppError::new("Storage not found".to_string()))?;

    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;
    let base_prefix = format!("{}/{}/{}/", work_folder, storage.name, storage.subscribe);

    let (client, bucket) = get_or_build_client()?;

    let now = chrono::Local::now();
    let date_ymd = now.format("%Y%m%d").to_string();
    let time_hm = now.format("%H%M").to_string();
    let time_hms = now.format("%H%M%S").to_string();

    let dest = Path::new(&local_path)
        .join(&storage.name)
        .join(&date_ymd)
        .join(&time_hm);
    let sync_path = dest.to_string_lossy().to_string();

    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut download_details: Vec<crate::database::download_store::DownloadDetail> = Vec::new();

    // List every bug folder concurrently instead of one at a time.
    let list_results: Vec<(String, AppResult<Vec<String>>)> =
        stream::iter(bug_list.iter().cloned().map(|bug| {
            let client = client.clone();
            let bucket = bucket.clone();
            let base_prefix = base_prefix.clone();
            async move {
                let prefix = format!("{}{}/", base_prefix, bug);
                let r = list_all_objects_recursive(&client, &bucket, &prefix).await;
                (bug, r)
            }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    let mut targets: Vec<(String, String, std::path::PathBuf)> = Vec::new(); // (bug, key, local_file)
    for (bug, r) in list_results {
        match r {
            Ok(keys) => {
                for key in keys.into_iter().filter(|k| !k.ends_with('/')) {
                    let relative = key.strip_prefix(base_prefix.as_str()).unwrap_or(&key).to_string();
                    let local_file = dest.join(&relative);
                    targets.push((bug.clone(), key, local_file));
                }
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{bug}: list failed: {e}"));
            }
        }
    }

    let results: Vec<(String, std::path::PathBuf, Result<(), String>)> =
        stream::iter(targets.into_iter().map(|(bug, key, local_file)| {
            let client = client.clone();
            let bucket = bucket.clone();
            async move {
                let r = download_one_object(&client, &bucket, &key, &local_file).await;
                (bug, local_file, r)
            }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    for (bug, local_file, r) in results {
        match r {
            Ok(()) => {
                processed += 1;
                download_details.push(crate::database::download_store::DownloadDetail {
                    bug_no: bug,
                    sync_path: local_file.to_string_lossy().to_string(),
                });
            }
            Err(e) => {
                failed += 1;
                errors.push(e);
            }
        }
    }

    if !download_details.is_empty() {
        if let Err(e) = crate::database::download_store::insert_download(
            &code,
            &date_ymd,
            &time_hms,
            &user_id,
            &sync_path,
            &download_details,
        )
        .await
        {
            log::error!("Failed to save download history: {e}");
        }

        if let Err(e) = crate::services::s3_watch_service::mark_as_seen(&code, &bug_list) {
            log::error!("Failed to update seen.json after download: {e}");
        }
    }

    let message = if errors.is_empty() {
        format!("Tải về thành công {processed} tập tin.")
    } else {
        format!(
            "Tải về {processed} tập tin, {failed} thất bại.\n{}",
            errors.join("\n")
        )
    };

    Ok(DownloadByStorageResult {
        success: failed == 0,
        message,
        processed,
        failed,
        sync_path,
    })
}

pub async fn get_download_history(user_id: String) -> AppResult<Vec<DownloadHistoryItem>> {
    crate::database::download_store::get_download_history(&user_id).await
}

pub async fn search_download_history(
    params: DownloadHistorySearchParams,
) -> AppResult<Vec<DownloadHistorySearchItem>> {
    crate::database::download_store::search_download_history(&params).await
}

pub async fn update_download_moved_local(id: i32, path_copied: String) -> AppResult<()> {
    crate::database::download_store::update_moved_at_local(id, &path_copied).await
}

pub async fn move_s3_objects(
    code: String,
    items: Vec<String>,
) -> AppResult<S3OperationResult> {
    let storages = crate::database::aws_storage_store::list_by_codes(&[code.clone()]).await?;
    let storage = storages
        .first()
        .ok_or_else(|| AppError::new("Storage not found".to_string()))?;

    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;
    let source_prefix = format!("{}/{}/{}/", work_folder, storage.name, storage.subscribe);
    let target_prefix = format!("{}/{}/", work_folder, storage.name);

    let (client, bucket) = get_or_build_client()?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    // List every item's objects concurrently first, then run one combined
    // concurrent move batch across all items instead of moving item-by-item.
    let list_results: Vec<(String, String, AppResult<Vec<String>>)> =
        stream::iter(items.iter().cloned().map(|item| {
            let client = client.clone();
            let bucket = bucket.clone();
            let source_prefix = source_prefix.clone();
            async move {
                let item_prefix = format!("{}{}/", source_prefix, item);
                let r = list_all_objects_recursive(&client, &bucket, &item_prefix).await;
                (item, item_prefix, r)
            }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    let mut move_targets: Vec<(String, String)> = Vec::new();
    let mut item_prefixes: Vec<String> = Vec::new();
    for (item, item_prefix, r) in list_results {
        match r {
            Ok(keys) => {
                for key in &keys {
                    let relative = key.strip_prefix(source_prefix.as_str()).unwrap_or(key.as_str());
                    move_targets.push((key.clone(), format!("{}{}", target_prefix, relative)));
                }
                item_prefixes.push(item_prefix);
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{item}: list failed: {e}"));
            }
        }
    }

    let results: Vec<Result<(), String>> =
        stream::iter(move_targets.into_iter().map(|(key, target_key)| {
            let client = client.clone();
            let bucket = bucket.clone();
            async move { move_one_object(&client, &bucket, &key, &target_key).await }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    for r in results {
        match r {
            Ok(()) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(e);
            }
        }
    }

    // Best-effort cleanup of the now-empty folder marker objects.
    for item_prefix in &item_prefixes {
        let _ = client.delete_object().bucket(&bucket).key(item_prefix).send().await;
    }

    if processed > 0 {
        if let Err(e) = crate::database::download_store::update_moved_at_s3(&code, &items).await {
            log::error!("Failed to update is_moved_at_s3: {e}");
        }
    }

    let message = if errors.is_empty() {
        format!("Đã di chuyển thành công {processed} tập tin.")
    } else {
        format!(
            "Di chuyển {processed} tập tin, {failed} thất bại.\n{}",
            errors.join("\n")
        )
    };

    Ok(S3OperationResult {
        success: failed == 0,
        message,
        processed,
        failed,
    })
}

pub async fn delete_s3_objects_by_storage(
    code: String,
    items: Vec<String>,
) -> AppResult<S3OperationResult> {
    let storages = crate::database::aws_storage_store::list_by_codes(&[code]).await?;
    let storage = storages
        .first()
        .ok_or_else(|| AppError::new("Storage not found".to_string()))?;

    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;
    let base_prefix = format!("{}/{}/{}/", work_folder, storage.name, storage.subscribe);

    let (client, bucket) = get_or_build_client()?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    // List every item's objects concurrently before deleting.
    let list_results: Vec<(String, String, AppResult<Vec<String>>)> =
        stream::iter(items.iter().cloned().map(|item| {
            let client = client.clone();
            let bucket = bucket.clone();
            let base_prefix = base_prefix.clone();
            async move {
                let item_prefix = format!("{}{}/", base_prefix, item);
                let r = list_all_objects_recursive(&client, &bucket, &item_prefix).await;
                (item, item_prefix, r)
            }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    for (item, item_prefix, r) in list_results {
        match r {
            Ok(keys) => {
                let results: Vec<Result<(), String>> =
                    stream::iter(keys.iter().cloned().map(|key| {
                        let client = client.clone();
                        let bucket = bucket.clone();
                        async move { delete_one_object(&client, &bucket, &key).await }
                    }))
                    .buffer_unordered(S3_CONCURRENCY)
                    .collect()
                    .await;

                let mut item_failed = false;
                for r in results {
                    if let Err(e) = r {
                        failed += 1;
                        item_failed = true;
                        errors.push(e);
                    }
                }
                let _ = client.delete_object().bucket(&bucket).key(&item_prefix).send().await;
                if !item_failed {
                    processed += 1;
                }
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{item}: list failed: {e}"));
            }
        }
    }

    let message = if errors.is_empty() {
        format!("Đã xoá thành công {processed} thư mục.")
    } else {
        format!(
            "Đã xoá {processed} thư mục, {failed} thất bại.\n{}",
            errors.join("\n")
        )
    };

    Ok(S3OperationResult {
        success: failed == 0,
        message,
        processed,
        failed,
    })
}

pub async fn move_browser_objects(
    keys: Vec<String>,
    destination_prefix: String,
) -> AppResult<S3OperationResult> {
    let (client, bucket) = get_or_build_client()?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    // Gather every (source_key, target_key) pair up front — folder keys expand to
    // their children via a (cheap, metadata-only) list call, run concurrently
    // across all selected folders — then run all the actual copy+delete moves
    // concurrently instead of one at a time.
    let (folder_keys, file_keys): (Vec<String>, Vec<String>) =
        keys.into_iter().partition(|k| k.ends_with('/'));

    let mut move_targets: Vec<(String, String)> = file_keys
        .into_iter()
        .map(|key| {
            let file_name = key.rsplit('/').next().unwrap_or(key.as_str()).to_string();
            let target_key = format!("{}{}", destination_prefix, file_name);
            (key, target_key)
        })
        .collect();
    let mut folder_markers: Vec<String> = Vec::new();

    if !folder_keys.is_empty() {
        let list_results: Vec<(String, AppResult<Vec<String>>)> =
            stream::iter(folder_keys.into_iter().map(|key| {
                let client = client.clone();
                let bucket = bucket.clone();
                async move {
                    let r = list_all_objects_recursive(&client, &bucket, &key).await;
                    (key, r)
                }
            }))
            .buffer_unordered(S3_CONCURRENCY)
            .collect()
            .await;

        for (key, r) in list_results {
            match r {
                Ok(sub_keys) => {
                    let folder_name = key.trim_end_matches('/').rsplit('/').next().unwrap_or("");
                    for sub_key in sub_keys {
                        let relative = sub_key.strip_prefix(&key).unwrap_or(&sub_key).to_string();
                        let target_key = format!("{}{}/{}", destination_prefix, folder_name, relative);
                        move_targets.push((sub_key, target_key));
                    }
                    folder_markers.push(key);
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{key}: list failed: {e}"));
                }
            }
        }
    }

    let results: Vec<Result<(), String>> =
        stream::iter(move_targets.into_iter().map(|(key, target_key)| {
            let client = client.clone();
            let bucket = bucket.clone();
            async move { move_one_object(&client, &bucket, &key, &target_key).await }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    for r in results {
        match r {
            Ok(()) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(e);
            }
        }
    }

    // Best-effort cleanup of the now-empty folder marker objects.
    for marker in &folder_markers {
        let _ = client.delete_object().bucket(&bucket).key(marker).send().await;
    }

    let message = if errors.is_empty() {
        format!("Đã di chuyển thành công {processed} tập tin.")
    } else {
        format!(
            "Di chuyển {processed} tập tin, {failed} thất bại.\n{}",
            errors.join("\n")
        )
    };

    Ok(S3OperationResult {
        success: failed == 0,
        message,
        processed,
        failed,
    })
}

pub async fn list_all_bug_folders() -> AppResult<Vec<StorageBugFolders>> {
    let storages = crate::database::aws_storage_store::list_by_download().await?;
    if storages.is_empty() {
        return Ok(Vec::new());
    }

    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;
    let (client, bucket) = get_or_build_client()?;

    let results: Vec<StorageBugFolders> = stream::iter(storages.into_iter().map(|storage| {
        let client = client.clone();
        let bucket = bucket.clone();
        let work_folder = work_folder.clone();
        async move {
            let prefix = if storage.subscribe.is_empty() {
                format!("{}/{}/", work_folder, storage.name)
            } else {
                format!("{}/{}/{}/", work_folder, storage.name, storage.subscribe)
            };

            let bugs = match client
                .list_objects_v2()
                .bucket(&bucket)
                .prefix(&prefix)
                .delimiter("/")
                .send()
                .await
            {
                Ok(output) => output
                    .common_prefixes()
                    .iter()
                    .filter_map(|p| p.prefix())
                    .filter_map(|p| {
                        p.strip_prefix(&prefix)
                            .map(|s| s.trim_end_matches('/').to_string())
                    })
                    .filter(|s| !s.is_empty())
                    .collect(),
                Err(_) => Vec::new(),
            };

            StorageBugFolders { storage, bugs }
        }
    }))
    .buffer_unordered(S3_CONCURRENCY)
    .collect()
    .await;

    Ok(results)
}

pub async fn list_bug_folder_tabs() -> AppResult<Vec<BugFolderTab>> {
    let storages = crate::database::aws_storage_store::list_all().await?;
    if storages.is_empty() {
        return Ok(Vec::new());
    }

    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;
    let (client, bucket) = get_or_build_client()?;

    struct GroupEntry {
        name: String,
        name_alias: String,
        subscribes: Vec<String>,
        excludes: std::collections::HashSet<String>,
    }

    let mut groups: Vec<GroupEntry> = Vec::new();
    for s in &storages {
        let entry = groups.iter_mut().find(|g| g.name == s.name);
        if let Some(entry) = entry {
            if !s.subscribe.is_empty() {
                entry.subscribes.push(s.subscribe.clone());
            }
            for ex in &s.exclude_subscribe {
                entry.excludes.insert(ex.clone());
            }
        } else {
            let mut excludes = std::collections::HashSet::new();
            for ex in &s.exclude_subscribe {
                excludes.insert(ex.clone());
            }
            let subscribes = if s.subscribe.is_empty() {
                Vec::new()
            } else {
                vec![s.subscribe.clone()]
            };
            groups.push(GroupEntry {
                name: s.name.clone(),
                name_alias: s.name_alias.clone(),
                subscribes,
                excludes,
            });
        }
    }

    // Flatten every parent/subscribe listing across all groups into one task list
    // and run them all concurrently, instead of awaiting each group and each
    // subscribe within it one at a time.
    let mut list_tasks: Vec<(usize, Option<String>, String)> = Vec::new(); // (group_idx, subscribe, prefix)
    for (idx, g) in groups.iter().enumerate() {
        list_tasks.push((idx, None, format!("{}/{}/", work_folder, g.name)));
        for sub in &g.subscribes {
            list_tasks.push((idx, Some(sub.clone()), format!("{}/{}/{}/", work_folder, g.name, sub)));
        }
    }

    let task_results: Vec<(usize, Option<String>, HashMap<String, String>)> =
        stream::iter(list_tasks.into_iter().map(|(idx, sub, prefix)| {
            let client = client.clone();
            let bucket = bucket.clone();
            async move {
                let dates = list_subfolders_with_dates(&client, &bucket, &prefix).await;
                (idx, sub, dates)
            }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    let mut parent_dates_by_group: HashMap<usize, HashMap<String, String>> = HashMap::new();
    let mut sub_dates_by_group: HashMap<(usize, String), HashMap<String, String>> = HashMap::new();
    for (idx, sub, dates) in task_results {
        match sub {
            None => {
                parent_dates_by_group.insert(idx, dates);
            }
            Some(sub) => {
                sub_dates_by_group.insert((idx, sub), dates);
            }
        }
    }

    let mut tabs = Vec::new();
    for (idx, GroupEntry { name, name_alias, subscribes, excludes }) in groups.iter().enumerate() {
        let mut items: Vec<BugFolderItem> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

        let skip_set: std::collections::HashSet<&str> = subscribes
            .iter()
            .map(|s| s.as_str())
            .chain(excludes.iter().map(|s| s.as_str()))
            .collect();

        if let Some(parent_dates) = parent_dates_by_group.get(&idx) {
            for (folder, date) in parent_dates {
                if !skip_set.contains(folder.as_str()) && seen.insert(folder.clone()) {
                    items.push(BugFolderItem {
                        bug_no: folder.clone(),
                        in_subscribe: false,
                        last_modified: date.clone(),
                    });
                }
            }
        }

        for sub in subscribes {
            if let Some(sub_dates) = sub_dates_by_group.get(&(idx, sub.clone())) {
                for (folder, date) in sub_dates {
                    if seen.insert(folder.clone()) {
                        items.push(BugFolderItem {
                            bug_no: folder.clone(),
                            in_subscribe: true,
                            last_modified: date.clone(),
                        });
                    }
                }
            }
        }

        items.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        tabs.push(BugFolderTab {
            name: name.clone(),
            name_alias: if name_alias.is_empty() { name.clone() } else { name_alias.clone() },
            items,
        });
    }

    Ok(tabs)
}

pub async fn list_delete_options(destination_code: String) -> AppResult<Vec<AwsStorage>> {
    crate::database::aws_storage_store::list_delete_options(&destination_code).await
}

pub async fn delete_uploaded_items(
    items: Vec<DeleteUploadedItem>,
) -> AppResult<S3OperationResult> {
    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;
    let all_codes: Vec<String> = items.iter().map(|i| i.aws_cd.clone()).collect();
    let storages = crate::database::aws_storage_store::list_by_codes(&all_codes).await?;

    let (client, bucket) = get_or_build_client()?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    // Resolve each item's prefix up front (local lookup, no I/O), then list every
    // item's objects concurrently before deleting.
    let mut list_targets: Vec<(String, String)> = Vec::new(); // (bug_no, prefix)
    for item in &items {
        match storages.iter().find(|s| s.code == item.aws_cd) {
            Some(storage) => {
                let prefix = format!("{}/{}/{}/", work_folder, storage.name, item.bug_no);
                list_targets.push((item.bug_no.clone(), prefix));
            }
            None => {
                failed += 1;
                errors.push(format!("{}: storage code '{}' not found", item.bug_no, item.aws_cd));
            }
        }
    }

    let list_results: Vec<(String, String, AppResult<Vec<String>>)> =
        stream::iter(list_targets.into_iter().map(|(bug_no, prefix)| {
            let client = client.clone();
            let bucket = bucket.clone();
            async move {
                let r = list_all_objects_recursive(&client, &bucket, &prefix).await;
                (bug_no, prefix, r)
            }
        }))
        .buffer_unordered(S3_CONCURRENCY)
        .collect()
        .await;

    for (bug_no, prefix, r) in list_results {
        match r {
            Ok(keys) => {
                if keys.is_empty() {
                    processed += 1;
                    continue;
                }
                let results: Vec<Result<(), String>> =
                    stream::iter(keys.iter().cloned().map(|key| {
                        let client = client.clone();
                        let bucket = bucket.clone();
                        async move { delete_one_object(&client, &bucket, &key).await }
                    }))
                    .buffer_unordered(S3_CONCURRENCY)
                    .collect()
                    .await;

                let mut item_failed = false;
                for r in results {
                    if let Err(e) = r {
                        failed += 1;
                        item_failed = true;
                        errors.push(e);
                    }
                }
                let _ = client.delete_object().bucket(&bucket).key(&prefix).send().await;
                if !item_failed {
                    processed += 1;
                }
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{bug_no}: list failed: {e}"));
            }
        }
    }

    let message = if errors.is_empty() {
        format!("Đã thực hiện xoá thành công {processed} thư mục.")
    } else {
        format!(
            "Đã xoá {processed} thư mục, {failed} thất bại.\n{}",
            errors.join("\n")
        )
    };

    Ok(S3OperationResult {
        success: failed == 0,
        message,
        processed,
        failed,
    })
}
