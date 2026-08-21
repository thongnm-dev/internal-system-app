//! Tầng lưu trữ cục bộ cho từ điển replace (vn_text → jp_text) của công cụ đồng bộ VN → JP.
//!
//! Mỗi lượt "Áp dụng" (`vnjp_sync_service::apply_changes`) thu thập thêm các cặp vn_text →
//! jp_text từ những ô mà output đã giữ nội dung JP thay vì dùng VN (xem
//! `sync_service::build_replace_dictionary`), rồi gộp vào file JSON `vnjp_dictionary.json`
//! trong thư mục `data` cùng cấp với thư mục `config` ([`app_config::local_data_dir`]) — để
//! tái sử dụng cho các tài liệu khác (khác doc type, khác lần chạy).

use std::collections::HashMap;
use std::path::PathBuf;

use crate::app::result::AppResult;
use crate::utils::app_config;

const DATA_FILE: &str = "vnjp_dictionary.json";

fn data_path() -> PathBuf {
    app_config::local_data_dir().join(DATA_FILE)
}

/// Đọc từ điển từ file. File chưa tồn tại → trả về rỗng.
pub fn load() -> AppResult<HashMap<String, String>> {
    let path = data_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

/// Ghi từ điển xuống file (pretty JSON, ghi đè).
pub fn save(data: &HashMap<String, String>) -> AppResult<()> {
    let path = data_path();
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Gộp các cặp mới vào từ điển đã lưu — giữ nguyên giá trị cũ nếu key đã tồn tại (từ điển tích
/// luỹ qua nhiều lượt "Áp dụng", không cho lượt sau ghi đè bản dịch đã có).
pub fn merge(new_entries: &HashMap<String, String>) -> AppResult<()> {
    if new_entries.is_empty() {
        return Ok(());
    }
    let mut dict = load()?;
    for (vn_text, jp_text) in new_entries {
        dict.entry(vn_text.clone()).or_insert_with(|| jp_text.clone());
    }
    save(&dict)
}
