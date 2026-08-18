//! API công khai (Tauri command) cho công cụ đồng bộ tài liệu thiết kế chi tiết VN → JP.
//!
//! Toàn bộ xử lý thực sự nằm ở `crate::services::vnjp`: `vnjp::sync_service` chứa các hàm CẤP
//! THẤP dùng chung (đọc file, quét ô đỏ/strikethrough, dọn dẹp, canh dòng, xuất báo cáo, dịch
//! AI...); mỗi `vnjp::c23{2,3,4,5,6,8}_sync_service` là class xử lý chi tiết của 1 loại tài liệu —
//! khai báo vùng cột nội dung riêng (và với C2.3.8 — thuật toán canh dòng theo group), đồng thời
//! TỰ lắp ráp đủ pipeline "Áp dụng" từ các hàm cấp thấp đó (chấp nhận lặp code giữa các loại tài
//! liệu). File này chỉ là lớp mỏng để các Tauri command trong `commands::vnjp_sync_commands` gọi
//! vào; `apply_changes` bên dưới là nơi DUY NHẤT làm việc `detect_doc_type` để chọn đúng class.

use crate::app::result::AppResult;
use crate::models::vnjp_sync::*;
use crate::services::vnjp::sync_service::{self, DocType};
use crate::services::vnjp::{
    c232_sync_service, c233_sync_service, c234_sync_service, c235_sync_service,
    c236_sync_service, c238_sync_service,
};

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

/// Chạy pipeline "Áp dụng" VN → JP: dọn dẹp, đồng bộ cấu trúc sheet (clone/đánh dấu xóa), canh
/// dòng tự động, rồi ghi nội dung ô đỏ VN — lưu ra `Temp/{tên JP gốc}_merged.{ext}` (đường dẫn
/// tự tính, không còn tham số `output_path`). DISPATCH theo `detect_doc_type` sang đúng
/// `apply_changes` của class xử lý chi tiết tương ứng — mỗi class tự chạy đủ pipeline (xem module
/// doc của từng `c23X_sync_service`). Loại tài liệu không nhận diện được (`DocType::Unknown`)
/// không có class riêng để dispatch tới — chạy tạm theo pipeline của C2.3.2, vì pipeline mọi class
/// đều gọi lại `detect_doc_type` trên chính 2 file đầu vào để tự xác định vùng cột nội dung, nên
/// kết quả không phụ thuộc việc chọn class nào ở đây khi loại tài liệu là Unknown.
pub fn apply_changes(vn_path: &str, jp_path: &str) -> AppResult<ApplyResult> {
    match sync_service::detect_doc_type(vn_path, jp_path) {
        DocType::C232 => c232_sync_service::apply_changes(vn_path, jp_path),
        DocType::C233 => c233_sync_service::apply_changes(vn_path, jp_path),
        DocType::C234 => c234_sync_service::apply_changes(vn_path, jp_path),
        DocType::C235 => c235_sync_service::apply_changes(vn_path, jp_path),
        DocType::C236 => c236_sync_service::apply_changes(vn_path, jp_path),
        DocType::C238 => c238_sync_service::apply_changes(vn_path, jp_path),
        DocType::Unknown => c232_sync_service::apply_changes(vn_path, jp_path),
    }
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
