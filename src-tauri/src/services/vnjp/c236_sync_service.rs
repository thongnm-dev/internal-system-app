//! Method xử lý riêng cho C2.3.6 画面仕様書（相関チェック）.
//!
//! Cùng chiến lược với C2.3.4/C2.3.8: pipeline "Áp dụng" ở đây clone TOÀN BỘ nội dung sheet VN
//! vào JP output thay vì chỉ ghi đè ô đỏ — vì C2.3.6 cùng họ tài liệu "画面仕様書" với C2.3.4 nên
//! cũng có thể có thêm dòng mới, đổi style cả dòng, đổi màu nền, border... đòi hỏi toàn bộ nội
//! dung + style của VN phải được phản ánh chính xác vào kết quả. Cột A được bảo toàn bằng công
//! thức JP (STT tự đánh số) thay vì nội dung VN.
//!
//! Pipeline `apply_changes`:
//! 1. `sync_structure` — dọn dẹp JP, clone sheet chỉ có ở VN, đổi tên "(DEL)", sắp xếp sheet.
//! 2. Merge styles VN→JP.
//! 3. Quét toàn bộ ô VN "coi như đã thay đổi": strikethrough HOẶC màu chữ KHÔNG phải đen (bất kỳ
//!    màu nào, không riêng đỏ/xanh) — `find_changed_style_cells_xlsx`.
//! 4. Vòng loop qua từng sheet chung (VN ∩ JP − cloned − DEL):
//!    a. Trích style + dòng đầu tiên có công thức ở cột A JP (`extract_jp_col_a_info`) — CHỈ lấy
//!       style, bỏ qua chuỗi công thức gốc (tham chiếu ô tương đối `MAX($A$2:A{n})+1`, KHÔNG
//!       copy y nguyên được sang dòng khác — khác `OFFSET/INDIRECT/ROW()` của C2.3.4).
//!    b. Clone toàn bộ sheet VN vào JP output (`clone_vn_sheet_for_jp`, `use_col_a_formula =
//!       false` — cột A xử lý như cột thường ở bước này):
//!       - Hàng header (row < 7): giữ nguyên (kể cả cột A), NGOẠI TRỪ các ô A3, C3, K3, M3
//!         luôn lấy từ JP (`jp_preserved_header_cells`).
//!       - Hàng nội dung (row ≥ 7): remap style, inline string; riêng từng ô (kể cả cột A) —
//!         nếu ô đó KHÔNG nằm trong tập "đã thay đổi" ở bước 3 VÀ JP đã có ô tại đúng vị trí
//!         đó, GIỮ NGUYÊN ô JP thay vì ghi đè bằng VN.
//!    c. `inject_stt_group_formula` — GHI ĐÈ cột A của dòng GIỮA mỗi nhóm 3 dòng (tính từ dòng
//!       có công thức đầu tiên, trừ dòng tiêu đề nhóm có merge bắt đầu từ cột A) bằng công thức
//!       STT tự sinh `MAX($A$2:A{row1-1})+1` theo đúng dòng đích.
//!       - Riêng sheet "変更履歴" (`CHANGE_HISTORY_SHEET_NAME`): KHÔNG có cột STT tự đánh số —
//!         bỏ qua bước này, cột A giữ nguyên như clone thường.
//! 4b. Chuẩn hóa rows 4-6: toàn bộ cells ở rows 4, 5, 6 được thay bằng JP reference
//!     (`normalize_rows_4_to_6`). Xóa mergeCell bao phủ A4:A6 — đảm bảo A4="項", A5=blank,
//!     A6="番" là cells riêng lẻ, không merge. Bỏ qua 変更履歴.
//! 4c. Sheet clone trực tiếp từ VN: các ô header row 3 (A3, C3, K3, M3) được ghi đè bằng nội
//!     dung từ sheet JP bất kỳ, sau đó normalize rows 4-6.
//! 5. Ghi output.

use std::collections::{HashMap, HashSet};
use std::fs::File;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::vnjp_sync::ApplyResult;

use super::sync_service::{
    apply_surgery, clone_vn_sheet_for_jp, extract_all_shared_strings, extract_jp_col_a_info,
    find_changed_style_cells_xlsx, is_del_sheet_name, merged_output_path, merge_vn_styles_into_jp,
    parse_cell_ref, read_zip_entry, resolve_sheet_xml_paths, sync_structure, write_output_zip,
    ContentBounds, SurgeryEdit, CHANGE_HISTORY_SHEET_NAME,
};

/// Nội dung cột A ~ N (0-based 13).
pub fn content_bounds(sheet_name: &str) -> ContentBounds {
    super::sync_service::content_bounds_for_sheet(sheet_name, 13)
}

/// Các ô header ROW 3 luôn lấy từ JP: A3, C3, K3, M3 — (row1, col0).
/// Rows 4-6 được xử lý riêng bởi `normalize_rows_4_to_6`.
fn jp_preserved_header_cells() -> HashSet<(usize, usize)> {
    [
        (3, 0),  // A3
        (3, 2),  // C3
        (3, 10), // K3
        (3, 12), // M3
    ]
    .into_iter()
    .collect()
}

/// Trích toàn bộ cell XML của rows 4, 5, 6 từ sheet JP đầu tiên có nội dung (bỏ qua
/// 変更履歴 và DEL sheets).
fn extract_jp_ref_rows_4_to_6(
    jp_archive: &mut zip::ZipArchive<File>,
    jp_sheet_map: &HashMap<String, String>,
    exclude_names: &HashSet<String>,
) -> HashMap<usize, String> {
    let target_rows: [usize; 3] = [4, 5, 6];
    let mut result: HashMap<usize, String> = HashMap::new();
    for (sheet_name, xml_path) in jp_sheet_map {
        if exclude_names.contains(sheet_name)
            || sheet_name == CHANGE_HISTORY_SHEET_NAME
            || is_del_sheet_name(sheet_name)
        {
            continue;
        }
        let Some(sheet_xml) = read_zip_entry(jp_archive, xml_path) else {
            continue;
        };
        let Ok(doc) = roxmltree::Document::parse(&sheet_xml) else {
            continue;
        };
        let Some(sd) = doc
            .descendants()
            .find(|n| n.tag_name().name() == "sheetData")
        else {
            continue;
        };
        for row in sd.children().filter(|n| n.tag_name().name() == "row") {
            let row1 = row
                .attribute("r")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            if !target_rows.contains(&row1) {
                continue;
            }
            result.entry(row1).or_insert_with(|| {
                row.children()
                    .filter(|c| c.tag_name().name() == "c")
                    .map(|c| sheet_xml[c.range()].to_string())
                    .collect::<Vec<_>>()
                    .join("")
            });
        }
        if result.len() == target_rows.len() {
            break;
        }
    }
    result
}

/// Kiểm tra merge ref (vd "A4:A6") có overlap với bất kỳ ô nào trong `cells` (row0, col0) không.
fn merge_overlaps_forbidden(ref_str: &str, cells: &[(usize, usize)]) -> bool {
    let Some((start, end)) = ref_str.split_once(':') else {
        return false;
    };
    let Some((sr0, sc0)) = parse_cell_ref(start) else {
        return false;
    };
    let Some((er0, ec0)) = parse_cell_ref(end) else {
        return false;
    };
    cells
        .iter()
        .any(|&(r0, c0)| r0 >= sr0 && r0 <= er0 && c0 >= sc0 && c0 <= ec0)
}

/// Chuẩn hóa rows 4-6: thay toàn bộ cells bằng JP reference, xóa mergeCell bao phủ A4:A6.
fn normalize_rows_4_to_6(sheet_xml: &str, jp_ref_cells: &HashMap<usize, String>) -> String {
    if jp_ref_cells.is_empty() {
        return sheet_xml.to_string();
    }
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return sheet_xml.to_string();
    };
    let Some(sd) = doc
        .descendants()
        .find(|n| n.tag_name().name() == "sheetData")
    else {
        return sheet_xml.to_string();
    };

    let mut edits: Vec<SurgeryEdit> = Vec::new();

    for row in sd.children().filter(|n| n.tag_name().name() == "row") {
        let row1 = row
            .attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let Some(jp_cells_xml) = jp_ref_cells.get(&row1) else {
            continue;
        };
        let first_cell = row.children().find(|c| c.tag_name().name() == "c");
        let last_cell = row
            .children()
            .filter(|c| c.tag_name().name() == "c")
            .last();
        if let (Some(first), Some(last)) = (first_cell, last_cell) {
            edits.push(SurgeryEdit {
                start: first.range().start,
                end: last.range().end,
                replacement: jp_cells_xml.clone(),
            });
        }
    }

    let forbidden: [(usize, usize); 3] = [(3, 0), (4, 0), (5, 0)];
    if let Some(mc) = doc
        .descendants()
        .find(|n| n.tag_name().name() == "mergeCells")
    {
        let mut removed = 0usize;
        for merge in mc.children().filter(|n| n.tag_name().name() == "mergeCell") {
            if let Some(ref_str) = merge.attribute("ref") {
                if merge_overlaps_forbidden(ref_str, &forbidden) {
                    edits.push(SurgeryEdit {
                        start: merge.range().start,
                        end: merge.range().end,
                        replacement: String::new(),
                    });
                    removed += 1;
                }
            }
        }
        if removed > 0 {
            if let Some(attr) = mc.attributes().find(|a| a.name() == "count") {
                let old_count: usize = attr.value().parse().unwrap_or(0);
                let new_count = old_count.saturating_sub(removed);
                edits.push(SurgeryEdit {
                    start: attr.range_value().start,
                    end: attr.range_value().end,
                    replacement: new_count.to_string(),
                });
            }
        }
    }

    if edits.is_empty() {
        return sheet_xml.to_string();
    }
    apply_surgery(sheet_xml, edits)
}

/// Trích raw cell XML tại các vị trí `positions` từ sheet JP bất kỳ.
fn extract_jp_header_cells(
    jp_archive: &mut zip::ZipArchive<File>,
    jp_sheet_map: &HashMap<String, String>,
    positions: &HashSet<(usize, usize)>,
) -> HashMap<(usize, usize), String> {
    let mut result = HashMap::new();
    for xml_path in jp_sheet_map.values() {
        let Some(sheet_xml) = read_zip_entry(jp_archive, xml_path) else {
            continue;
        };
        let Ok(doc) = roxmltree::Document::parse(&sheet_xml) else {
            continue;
        };
        let Some(sd) = doc
            .descendants()
            .find(|n| n.tag_name().name() == "sheetData")
        else {
            continue;
        };
        for row in sd.children().filter(|n| n.tag_name().name() == "row") {
            let row1 = row
                .attribute("r")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            for cell in row.children().filter(|n| n.tag_name().name() == "c") {
                if let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) {
                    if positions.contains(&(row1, col0)) {
                        result
                            .entry((row1, col0))
                            .or_insert_with(|| sheet_xml[cell.range()].to_string());
                    }
                }
            }
        }
        if result.len() == positions.len() {
            break;
        }
    }
    result
}

/// Ghi đè các ô header trong sheet XML bằng nội dung JP đã trích.
fn patch_header_cells_in_sheet(
    sheet_xml: &str,
    jp_cells: &HashMap<(usize, usize), String>,
) -> String {
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return sheet_xml.to_string();
    };
    let Some(sd) = doc
        .descendants()
        .find(|n| n.tag_name().name() == "sheetData")
    else {
        return sheet_xml.to_string();
    };

    struct Edit {
        start: usize,
        end: usize,
        replacement: String,
    }
    let mut edits: Vec<Edit> = Vec::new();

    for row in sd.children().filter(|n| n.tag_name().name() == "row") {
        let row1 = row
            .attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        for cell in row.children().filter(|n| n.tag_name().name() == "c") {
            if let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) {
                if let Some(jp_raw) = jp_cells.get(&(row1, col0)) {
                    edits.push(Edit {
                        start: cell.range().start,
                        end: cell.range().end,
                        replacement: jp_raw.clone(),
                    });
                }
            }
        }
    }

    if edits.is_empty() {
        return sheet_xml.to_string();
    }

    edits.sort_by(|a, b| b.start.cmp(&a.start));
    let mut result = sheet_xml.to_string();
    for edit in edits {
        result.replace_range(edit.start..edit.end, &edit.replacement);
    }
    result
}

/// Số dòng Excel gộp thành 1 mã STT (không merge cell — 3 dòng rời, công thức chỉ ở dòng giữa).
const STT_GROUP_SIZE: usize = 3;

/// Ghi đè cột A của dòng GIỮA mỗi nhóm `STT_GROUP_SIZE` dòng (tính từ `first_formula_row1`)
/// bằng công thức STT `MAX($A$2:A{row1-1})+1`, BỎ QUA dòng tiêu đề nhóm (merge bắt đầu từ cột
/// A). Tương tự `inject_stt_group_formula` của C2.3.5, có thêm logic skip dòng header nhóm.
fn inject_stt_group_formula(
    sheet_xml: &str,
    first_formula_row1: usize,
    style: usize,
) -> String {
    if STT_GROUP_SIZE == 0 {
        return sheet_xml.to_string();
    }
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return sheet_xml.to_string();
    };
    let Some(sheet_data) = doc
        .descendants()
        .find(|n| n.tag_name().name() == "sheetData")
    else {
        return sheet_xml.to_string();
    };

    let group_header_rows: HashSet<usize> = {
        let mut set = HashSet::new();
        if let Some(mc) = doc
            .descendants()
            .find(|n| n.tag_name().name() == "mergeCells")
        {
            for merge in mc.children().filter(|n| n.tag_name().name() == "mergeCell") {
                if let Some(ref_str) = merge.attribute("ref") {
                    if let Some((start, _)) = ref_str.split_once(':') {
                        if let Some((r0, c0)) = parse_cell_ref(start) {
                            if c0 == 0 {
                                set.insert(r0 + 1);
                            }
                        }
                    }
                }
            }
        }
        set
    };

    let formula_offset = first_formula_row1 % STT_GROUP_SIZE;

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    for row in sheet_data
        .children()
        .filter(|n| n.tag_name().name() == "row")
    {
        let Some(row1) = row.attribute("r").and_then(|s| s.parse::<usize>().ok()) else {
            continue;
        };
        if row1 < first_formula_row1
            || row1 % STT_GROUP_SIZE != formula_offset
            || group_header_rows.contains(&row1)
        {
            continue;
        }

        let new_cell = format!(
            r#"<c r="A{row1}" s="{style}"><f ca="1">MAX($A$2:A{})+1</f></c>"#,
            row1 - 1
        );

        let existing_a = row.children().find(|c| {
            c.tag_name().name() == "c"
                && c.attribute("r")
                    .and_then(parse_cell_ref)
                    .map(|(_, col)| col)
                    == Some(0)
        });
        match existing_a {
            Some(cell) => edits.push(SurgeryEdit {
                start: cell.range().start,
                end: cell.range().end,
                replacement: new_cell,
            }),
            None => {
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

/// Pipeline "Áp dụng" VN → JP cho C2.3.6. Xem module doc.
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

    // Sheet "ﾜｰｸｼｰﾄ" có header rows 1-6, nội dung bắt đầu từ row 7.
    // Các sheet khác: nội dung bắt đầu từ row 4.
    const WORKSHEET_NAME: &str = "ﾜｰｸｼｰﾄ";
    const CONTENT_START_ROW1_DEFAULT: usize = 4;
    const CONTENT_START_ROW1_WORKSHEET: usize = 7;

    let preserved_cells = jp_preserved_header_cells();

    // Trích JP reference rows 4-6 — chỉ dùng cho sheet "ﾜｰｸｼｰﾄ".
    let jp_ref_rows = extract_jp_ref_rows_4_to_6(
        &mut jp_archive,
        &jp_sheet_map,
        cloned_names,
    );

    let mut replaced: HashMap<String, Vec<u8>> = HashMap::new();
    let mut applied_count = 0usize;
    let mut sheets_modified: Vec<String> = structure.sheets_modified.clone();

    // Styles đã merge — ghi vào output
    replaced.insert(
        "xl/styles.xml".to_string(),
        style_result.new_styles_xml.into_bytes(),
    );

    // ── 6. Vòng loop sheet-by-sheet: clone toàn bộ VN → JP ───────────────────
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

        let is_worksheet = sheet_name == WORKSHEET_NAME;
        let content_start = if is_worksheet {
            CONTENT_START_ROW1_WORKSHEET
        } else {
            CONTENT_START_ROW1_DEFAULT
        };

        let (_jp_col_a_formula, jp_col_a_style, formula_start_row1) =
            extract_jp_col_a_info(&jp_sheet_xml, content_start);
        // Công thức STT gốc JP dùng tham chiếu ô tương đối (`MAX($A$2:A7)+1`) — KHÔNG thể copy
        // y nguyên sang dòng khác. Vì vậy cột A xử lý như cột thường khi clone, sau đó ghi đè
        // bằng công thức tự sinh đúng per-row (trừ 変更履歴 không có cột STT).
        let is_change_history = sheet_name == CHANGE_HISTORY_SHEET_NAME;

        // Clone toàn bộ VN sheet vào khung JP.
        // use_col_a_formula = false — cột A xử lý như cột thường, công thức STT chèn riêng sau.
        // Sheet "ﾜｰｸｼｰﾄ": A3, C3, K3, M3 luôn lấy từ JP. Các sheet khác: không preserved.
        let cloned_xml = clone_vn_sheet_for_jp(
            &vn_sheet_xml,
            &jp_sheet_xml,
            "",
            0,
            formula_start_row1,
            &style_result.xf_remap,
            &vn_plain_ssi,
            &vn_rich_ssi,
            vn_changed_cells.get(sheet_name),
            false,
            if is_worksheet { Some(&preserved_cells) } else { None },
        );

        // Chèn công thức STT cho dòng giữa mỗi nhóm 3 dòng (trừ 変更履歴).
        let with_stt = if !is_change_history {
            inject_stt_group_formula(&cloned_xml, formula_start_row1, jp_col_a_style)
        } else {
            cloned_xml
        };

        // Chuẩn hóa rows 4-6 — chỉ cho sheet "ﾜｰｸｼｰﾄ".
        let new_sheet_xml = if is_worksheet {
            normalize_rows_4_to_6(&with_stt, &jp_ref_rows)
        } else {
            with_stt
        };

        applied_count += 1;
        replaced.insert(jp_xml_path, new_sheet_xml.into_bytes());
        if !sheets_modified.contains(sheet_name) {
            sheets_modified.push(sheet_name.clone());
        }
    }

    // ── 6b. Cloned sheets: chỉ sheet "ﾜｰｸｼｰﾄ" mới ghi đè header + normalize ──
    if !cloned_names.is_empty() {
        let jp_header_cells =
            extract_jp_header_cells(&mut jp_archive, &jp_sheet_map, &preserved_cells);
        for cloned_name in cloned_names {
            if cloned_name != WORKSHEET_NAME {
                continue;
            }
            let cloned_xml_path = match jp_sheet_map.get(cloned_name) {
                Some(p) => p.clone(),
                None => continue,
            };
            let cloned_xml = if let Some(bytes) = replaced.get(&cloned_xml_path) {
                String::from_utf8_lossy(bytes).to_string()
            } else if let Some(xml) = read_zip_entry(&mut jp_archive, &cloned_xml_path) {
                xml
            } else {
                continue;
            };
            let patched = if !jp_header_cells.is_empty() {
                patch_header_cells_in_sheet(&cloned_xml, &jp_header_cells)
            } else {
                cloned_xml
            };
            let normalized = normalize_rows_4_to_6(&patched, &jp_ref_rows);
            replaced.insert(cloned_xml_path, normalized.into_bytes());
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
