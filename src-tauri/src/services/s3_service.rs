use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::s3::{AwsStorage, DeleteUploadedItem, DownloadAvailability, DownloadByStorageResult, DownloadHistoryItem, DownloadHistorySearchItem, DownloadHistorySearchParams, LocalFileEntry, S3Config, S3ListResult, S3Object, S3OperationResult, ScannedFile, UploadFileRequest, UploadHistorySearchItem, UploadHistorySearchParams};
use crate::utils::app_config;

use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use ini::Ini;
use std::collections::HashMap;
use std::path::Path;

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
        .behavior_version_latest();

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
    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
    client
        .head_bucket()
        .bucket(&bucket)
        .send()
        .await
        .map_err(|e| AppError::new(format!("Connection failed: {e}")))?;
    Ok(format!("Connected to bucket '{bucket}' successfully."))
}

pub async fn list_objects(prefix: String) -> AppResult<S3ListResult> {
    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;

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
        .map_err(|e| AppError::new(format!("Failed to list objects: {e}")))?;

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
    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
    let dest = Path::new(&destination_dir);
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    let mut all_keys = keys.clone();

    // Expand folder prefixes to individual object keys
    let mut i = 0;
    while i < all_keys.len() {
        if all_keys[i].ends_with('/') {
            let folder_prefix = all_keys.remove(i);
            match list_all_objects_recursive(&client, &bucket, &folder_prefix).await {
                Ok(child_keys) => {
                    for k in child_keys.into_iter().rev() {
                        all_keys.insert(i, k);
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{folder_prefix}: {e}"));
                }
            }
        } else {
            i += 1;
        }
    }

    for key in &all_keys {
        let relative = key.strip_prefix(strip_prefix.as_str()).unwrap_or(key.as_str());
        let local_path = dest.join(relative);

        if let Some(parent) = local_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                failed += 1;
                errors.push(format!("{key}: failed to create directory: {e}"));
                continue;
            }
        }

        match client
            .get_object()
            .bucket(&bucket)
            .key(key)
            .send()
            .await
        {
            Ok(output) => {
                match output.body.collect().await {
                    Ok(data) => {
                        let bytes = data.into_bytes();
                        if let Err(e) = tokio::fs::write(&local_path, &bytes).await {
                            failed += 1;
                            errors.push(format!("{key}: write failed: {e}"));
                        } else {
                            processed += 1;
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{key}: read stream failed: {e}"));
                    }
                }
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{key}: download failed: {e}"));
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
            .map_err(|e| AppError::new(format!("Failed to list objects under '{prefix}': {e}")))?;

        for obj in output.contents() {
            if let Some(key) = obj.key() {
                if !key.ends_with('/') {
                    keys.push(key.to_string());
                }
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

pub async fn upload_file(
    local_path: String,
    s3_key: String,
) -> AppResult<S3OperationResult> {
    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
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
        .map_err(|e| AppError::new(format!("Upload failed: {e}")))?;

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
    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    // Expand folder prefixes
    let mut all_keys: Vec<String> = Vec::new();
    for key in &keys {
        if key.ends_with('/') {
            match list_all_objects_recursive(&client, &bucket, key).await {
                Ok(child_keys) => {
                    all_keys.extend(child_keys);
                    all_keys.push(key.clone());
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{key}: {e}"));
                }
            }
        } else {
            all_keys.push(key.clone());
        }
    }

    for key in &all_keys {
        match client
            .delete_object()
            .bucket(&bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("{key}: {e}"));
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
    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;

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
        .map_err(|e| AppError::new(format!("Failed to create folder: {e}")))?;

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
            let sub_entries = std::fs::read_dir(&path)
                .map_err(|e| AppError::new(format!("Failed to read sub-directory: {e}")))?;
            for sub_entry in sub_entries.flatten() {
                let sub_path = sub_entry.path();
                if sub_path.is_file() {
                    let name = sub_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let file_size = sub_path.metadata().map(|m| m.len()).unwrap_or(0);
                    let full_path = sub_path.to_string_lossy().to_string();
                    files.push(ScannedFile {
                        parent_name: parent_name.clone(),
                        name,
                        file_path: full_path.clone(),
                        full_path,
                        file_size,
                    });
                }
            }
        }
    }

    Ok(files)
}

pub async fn scan_upload_folders(dir_paths: Vec<String>) -> AppResult<Vec<ScannedFile>> {
    use regex::Regex;

    let bug_pattern = Regex::new(
        r"^F3\.1_バグ管理表_\d{4}(?:（再）（急）|（特急）|（急）|（再）)?$"
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
        let same_name: Vec<&ScannedFile> = folder_files.iter()
            .filter(|f| file_stem(&f.name) == *folder_name)
            .collect();
        if !same_name.is_empty() {
            result.extend(same_name.into_iter().cloned());
        } else {
            let pattern_match: Vec<&ScannedFile> = folder_files.iter()
                .filter(|f| bug_pattern.is_match(&file_stem(&f.name)))
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

pub async fn get_work_folder(folder_key: &str) -> AppResult<String> {
    crate::database::aws_storage_store::get_work_folder_name(folder_key).await
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

    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    // {work_folder}/{storage_name}/{subscribe}/{parent_name}/{file_name}
    let base_prefix = format!("{}/{}/{}", work_folder, storage_name, subscribe);

    for file in &files {
        let s3_key = if create_folder_same_name {
            let stem = Path::new(&file.name)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| file.name.clone());
            format!("{}/{}/{}/{}", base_prefix, file.parent_name, stem, file.name)
        } else {
            format!("{}/{}/{}", base_prefix, file.parent_name, file.name)
        };
        let path = Path::new(&file.local_path);
        if !path.exists() {
            failed += 1;
            errors.push(format!("{}: file not found", file.name));
            continue;
        }

        match tokio::fs::read(path).await {
            Ok(body) => {
                match client
                    .put_object()
                    .bucket(&bucket)
                    .key(&s3_key)
                    .body(body.into())
                    .send()
                    .await
                {
                    Ok(_) => processed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: upload failed: {e}", file.name));
                    }
                }
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: read failed: {e}", file.name));
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

    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    let root = Path::new(&folder_path);
    let folder_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    for file in &files {
        let s3_key = format!("{}{}/{}", s3_prefix, folder_name, file.relative_path);
        let path = Path::new(&file.full_path);

        match tokio::fs::read(path).await {
            Ok(body) => {
                match client
                    .put_object()
                    .bucket(&bucket)
                    .key(&s3_key)
                    .body(body.into())
                    .send()
                    .await
                {
                    Ok(_) => processed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: upload failed: {e}", file.relative_path));
                    }
                }
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: read failed: {e}", file.relative_path));
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
    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;

    let mut result = HashMap::new();
    for storage in &storages {
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
        result.insert(
            storage.code.clone(),
            DownloadAvailability {
                download_available: has_items,
            },
        );
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

    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;

    let output = client
        .list_objects_v2()
        .bucket(&bucket)
        .prefix(&prefix)
        .delimiter("/")
        .send()
        .await
        .map_err(|e| AppError::new(format!("Failed to list download items: {e}")))?;

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

    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;

    let now = chrono::Local::now();
    let date_ymd = now.format("%Y%m%d").to_string();
    let time_hm = now.format("%H%M").to_string();
    let time_hms = now.format("%H%M%S").to_string();

    let sync_path = format!(
        "{}/{}/{}/{}",
        local_path.trim_end_matches(['/', '\\']),
        storage.name,
        date_ymd,
        time_hm
    );
    let dest = Path::new(&sync_path);

    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut download_details: Vec<crate::database::download_store::DownloadDetail> = Vec::new();

    for bug in &bug_list {
        let prefix = format!("{}{}/", base_prefix, bug);
        match list_all_objects_recursive(&client, &bucket, &prefix).await {
            Ok(keys) => {
                for key in &keys {
                    let relative = key.strip_prefix(base_prefix.as_str()).unwrap_or(key.as_str());
                    let local_file = dest.join(relative);

                    if let Some(parent) = local_file.parent() {
                        if let Err(e) = tokio::fs::create_dir_all(parent).await {
                            failed += 1;
                            errors.push(format!("{key}: failed to create directory: {e}"));
                            continue;
                        }
                    }

                    match client
                        .get_object()
                        .bucket(&bucket)
                        .key(key)
                        .send()
                        .await
                    {
                        Ok(output) => match output.body.collect().await {
                            Ok(data) => {
                                if let Err(e) = tokio::fs::write(&local_file, &data.into_bytes()).await {
                                    failed += 1;
                                    errors.push(format!("{key}: write failed: {e}"));
                                } else {
                                    processed += 1;
                                    download_details.push(
                                        crate::database::download_store::DownloadDetail {
                                            bug_no: bug.clone(),
                                            sync_path: local_file.to_string_lossy().to_string(),
                                        },
                                    );
                                }
                            }
                            Err(e) => {
                                failed += 1;
                                errors.push(format!("{key}: read stream failed: {e}"));
                            }
                        },
                        Err(e) => {
                            failed += 1;
                            errors.push(format!("{key}: download failed: {e}"));
                        }
                    }
                }
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{bug}: list failed: {e}"));
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

    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    for item in &items {
        let item_prefix = format!("{}{}/", source_prefix, item);
        match list_all_objects_recursive(&client, &bucket, &item_prefix).await {
            Ok(keys) => {
                for key in &keys {
                    let relative = key.strip_prefix(source_prefix.as_str()).unwrap_or(key.as_str());
                    let target_key = format!("{}{}", target_prefix, relative);
                    let encoded_key = key
                        .split('/')
                        .map(|seg| urlencoding::encode(seg).into_owned())
                        .collect::<Vec<_>>()
                        .join("/");
                    let copy_source = format!("{}/{}", bucket, encoded_key);

                    match client
                        .copy_object()
                        .bucket(&bucket)
                        .copy_source(&copy_source)
                        .key(&target_key)
                        .send()
                        .await
                    {
                        Ok(_) => {
                            if let Err(e) = client.delete_object().bucket(&bucket).key(key).send().await {
                                failed += 1;
                                errors.push(format!("{key}: delete after copy failed: {e}"));
                            } else {
                                processed += 1;
                            }
                        }
                        Err(e) => {
                            failed += 1;
                            errors.push(format!("{key}: copy failed: {e}"));
                        }
                    }
                }
                let _ = client.delete_object().bucket(&bucket).key(&item_prefix).send().await;
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{item}: list failed: {e}"));
            }
        }
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

    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    for item in &items {
        let item_prefix = format!("{}{}/", base_prefix, item);
        match list_all_objects_recursive(&client, &bucket, &item_prefix).await {
            Ok(keys) => {
                let mut item_failed = false;
                for key in &keys {
                    if let Err(e) = client.delete_object().bucket(&bucket).key(key).send().await {
                        failed += 1;
                        item_failed = true;
                        errors.push(format!("{key}: {e}"));
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
    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    for key in &keys {
        if key.ends_with('/') {
            let folder_name = key.trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("");
            match list_all_objects_recursive(&client, &bucket, key).await {
                Ok(sub_keys) => {
                    for sub_key in &sub_keys {
                        let relative = sub_key.strip_prefix(key).unwrap_or(sub_key);
                        let target_key = format!("{}{}/{}", destination_prefix, folder_name, relative);
                        let encoded_key = sub_key
                            .split('/')
                            .map(|seg| urlencoding::encode(seg).into_owned())
                            .collect::<Vec<_>>()
                            .join("/");
                        let copy_source = format!("{}/{}", bucket, encoded_key);
                        match client
                            .copy_object()
                            .bucket(&bucket)
                            .copy_source(&copy_source)
                            .key(&target_key)
                            .send()
                            .await
                        {
                            Ok(_) => {
                                if let Err(e) = client.delete_object().bucket(&bucket).key(sub_key).send().await {
                                    failed += 1;
                                    errors.push(format!("{sub_key}: delete after copy failed: {e}"));
                                } else {
                                    processed += 1;
                                }
                            }
                            Err(e) => {
                                failed += 1;
                                errors.push(format!("{sub_key}: copy failed: {e}"));
                            }
                        }
                    }
                    let _ = client.delete_object().bucket(&bucket).key(key).send().await;
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{key}: list failed: {e}"));
                }
            }
        } else {
            let file_name = key.rsplit('/').next().unwrap_or(key);
            let target_key = format!("{}{}", destination_prefix, file_name);
            let encoded_key = key
                .split('/')
                .map(|seg| urlencoding::encode(seg).into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let copy_source = format!("{}/{}", bucket, encoded_key);
            match client
                .copy_object()
                .bucket(&bucket)
                .copy_source(&copy_source)
                .key(&target_key)
                .send()
                .await
            {
                Ok(_) => {
                    if let Err(e) = client.delete_object().bucket(&bucket).key(key).send().await {
                        failed += 1;
                        errors.push(format!("{key}: delete after copy failed: {e}"));
                    } else {
                        processed += 1;
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{key}: copy failed: {e}"));
                }
            }
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

pub async fn list_delete_options(destination_code: String) -> AppResult<Vec<AwsStorage>> {
    crate::database::aws_storage_store::list_delete_options(&destination_code).await
}

pub async fn delete_uploaded_items(
    items: Vec<DeleteUploadedItem>,
) -> AppResult<S3OperationResult> {
    let work_folder = get_work_folder("CORRECT_BUG_TEST").await?;
    let all_codes: Vec<String> = items.iter().map(|i| i.aws_cd.clone()).collect();
    let storages = crate::database::aws_storage_store::list_by_codes(&all_codes).await?;

    let config = load_config_from_ini()?;
    let (client, bucket) = build_client(&config)?;
    let mut processed: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    for item in &items {
        let storage = match storages.iter().find(|s| s.code == item.aws_cd) {
            Some(s) => s,
            None => {
                failed += 1;
                errors.push(format!("{}: storage code '{}' not found", item.bug_no, item.aws_cd));
                continue;
            }
        };

        let prefix = format!(
            "{}/{}/{}/",
            work_folder, storage.name, item.bug_no
        );
        match list_all_objects_recursive(&client, &bucket, &prefix).await {
            Ok(keys) => {
                if keys.is_empty() {
                    processed += 1;
                    continue;
                }
                let mut item_failed = false;
                for key in &keys {
                    if let Err(e) = client.delete_object().bucket(&bucket).key(key).send().await {
                        failed += 1;
                        item_failed = true;
                        errors.push(format!("{}: {e}", key));
                    }
                }
                let _ = client.delete_object().bucket(&bucket).key(&prefix).send().await;
                if !item_failed {
                    processed += 1;
                }
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: list failed: {e}", item.bug_no));
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
