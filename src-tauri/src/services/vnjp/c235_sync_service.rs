//! Method xử lý riêng cho C2.3.5 画面仕様書（単独チェック）.
//!
//! Cùng chiến lược với C2.3.4/C2.3.6: pipeline "Áp dụng" ở đây clone TOÀN BỘ nội dung sheet VN
//! vào JP output thay vì chỉ ghi đè ô đỏ. Điểm khác biệt DUY NHẤT so với C2.3.4/C2.3.6: cột STT
//! (cột A) của C2.3.5 không đánh số theo TỪNG dòng — mỗi mã STT gộp 3 dòng Excel liên tiếp (không
//! merge cell, 3 ô rời), công thức STT thực sự chỉ nằm ở dòng GIỮA của mỗi nhóm 3 dòng đó, 2 dòng
//! còn lại cột A để trống. Công thức gốc trong JP dùng tham chiếu ô tương đối kiểu
//! `=MAX($A$2:A7)+1` (không dùng ROW()/OFFSET()/INDIRECT() như C2.3.4 nên KHÔNG thể copy y nguyên
//! chuỗi công thức sang dòng khác — tham chiếu ô sẽ sai lệch). Vì vậy công thức được TỰ SINH lại
//! cho từng dòng giữa theo đúng quy luật đó: `=MAX($A$2:A{dòng-1})+1` (xem `inject_stt_group_formula`).
//!
//! Pipeline `apply_changes`:
//! 1. `sync_structure` — dọn dẹp JP, clone sheet chỉ có ở VN, đổi tên "(DEL)", sắp xếp sheet.
//! 2. Merge styles VN→JP.
//! 3. Quét toàn bộ ô VN "coi như đã thay đổi": strikethrough HOẶC màu chữ KHÔNG phải đen (bất kỳ
//!    màu nào, không riêng đỏ/xanh) — `find_changed_style_cells_xlsx`.
//! 4. Vòng loop qua từng sheet chung (VN ∩ JP − cloned − DEL):
//!    a. Trích style + dòng đầu tiên CÓ công thức ở cột A JP (`extract_jp_col_a_info`) — CHỈ lấy
//!       style, bỏ qua chuỗi công thức gốc (không dùng được do tham chiếu ô tương đối).
//!    b. Clone toàn bộ sheet VN vào JP output (`clone_vn_sheet_for_jp`, `use_col_a_formula =
//!       false` — cột A xử lý như cột thường ở bước này, xem (c)):
//!       - Hàng header (row < 4): giữ nguyên (kể cả cột A).
//!       - Hàng nội dung (row ≥ 4): remap style, inline string; riêng từng ô (kể cả cột A) — nếu
//!         ô đó KHÔNG "đã thay đổi" (chữ đen, không strikethrough) VÀ JP đã có ô tại đúng vị trí
//!         đó, GIỮ NGUYÊN ô JP thay vì ghi đè bằng VN.
//!    c. `inject_stt_group_formula` — GHI ĐÈ cột A của riêng dòng GIỮA mỗi nhóm 3 dòng (tính từ
//!       dòng có công thức đầu tiên ở bước a) bằng công thức STT tự sinh theo đúng dòng đó.
//! 5. Ghi output.

use std::collections::HashMap;
use std::fs::File;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::vnjp_sync::ApplyResult;

use super::sync_service::{
    apply_surgery, clone_vn_sheet_for_jp, extract_all_shared_strings, extract_jp_col_a_info,
    find_changed_style_cells_xlsx, is_del_sheet_name, merged_output_path, merge_vn_styles_into_jp,
    parse_cell_ref, read_zip_entry, resolve_sheet_xml_paths, sync_structure, write_output_zip,
    ContentBounds, SurgeryEdit,
};

/// Nội dung cột A ~ N (0-based 13).
pub fn content_bounds(sheet_name: &str) -> ContentBounds {
    super::sync_service::content_bounds_for_sheet(sheet_name, 13)
}

/// Row bắt đầu vùng nội dung (1-based) — bỏ vùng header cố định (row Excel 1~3).
const CONTENT_START_ROW1: usize = 4;

/// Số dòng Excel gộp thành 1 mã STT (không merge cell — 3 dòng rời, công thức chỉ ở dòng giữa).
const STT_GROUP_SIZE: usize = 3;

/// Ghi đè cột A của riêng dòng GIỮA mỗi nhóm `STT_GROUP_SIZE` dòng (tính từ `first_formula_row1`)
/// bằng công thức STT tự sinh `=MAX($A$2:A{row1-1})+1` — tương đương công thức gốc JP
/// (`=MAX($A$2:A{row phía trên})+1`) nhưng tính lại theo ĐÚNG dòng đích, vì tham chiếu ô tương đối
/// trong chuỗi công thức không tự điều chỉnh khi copy thẳng vào XML (khác `MAX($A$2:OFFSET(...))+1`
/// dùng ROW()/OFFSET()/INDIRECT() của C2.3.4 — chuỗi đó bất biến theo vị trí nên copy y nguyên
/// được). Các dòng còn lại trong nhóm (không phải dòng giữa) giữ nguyên như `clone_vn_sheet_for_jp`
/// đã xử lý (cột A như cột thường).
fn inject_stt_group_formula(
    sheet_xml: &str,
    first_formula_row1: usize,
    group_size: usize,
    style: usize,
) -> String {
    if group_size == 0 {
        return sheet_xml.to_string();
    }
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return sheet_xml.to_string();
    };
    let Some(sheet_data) = doc.descendants().find(|n| n.tag_name().name() == "sheetData") else {
        return sheet_xml.to_string();
    };

    // Vị trí dòng giữa lặp lại theo chu kỳ `group_size`, cùng "pha" với `first_formula_row1`.
    let formula_offset = first_formula_row1 % group_size;

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    for row in sheet_data.children().filter(|n| n.tag_name().name() == "row") {
        let Some(row1) = row.attribute("r").and_then(|s| s.parse::<usize>().ok()) else {
            continue;
        };
        if row1 < first_formula_row1 || row1 % group_size != formula_offset {
            continue;
        }

        let new_cell = format!(
            r#"<c r="A{row1}" s="{style}"><f ca="1">MAX($A$2:A{})+1</f></c>"#,
            row1 - 1
        );

        let existing_a = row.children().find(|c| {
            c.tag_name().name() == "c"
                && c.attribute("r").and_then(parse_cell_ref).map(|(_, col)| col) == Some(0)
        });
        match existing_a {
            Some(cell) => edits.push(SurgeryEdit {
                start: cell.range().start,
                end: cell.range().end,
                replacement: new_cell,
            }),
            None => {
                // Chưa có ô cột A (VN để trống) — chèn ngay trước ô `<c>` đầu tiên của dòng (cột A
                // luôn đứng đầu); dòng rỗng hoàn toàn thì chèn trước `</row>`.
                let insert_pos = row
                    .children()
                    .find(|c| c.tag_name().name() == "c")
                    .map(|c| c.range().start)
                    .unwrap_or_else(|| row.range().end - "</row>".len());
                edits.push(SurgeryEdit {
                    start: insert_pos,
                    end: insert_pos,
                    replacement: new_cell,
                });
            }
        }
    }

    apply_surgery(sheet_xml, edits)
}

/// Pipeline "Áp dụng" VN → JP cho C2.3.5. Xem module doc.
pub fn apply_changes(vn_path: &str, jp_path: &str) -> AppResult<ApplyResult> {
    // ── 1. sync_structure ──────────────────────────────────────────────────────
    let output_path = merged_output_path(jp_path)?;
    let output_path_str = output_path.to_string_lossy().to_string();
    let structure = sync_structure(vn_path, jp_path, &output_path_str)?;

    // ── 2. Mở VN zip ──────────────────────────────────────────────────────────
    let vn_file = File::open(vn_path)
        .map_err(|e| AppError::new(format!("Không mở được file VN: {e}")))?;
    let mut vn_archive = zip::ZipArchive::new(vn_file)
        .map_err(|e| AppError::new(format!("File VN không phải ZIP hợp lệ: {e}")))?;

    let vn_wb_xml = read_zip_entry(&mut vn_archive, "xl/workbook.xml").unwrap_or_default();
    let vn_rels_xml =
        read_zip_entry(&mut vn_archive, "xl/_rels/workbook.xml.rels").unwrap_or_default();
    let vn_sst_xml =
        read_zip_entry(&mut vn_archive, "xl/sharedStrings.xml").unwrap_or_default();
    let vn_styles_xml = read_zip_entry(&mut vn_archive, "xl/styles.xml").unwrap_or_default();
    let (vn_plain_ssi, vn_rich_ssi) = extract_all_shared_strings(&vn_sst_xml);
    let vn_sheet_paths = resolve_sheet_xml_paths(&vn_wb_xml, &vn_rels_xml);
    let vn_sheet_map: HashMap<String, String> = vn_sheet_paths.into_iter().collect();

    // ── 3. Mở JP output zip (sau sync_structure) ──────────────────────────────
    let jp_file = File::open(&output_path_str)
        .map_err(|e| AppError::new(format!("Không mở được file JP output: {e}")))?;
    let mut jp_archive = zip::ZipArchive::new(jp_file)
        .map_err(|e| AppError::new(format!("File JP output không phải ZIP hợp lệ: {e}")))?;

    let jp_wb_xml = read_zip_entry(&mut jp_archive, "xl/workbook.xml").unwrap_or_default();
    let jp_rels_xml =
        read_zip_entry(&mut jp_archive, "xl/_rels/workbook.xml.rels").unwrap_or_default();
    let jp_styles_xml = read_zip_entry(&mut jp_archive, "xl/styles.xml").unwrap_or_default();
    let jp_sheet_paths = resolve_sheet_xml_paths(&jp_wb_xml, &jp_rels_xml);
    let jp_sheet_map: HashMap<String, String> = jp_sheet_paths.into_iter().collect();

    // ── 4. Merge styles VN→JP ─────────────────────────────────────────────────
    let style_result = merge_vn_styles_into_jp(&jp_styles_xml, &vn_styles_xml);

    // Ô VN "coi như đã thay đổi" (strikethrough hoặc màu chữ không phải đen) theo từng sheet — ô
    // KHÔNG nằm trong set này (chữ đen, không strikethrough) sẽ được giữ nguyên bản JP tại đúng
    // vị trí khi clone (xem `clone_vn_sheet_for_jp`).
    let vn_changed_cells = find_changed_style_cells_xlsx(vn_path);

    // ── 5. Chuẩn bị vòng loop ─────────────────────────────────────────────────
    let cloned_names = &structure.cloned_names;
    // Sheet chung: tồn tại ở cả VN lẫn JP output, không phải sheet clone mới, không phải DEL
    let mut common_sheets: Vec<String> = jp_sheet_map
        .keys()
        .filter(|name| {
            vn_sheet_map.contains_key(*name)
                && !cloned_names.contains(*name)
                && !is_del_sheet_name(name)
        })
        .cloned()
        .collect();
    common_sheets.sort();

    let mut replaced: HashMap<String, Vec<u8>> = HashMap::new();
    let mut applied_count = 0usize;
    let mut sheets_modified: Vec<String> = structure.sheets_modified.clone();

    // Styles đã merge — ghi vào output
    replaced.insert(
        "xl/styles.xml".to_string(),
        style_result.new_styles_xml.into_bytes(),
    );

    // ── 6. Vòng loop sheet-by-sheet: clone toàn bộ VN → JP, sau đó chèn công thức STT ──
    for sheet_name in &common_sheets {
        let vn_xml_path = match vn_sheet_map.get(sheet_name) {
            Some(p) => p.clone(),
            None => continue,
        };
        let jp_xml_path = match jp_sheet_map.get(sheet_name) {
            Some(p) => p.clone(),
            None => continue,
        };

        let vn_sheet_xml = match read_zip_entry(&mut vn_archive, &vn_xml_path) {
            Some(x) => x,
            None => continue,
        };
        let jp_sheet_xml = match read_zip_entry(&mut jp_archive, &jp_xml_path) {
            Some(x) => x,
            None => continue,
        };

        // Chỉ lấy style + dòng đầu tiên có công thức ở cột A JP — KHÔNG dùng chuỗi công thức trả
        // về (tham chiếu ô tương đối, không copy y nguyên được sang dòng khác).
        let (_jp_col_a_formula, jp_col_a_style, first_formula_row1) =
            extract_jp_col_a_info(&jp_sheet_xml, CONTENT_START_ROW1);

        // Cột A xử lý như cột thường ở bước clone (use_col_a_formula = false); công thức STT thật
        // sự được chèn riêng ở bước sau (`inject_stt_group_formula`) — chỉ đúng dòng giữa mỗi
        // nhóm 3 dòng mới có công thức, 2 dòng còn lại giữ nguyên như clone thường.
        let cloned_sheet_xml = clone_vn_sheet_for_jp(
            &vn_sheet_xml,
            &jp_sheet_xml,
            "",
            0,
            CONTENT_START_ROW1,
            &style_result.xf_remap,
            &vn_plain_ssi,
            &vn_rich_ssi,
            vn_changed_cells.get(sheet_name),
            false,
        );

        let new_sheet_xml = inject_stt_group_formula(
            &cloned_sheet_xml,
            first_formula_row1,
            STT_GROUP_SIZE,
            jp_col_a_style,
        );

        applied_count += 1;
        replaced.insert(jp_xml_path, new_sheet_xml.into_bytes());
        if !sheets_modified.contains(sheet_name) {
            sheets_modified.push(sheet_name.clone());
        }
    }

    // ── 7. Ghi output ─────────────────────────────────────────────────────────
    write_output_zip(&mut jp_archive, &replaced, &output_path_str)?;

    let nothing_changed = applied_count == 0
        && structure.cloned_names.is_empty()
        && structure.del_renamed_count == 0
        && structure.strike_removed_count == 0;
    if nothing_changed {
        return Err(AppError::new(
            "Không có khác biệt nào giữa VN và JP để áp dụng.",
        ));
    }

    Ok(ApplyResult {
        output_path: output_path_str,
        applied_count,
        skipped_count: 0,
        sheets_modified,
        strike_removed_count: structure.strike_removed_count,
        red_blackened_count: structure.red_blackened_count,
        cleanup_skipped_count: structure.cleanup_skipped_count,
        column_corrected_count: 0,
        shape_applied_count: 0,
        shape_skipped_count: 0,
        cloned_sheet_count: structure.cloned_names.len(),
        del_sheet_count: structure.del_renamed_count,
        rows_inserted: 0,
    })
}
