//! Tauri command handlers cho công cụ đồng bộ tài liệu VN → JP.

use crate::app::error::log_err;
use crate::models::vnjp_sync::{ApplyResult, SyncAnalysis, TranslateBatchRequest, TranslateItemResult};
use crate::services::vnjp_sync_service;

/// Phân tích sự khác biệt giữa file Excel VN và JP.
/// Trả về SyncAnalysis đầy đủ bao gồm ô đỏ, ô strikethrough và vấn đề chất lượng.
#[tauri::command]
pub fn vnjp_sync_analyze(vn_path: String, jp_path: String) -> Result<SyncAnalysis, String> {
    vnjp_sync_service::analyze(&vn_path, &jp_path).map_err(log_err)
}

/// Dịch hàng loạt các đoạn văn VN → JP qua AI (Gemini hoặc Groq).
#[tauri::command]
pub async fn vnjp_sync_translate(
    request: TranslateBatchRequest,
) -> Result<Vec<TranslateItemResult>, String> {
    vnjp_sync_service::translate_batch(request)
        .await
        .map_err(log_err)
}

/// Xuất báo cáo phân tích ra file Excel (.xlsx).
#[tauri::command]
pub fn vnjp_sync_export_report(
    analysis: SyncAnalysis,
    output_path: String,
) -> Result<String, String> {
    vnjp_sync_service::export_report(&analysis, &output_path).map_err(log_err)
}

/// Áp dụng thay đổi từ VN → JP: ghi nội dung VN (giữ nguyên tiếng Việt, tô đỏ) vào
/// đúng vị trí ô tương ứng trong file JP, lưu ra output_path.
#[tauri::command]
pub fn vnjp_sync_apply(
    vn_path: String,
    jp_path: String,
    output_path: String,
) -> Result<ApplyResult, String> {
    vnjp_sync_service::apply_changes(&vn_path, &jp_path, &output_path).map_err(log_err)
}
