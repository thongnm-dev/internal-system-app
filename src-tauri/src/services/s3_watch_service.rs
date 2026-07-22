//! Theo dõi nền các storage S3 (download) và bắn **notification native** khi có
//! tài liệu mới xuất hiện.
//!
//! Cơ chế: chạy vòng lặp poll suốt vòng đời ứng dụng (giống
//! [`crate::services::ai_usage_service::run_poll_loop`]). Mỗi vòng:
//! 1. Liệt kê các storage download (`s3_service::list_download_storages`).
//! 2. Với mỗi storage, lấy danh sách tài liệu hiện tại (`s3_service::get_download_list`).
//! 3. So với danh sách "đã thấy" lưu ở `<data_dir>/s3_watch/seen.json`.
//! 4. Có tài liệu mới → bắn **1 notification gộp** + emit event `s3-new-documents`.
//!
//! Lần chạy đầu (chưa có file `seen.json`) chỉ ghi baseline, KHÔNG bắn notification
//! để tránh spam toàn bộ tài liệu cũ khi mở app lần đầu.
//!
//! Chu kỳ poll đọc từ `config.ini` — section `[Notification]`, key
//! `S3_POLL_INTERVAL_MINUTES` (mặc định 10 phút).

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use ini::Ini;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::app::result::AppResult;
use crate::services::s3_service;
use crate::utils::app_config;

/// Event bắn cho frontend khi phát hiện tài liệu S3 mới.
const NEW_DOCUMENTS_EVENT: &str = "s3-new-documents";
/// Chu kỳ poll mặc định (phút) khi không có cấu hình trong config.ini.
const DEFAULT_POLL_MINUTES: u64 = 10;
/// Chu kỳ chờ (phút) khi pre-check thất bại (S3/DB chưa sẵn sàng).
const RETRY_WAIT_MINUTES: u64 = 2;

/// Thư mục con lưu trạng thái theo dõi.
const WATCH_DIR: &str = "s3_watch";
/// File lưu danh sách tài liệu "đã thấy" theo từng storage code.
const SEEN_FILE: &str = "seen.json";

type SeenMap = HashMap<String, Vec<String>>;

/// Đường dẫn file `seen.json` trong app data dir.
fn seen_path() -> PathBuf {
    app_config::data_dir().join(WATCH_DIR).join(SEEN_FILE)
}

/// Đọc trạng thái đã thấy. Trả về `None` nếu file chưa tồn tại (lần chạy đầu).
fn load_seen() -> Option<SeenMap> {
    let content = std::fs::read_to_string(seen_path()).ok()?;
    serde_json::from_str(&content).ok()
}

/// Ghi trạng thái đã thấy (pretty JSON, ghi đè).
fn save_seen(map: &SeenMap) -> AppResult<()> {
    let path = seen_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(map)?)?;
    Ok(())
}

/// Đọc chu kỳ poll (phút) từ config.ini — tối thiểu 1 phút.
fn poll_interval_minutes() -> u64 {
    let minutes = Ini::load_from_file(app_config::config_path())
        .ok()
        .and_then(|ini| {
            ini.section(Some("Notification"))
                .and_then(|s| s.get("S3_POLL_INTERVAL_MINUTES").map(|v| v.to_string()))
        })
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_POLL_MINUTES);
    minutes.max(1)
}

/// Poll một lần: phát hiện tài liệu mới, bắn notification + emit event.
pub async fn poll_once(app: &AppHandle) -> AppResult<()> {
    let storages = s3_service::list_download_storages().await?;
    if storages.is_empty() {
        return Ok(());
    }

    let prev = load_seen();
    let baseline_mode = prev.is_none();
    let prev = prev.unwrap_or_default();

    let mut current: SeenMap = HashMap::new();
    // (tên hiển thị storage, danh sách tài liệu mới)
    let mut fresh_by_storage: Vec<(String, Vec<String>)> = Vec::new();

    for storage in &storages {
        let items = match s3_service::get_download_list(storage.code.clone()).await {
            Ok(v) => v,
            Err(e) => {
                // Lỗi liệt kê một storage → giữ nguyên baseline cũ (nếu có) để
                // không bắn lại nhầm ở vòng sau, rồi bỏ qua storage này.
                eprintln!("S3 watch: list '{}' failed: {e}", storage.code);
                if let Some(known) = prev.get(&storage.code) {
                    current.insert(storage.code.clone(), known.clone());
                }
                continue;
            }
        };

        if !baseline_mode {
            let known: HashSet<&String> = prev
                .get(&storage.code)
                .map(|v| v.iter().collect())
                .unwrap_or_default();
            let new_items: Vec<String> = items
                .iter()
                .filter(|k| !known.contains(*k))
                .cloned()
                .collect();
            if !new_items.is_empty() {
                let label = if storage.name_alias.trim().is_empty() {
                    storage.name.clone()
                } else {
                    storage.name_alias.clone()
                };
                fresh_by_storage.push((label, new_items));
            }
        }

        current.insert(storage.code.clone(), items);
    }

    save_seen(&current)?;

    if baseline_mode || fresh_by_storage.is_empty() {
        return Ok(());
    }

    // Gộp thành 1 notification.
    let total: usize = fresh_by_storage.iter().map(|(_, v)| v.len()).sum();
    let detail = fresh_by_storage
        .iter()
        .map(|(name, v)| format!("{}: {}", name, v.join(", ")))
        .collect::<Vec<_>>()
        .join(" · ");
    let body = format!("Có {total} tài liệu mới trên S3 — {detail}");

    let _ = app
        .notification()
        .builder()
        .title("Tài liệu S3 mới")
        .body(&body)
        .show();

    let payload = serde_json::json!({
        "total": total,
        "storages": fresh_by_storage
            .iter()
            .map(|(name, items)| serde_json::json!({ "name": name, "items": items }))
            .collect::<Vec<_>>(),
    });
    let _ = app.emit(NEW_DOCUMENTS_EVENT, payload);

    Ok(())
}

/// Kiểm tra S3 config + DB đã sẵn sàng chưa trước khi bắt đầu poll.
async fn is_ready() -> Result<(), String> {
    s3_service::check_config().map_err(|e| format!("S3 config: {e}"))?;
    crate::utils::pgsql_connect::connect()
        .await
        .map_err(|e| format!("Database: {e}"))?;
    Ok(())
}

/// Vòng lặp poll nền — chạy suốt vòng đời ứng dụng.
///
/// Trước khi bắt đầu, kiểm tra S3 config và DB. Nếu chưa sẵn sàng,
/// chờ [`RETRY_WAIT_MINUTES`] phút rồi thử lại cho đến khi OK.
/// Sau đó chạy baseline + poll theo chu kỳ cấu hình.
pub async fn run_poll_loop(app: AppHandle) {
    loop {
        match is_ready().await {
            Ok(()) => break,
            Err(reason) => {
                eprintln!("S3 watch: chưa sẵn sàng ({reason}), thử lại sau {RETRY_WAIT_MINUTES} phút");
                tokio::time::sleep(Duration::from_secs(RETRY_WAIT_MINUTES * 60)).await;
            }
        }
    }

    if let Err(e) = poll_once(&app).await {
        eprintln!("S3 watch initial poll error: {e}");
    }

    loop {
        let minutes = poll_interval_minutes();
        tokio::time::sleep(Duration::from_secs(minutes.saturating_mul(60))).await;
        if let Err(e) = poll_once(&app).await {
            eprintln!("S3 watch poll error: {e}");
        }
    }
}
