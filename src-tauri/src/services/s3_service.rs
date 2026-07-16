use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::s3::{S3Config, S3ListResult, S3Object, S3OperationResult};
use crate::utils::pgsql_connect;

use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use ini::Ini;
use std::path::Path;

pub(crate) fn load_config_from_ini() -> AppResult<S3Config> {
    let path = pgsql_connect::config_path();
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
        let relative = key.as_str();
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
