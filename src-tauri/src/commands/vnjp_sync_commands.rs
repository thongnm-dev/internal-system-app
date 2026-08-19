//! Tauri command handlers cho công cụ đồng bộ tài liệu VN → JP.

use crate::app::error::log_err;
use crate::models::vnjp_sync::{
    AnalyzeAndApplyResult, CleanupResult, ConfirmedInsert, RedCell, RedCellVerificationReport,
    RowAlignmentReport, RowInsertResult, SyncAnalysis, TranslateBatchRequest, TranslateItemResult,
};
use crate::services::vnjp_sync_service;

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

/// Phân tích sự khác biệt VN/JP RỒI áp dụng luôn trong cùng 1 lượt gọi (frontend gộp 2 nút
/// "Phân tích" + "Áp dụng" thành 1): dọn dẹp strikethrough/chữ đỏ cũ tồn đọng trên file JP, đồng
/// bộ cấu trúc sheet (clone sheet chỉ có ở VN, đánh dấu "(DEL)" sheet VN đã xóa), tự động canh
/// dòng lệch, rồi ghi nội dung VN (giữ nguyên tiếng Việt, tô đỏ) vào đúng vị trí ô tương ứng. Kết
/// quả lưu tự động vào thư mục Temp cạnh nơi cài đặt (không còn hộp thoại chọn nơi lưu). Trả về cả
/// `SyncAnalysis` (để hiển thị tab tổng quan/ô đỏ/strikethrough/quality issues) lẫn `ApplyResult`.
#[tauri::command]
pub fn vnjp_sync_analyze_and_apply(
    vn_path: String,
    jp_path: String,
) -> Result<AnalyzeAndApplyResult, String> {
    vnjp_sync_service::analyze_and_apply(&vn_path, &jp_path).map_err(log_err)
}

/// Dọn dẹp file JP: xóa hẳn nội dung strikethrough cũ + tô đen chữ đỏ cũ tồn đọng từ
/// bản tablet cũ, trên mọi sheet — không phản ánh chữ đỏ VN. Lưu ra output_path.
#[tauri::command]
pub fn vnjp_sync_cleanup(jp_path: String, output_path: String) -> Result<CleanupResult, String> {
    vnjp_sync_service::cleanup_jp(&jp_path, &output_path).map_err(log_err)
}

/// Phát hiện các vị trí VN có dòng mà JP chưa có (lệch dòng), để TL xác nhận trước khi chèn.
#[tauri::command]
pub fn vnjp_sync_analyze_row_alignment(
    vn_path: String,
    jp_path: String,
) -> Result<RowAlignmentReport, String> {
    vnjp_sync_service::analyze_row_alignment(&vn_path, &jp_path).map_err(log_err)
}

/// Chèn dòng trống vào file JP tại các vị trí TL đã xác nhận (đánh số lại row/cell/merge liên quan).
#[tauri::command]
pub fn vnjp_sync_insert_rows(
    jp_path: String,
    vn_path: String,
    output_path: String,
    inserts: Vec<ConfirmedInsert>,
) -> Result<RowInsertResult, String> {
    vnjp_sync_service::insert_rows(&jp_path, &vn_path, &output_path, &inserts).map_err(log_err)
}

/// Dịch VN→JP hàng loạt cho các ô đỏ CHỈ để so sánh (không ghi vào tài liệu), rồi so độ
/// tương đồng với nội dung JP hiện có — cảnh báo ô có thể không thật sự thay đổi hoặc lệch dòng.
#[tauri::command]
pub async fn vnjp_sync_verify_red_cells_ai(
    jp_path: String,
    red_cells: Vec<RedCell>,
    provider: String,
    model: String,
) -> Result<RedCellVerificationReport, String> {
    vnjp_sync_service::verify_red_cells_with_ai(&jp_path, &red_cells, &provider, &model)
        .await
        .map_err(log_err)
}
