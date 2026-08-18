//! Method xử lý riêng cho C2.3.2 プログラム処理概要図.
//!
//! Loại tài liệu này dùng chung MỌI xử lý CẤP THẤP (đọc file, quét ô đỏ/strikethrough, dọn dẹp,
//! ghi ô đỏ, canh dòng, xuất báo cáo...) với các loại khác — xem `super::sync_service`. Khác biệt
//! DUY NHẤT ở tầng đó là vùng cột nội dung, khai báo ở đây. `apply_changes` tự lắp ráp đủ pipeline
//! "Áp dụng" (chấp nhận lặp code giữa các loại tài liệu) thay vì gọi 1 hàm dùng chung duy nhất —
//! xem ghi chú tại hàm.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::vnjp_sync::{ApplyResult, ConfirmedInsert};

/// Nội dung cột A ~ AQ (0-based 42).
pub fn content_bounds(sheet_name: &str) -> super::sync_service::ContentBounds {
    super::sync_service::content_bounds_for_sheet(sheet_name, 42)
}

/// Pipeline "Áp dụng" VN → JP cho C2.3.2. Chạy tuần tự trên cùng 1 file kết quả (`Temp/{tên JP
/// gốc}_merged.{ext}`, xem `super::sync_service::merged_output_path`) — không có hộp thoại chọn
/// nơi lưu:
/// 1-2. `super::sync_service::sync_structure` — dọn dẹp JP, clone sheet chỉ có ở VN, đổi tên sheet
///    JP bị VN đánh dấu xóa thành "(DEL)", sắp xếp lại thứ tự sheet. Ghi ra file Temp (bước 4).
/// 3. Canh dòng: tự động phát hiện + chèn dòng lệch giữa VN/JP (không cần xác nhận thủ công —
///    khác với luồng "Kiểm tra khớp dòng" độc lập ở `analyze_row_alignment`/`insert_rows`). Ghi
///    đè vào chính file Temp ở trên.
/// 5-6. `super::sync_service::merge_content` — ghi nội dung ô đỏ/shape VN vào file Temp đã canh
///    dòng, ghi đè lần cuối.
pub fn apply_changes(vn_path: &str, jp_path: &str) -> AppResult<ApplyResult> {
    let output_path = super::sync_service::merged_output_path(jp_path)?;
    let output_path_str = output_path.to_string_lossy().to_string();

    let structure = super::sync_service::sync_structure(vn_path, jp_path, &output_path_str)?;

    // Canh dòng tự động — tính theo file JP GỐC: sheet vừa clone ở bước trên chưa tồn tại trong
    // JP gốc nên tự động bị `analyze_row_alignment` bỏ qua, đúng yêu cầu "không duyệt sheet mới
    // clone". Số dòng của các sheet còn lại không đổi giữa JP gốc và file Temp (cleanup/clone/đổi
    // tên không thêm/bớt dòng) nên vị trí chèn tính từ JP gốc vẫn áp dụng đúng lên file Temp.
    let alignment = super::sync_service::analyze_row_alignment(vn_path, jp_path)?;
    let rows_inserted = if alignment.suggestions.is_empty() {
        0
    } else {
        let inserts: Vec<ConfirmedInsert> = alignment
            .suggestions
            .iter()
            .map(|s| ConfirmedInsert {
                sheet: s.sheet.clone(),
                jp_insert_after_row: s.jp_insert_after_row,
                insert_count: s.insert_count,
                vn_row_start: Some(s.vn_row_start),
                vn_row_end: Some(s.vn_row_end),
            })
            .collect();
        super::sync_service::insert_rows(&output_path_str, vn_path, &output_path_str, &inserts)?
            .rows_inserted
    };

    let mut result = super::sync_service::merge_content(
        vn_path,
        &output_path_str,
        &output_path_str,
        &structure.cloned_names,
    )?;
    result.strike_removed_count += structure.strike_removed_count;
    result.red_blackened_count += structure.red_blackened_count;
    // `cleanup_skipped_count`: KHÔNG cộng dồn — `merge_content` quét lại file đã dọn ở
    // `sync_structure` nên chỉ tái phát hiện đúng những ô còn tồn đọng, không phải ô mới.
    result.cleanup_skipped_count = result.cleanup_skipped_count.max(structure.cleanup_skipped_count);
    result.cloned_sheet_count = structure.cloned_names.len();
    result.rows_inserted = rows_inserted;
    for sheet_name in &structure.sheets_modified {
        if !result.sheets_modified.contains(sheet_name) {
            result.sheets_modified.push(sheet_name.clone());
        }
    }

    let nothing_changed = result.applied_count == 0
        && result.shape_applied_count == 0
        && result.strike_removed_count == 0
        && result.red_blackened_count == 0
        && structure.cloned_names.is_empty()
        && structure.del_renamed_count == 0
        && rows_inserted == 0;
    if nothing_changed {
        return Err(AppError::new(
            "Không có khác biệt nào giữa VN và JP để áp dụng.",
        ));
    }

    Ok(result)
}
