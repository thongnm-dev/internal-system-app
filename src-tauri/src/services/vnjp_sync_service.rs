//! API công khai (Tauri command) cho công cụ đồng bộ tài liệu thiết kế chi tiết VN → JP.
//!
//! Toàn bộ xử lý thực sự nằm ở `crate::services::vnjp`: `vnjp::sync_service` chứa xử lý CHUNG
//! (đọc file, quét ô đỏ/strikethrough, dọn dẹp, ghi ô đỏ, canh dòng, xuất báo cáo, dịch AI...);
//! mỗi `vnjp::c23{2,3,4,5,6,8}_sync_service` chỉ chứa phần xử lý RIÊNG theo loại tài liệu (vùng
//! cột nội dung, và với C2.3.8 — thuật toán canh dòng theo group). File này chỉ là lớp mỏng để
//! các Tauri command trong `commands::vnjp_sync_commands` gọi vào.

use crate::app::result::AppResult;
use crate::models::vnjp_sync::*;
use crate::services::vnjp::sync_service;

/// Phân tích 2 file Excel VN + JP, trả về SyncAnalysis đầy đủ.
pub fn analyze(vn_path: &str, jp_path: &str) -> AppResult<SyncAnalysis> {
    sync_service::analyze(vn_path, jp_path)
}

/// Dịch hàng loạt các đoạn văn VN → JP qua AI API (Gemini hoặc Groq).
pub async fn translate_batch(
    request: TranslateBatchRequest,
) -> AppResult<Vec<TranslateItemResult>> {
    sync_service::translate_batch(request).await
}

/// Xuất báo cáo phân tích ra file Excel (.xlsx).
pub fn export_report(analysis: &SyncAnalysis, output_path: &str) -> AppResult<String> {
    sync_service::export_report(analysis, output_path)
}

/// Phân tích VN file, lấy danh sách ô đỏ, rồi ghi VN text (in đỏ) vào đúng vị trí
/// tương ứng trong JP file. Kết quả lưu ra `output_path`.
pub fn apply_changes(vn_path: &str, jp_path: &str, output_path: &str) -> AppResult<ApplyResult> {
    sync_service::apply_changes(vn_path, jp_path, output_path)
}

/// Dọn dẹp file JP: xóa hẳn nội dung strikethrough cũ + tô đen chữ đỏ cũ tồn đọng từ
/// bản tablet cũ, trên MỌI sheet. Kết quả lưu ra `output_path`.
pub fn cleanup_jp(jp_path: &str, output_path: &str) -> AppResult<CleanupResult> {
    sync_service::cleanup_jp(jp_path, output_path)
}

/// Phát hiện các vị trí VN có dòng mà JP chưa có (lệch dòng), theo từng sheet chung giữa 2 file.
pub fn analyze_row_alignment(vn_path: &str, jp_path: &str) -> AppResult<RowAlignmentReport> {
    sync_service::analyze_row_alignment(vn_path, jp_path)
}

/// Chèn dòng trống vào file JP tại các vị trí TL đã xác nhận, lưu ra file mới.
pub fn insert_rows(
    jp_path: &str,
    vn_path: &str,
    output_path: &str,
    inserts: &[ConfirmedInsert],
) -> AppResult<RowInsertResult> {
    sync_service::insert_rows(jp_path, vn_path, output_path, inserts)
}

/// Dịch VN→JP hàng loạt cho các ô đỏ CHỈ để so sánh (không ghi vào tài liệu), rồi so độ
/// tương đồng với nội dung JP hiện có.
pub async fn verify_red_cells_with_ai(
    jp_path: &str,
    red_cells: &[RedCell],
    provider: &str,
    model: &str,
) -> AppResult<RedCellVerificationReport> {
    sync_service::verify_red_cells_with_ai(jp_path, red_cells, provider, model).await
}
