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
use crate::services::vnjp::sync_service::{self, ContentBounds, DocType, content_bounds_for_sheet};
use crate::services::vnjp::{
    c232_sync_service, c233_sync_service, c234_sync_service, c235_sync_service,
    c236_sync_service, c238_sync_service,
};

/// Đường dẫn thư mục `Temp` — nơi lưu file kết quả của pipeline "Áp dụng" (xem
/// `sync_service::merged_output_path`). Dùng để frontend liệt kê các file đã tạo ra, cho TL tự
/// mở/copy sang thư mục làm việc khác.
pub fn temp_dir_path() -> String {
    crate::utils::app_config::temp_dir()
        .to_string_lossy()
        .to_string()
}

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
    let mut result = match sync_service::detect_doc_type(vn_path, jp_path) {
        DocType::C232 => c232_sync_service::apply_changes(vn_path, jp_path),
        DocType::C233 => c233_sync_service::apply_changes(vn_path, jp_path),
        DocType::C234 => c234_sync_service::apply_changes(vn_path, jp_path),
        DocType::C235 => c235_sync_service::apply_changes(vn_path, jp_path),
        DocType::C236 => c236_sync_service::apply_changes(vn_path, jp_path),
        DocType::C238 => c238_sync_service::apply_changes(vn_path, jp_path),
        DocType::Unknown => c232_sync_service::apply_changes(vn_path, jp_path),
    }?;

    // Áp từ điển đã học từ các tài liệu khác lên output, ĐỒNG THỜI kiểm tra output sau chuẩn
    // hoá (phát hiện ô có dữ liệu ở VN nhưng mất ở output, hoặc ngược lại) — cả 2 việc chạy
    // trong CÙNG 1 lượt loop sheet → row → cell (xem `sync_service::apply_dictionary_and_verify_data`)
    // thay vì 2 lượt scan riêng. Không làm hỏng kết quả "Áp dụng" nếu bước này lỗi.
    let dictionary = crate::database::vnjp_dictionary_store::load().unwrap_or_default();
    if let Ok((applied_count, mismatches)) = apply_dictionary_and_verify(
        vn_path,
        jp_path,
        &result.output_path,
        &result.output_path,
        &dictionary,
    ) {
        result.dictionary_applied_count = applied_count;
        result.data_mismatches = mismatches;
    }

    // Thu thập thêm từ điển replace từ các ô output (đã áp từ điển ở trên) mà vẫn giữ nội dung
    // JP, gộp vào từ điển tích luỹ cục bộ để tái sử dụng cho các tài liệu khác.
    if let Ok(new_entries) = build_dictionary(vn_path, jp_path, &result.output_path) {
        let _ = crate::database::vnjp_dictionary_store::merge(&new_entries);
    }

    Ok(result)
}

/// Gộp "Phân tích" + "Áp dụng" thành 1 lượt gọi duy nhất — tránh frontend phải gọi 2 command
/// riêng rồi tự nối kết quả. Phân tích trước để có dữ liệu hiển thị (tab tổng quan/ô đỏ/
/// strikethrough/quality issues) VÀ áp dụng luôn ngay sau đó trên cùng cặp file; nếu bước áp
/// dụng lỗi (vd không có gì khác biệt để áp dụng), trả lỗi đó nhưng phần phân tích coi như không
/// tồn tại — frontend không cần xử lý riêng trường hợp "có analysis mà không có apply".
pub fn analyze_and_apply(vn_path: &str, jp_path: &str) -> AppResult<AnalyzeAndApplyResult> {
    let analysis = analyze(vn_path, jp_path)?;
    let apply = apply_changes(vn_path, jp_path)?;
    Ok(AnalyzeAndApplyResult { analysis, apply })
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

/// Thu thập từ điển replace (vn_text → jp_text) từ các ô mà output đã giữ nội dung JP.
/// Dùng sau bước verify_output, kết quả tái sử dụng cho các tài liệu khác.
pub fn build_dictionary(
    vn_path: &str,
    jp_path: &str,
    output_path: &str,
) -> AppResult<std::collections::HashMap<String, String>> {
    match sync_service::detect_doc_type(vn_path, jp_path) {
        DocType::C234 => c234_sync_service::build_dictionary(vn_path, output_path),
        DocType::C235 => c235_sync_service::build_dictionary(vn_path, output_path),
        DocType::C236 => c236_sync_service::build_dictionary(vn_path, output_path),
        DocType::C238 => c238_sync_service::build_dictionary(vn_path, output_path),
        _ => Ok(std::collections::HashMap::new()),
    }
}

/// Áp từ điển replace lên file, ĐỒNG THỜI kiểm tra output sau chuẩn hoá (so sự có mặt dữ liệu
/// — có/không, không so nội dung — với `vn_path` tại từng ô) trong CÙNG 1 lượt loop
/// sheet → row → cell — DISPATCH theo `detect_doc_type` sang đúng class xử lý tương ứng.
/// Trả về `(số ô đã thay thế, danh sách mismatch)`.
pub fn apply_dictionary_and_verify(
    vn_path: &str,
    jp_path: &str,
    file_path: &str,
    output_path: &str,
    dictionary: &std::collections::HashMap<String, String>,
) -> AppResult<(usize, Vec<CellDataMismatch>)> {
    match sync_service::detect_doc_type(vn_path, jp_path) {
        DocType::C234 => {
            c234_sync_service::apply_dictionary_and_verify(file_path, vn_path, output_path, dictionary)
        }
        DocType::C235 => {
            c235_sync_service::apply_dictionary_and_verify(file_path, vn_path, output_path, dictionary)
        }
        DocType::C236 => {
            c236_sync_service::apply_dictionary_and_verify(file_path, vn_path, output_path, dictionary)
        }
        DocType::C238 => {
            c238_sync_service::apply_dictionary_and_verify(file_path, vn_path, output_path, dictionary)
        }
        _ => Ok((0, Vec::new())),
    }
}

/// Vùng nội dung hợp lệ của 1 sheet theo loại tài liệu — dispatch tới đúng method của từng loại.
/// `None` nếu không nhận diện được loại tài liệu (`DocType::Unknown`) ⇒ không giới hạn cột.
pub(crate) fn content_bounds_for(doc_type: DocType, sheet_name: &str) -> Option<ContentBounds> {
    match doc_type {
        DocType::C232 => Some(c232_sync_service::content_bounds(sheet_name)),
        DocType::C233 => Some(c233_sync_service::content_bounds(sheet_name)),
        DocType::C234 => Some(c234_sync_service::content_bounds(sheet_name)),
        DocType::C235 => Some(c235_sync_service::content_bounds(sheet_name)),
        DocType::C236 => Some(c236_sync_service::content_bounds(sheet_name)),
        DocType::C238 => Some(content_bounds_for_sheet(sheet_name, 10)),
        DocType::Unknown => None,
    }
}
