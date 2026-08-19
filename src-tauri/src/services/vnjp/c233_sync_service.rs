//! Method xử lý riêng cho C2.3.3 イベント詳細設計書.
//!
//! Cùng chiến lược với C2.3.2/C2.3.4/C2.3.6/C2.3.8: pipeline "Áp dụng" ở đây clone TOÀN BỘ nội
//! dung sheet VN vào JP output thay vì chỉ ghi đè ô đỏ — đòi hỏi toàn bộ nội dung + style của VN
//! phải được phản ánh chính xác vào kết quả (thêm dòng mới, đổi style cả dòng, đổi màu nền,
//! border...). C2.3.3 KHÔNG có cột STT tự đánh số (giống C2.3.2/C2.3.8) — cột A được xử lý như
//! mọi cột thường.
//!
//! Pipeline `apply_changes`:
//! 1. `sync_structure` — dọn dẹp JP, clone sheet chỉ có ở VN, đổi tên "(DEL)", sắp xếp sheet.
//! 2. Merge styles VN→JP.
//! 3. Quét toàn bộ ô VN "coi như đã thay đổi": strikethrough HOẶC màu chữ KHÔNG phải đen (bất kỳ
//!    màu nào, không riêng đỏ/xanh) — `find_changed_style_cells_xlsx`.
//!    Quét thêm ô VN bị strikethrough — `find_strike_cells_xlsx` — dùng để phân biệt dòng thêm
//!    mới (màu chữ, không strike) với dòng xóa (strike).
//! 4. Vòng loop qua từng sheet chung (VN ∩ JP − cloned − DEL): clone toàn bộ sheet VN vào JP
//!    output (`clone_c233_sheet_for_jp`):
//!    - Hàng header (row < 4): clone toàn bộ VN.
//!    - Dùng counter `inserted` để tính JP row tương ứng: jp_r = row1 - inserted.
//!    - Dòng trống VN + JP có nội dung tại jp_r → dòng trống mới được chèn → inserted++.
//!    - Dòng trống VN + JP cũng trống → clone VN (không tăng inserted).
//!    - Dòng THÊM MỚI (tất cả ô có dữ liệu đều có màu không phải đen VÀ không strike) →
//!      clone VN → inserted++.
//!    - Dòng KHÔNG THAY ĐỔI (tất cả ô đều chữ đen) → copy toàn bộ JP row tại jp_r sang row1.
//!    - Dòng CHỈNH SỬA (lẫn lộn chữ đen và chữ màu) → cell-level: ô đen từ JP tại jp_r, ô màu từ VN.
//!    SAU KHI chèn dòng: số dòng kết quả = số dòng VN (bắt buộc khớp nhau).
//! 5. Ghi output.

use std::collections::{HashMap, HashSet};
use std::fs::File;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::vnjp_sync::ApplyResult;

use super::sync_service::{
    apply_surgery, clone_vn_row_xml, extract_all_shared_strings, find_changed_style_cells_xlsx,
    find_strike_cells_xlsx, is_del_sheet_name, merged_output_path, merge_vn_styles_into_jp,
    parse_cell_ref, read_zip_entry, resolve_sheet_xml_paths, sync_structure, write_output_zip,
    ContentBounds, SurgeryEdit,
};

/// Nội dung cột A ~ AR (0-based 43).
pub fn content_bounds(sheet_name: &str) -> ContentBounds {
    super::sync_service::content_bounds_for_sheet(sheet_name, 43)
}

/// Row bắt đầu vùng nội dung (1-based) — bỏ vùng header cố định (row Excel 1~3).
const CONTENT_START_ROW1: usize = 4;
/// Cột cuối cùng (0-based, inclusive) của vùng nội dung C2.3.3 = AR.
const CONTENT_LAST_COL0: usize = 43;

// ─────────────────────────────────────────────────────────────────────────────
// Helper functions
// ─────────────────────────────────────────────────────────────────────────────

/// Chuyển chỉ số cột 0-based thành chuỗi chữ cái Excel (A, B, ..., Z, AA, AB, ...).
fn col_to_letter(mut col0: usize) -> String {
    let mut letters = Vec::new();
    loop {
        letters.push((b'A' + (col0 % 26) as u8) as char);
        if col0 < 26 {
            break;
        }
        col0 = col0 / 26 - 1;
    }
    letters.iter().rev().collect()
}

/// Escape XML attribute value (& và ").
fn esc_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;")
}

/// Escape XML text content (&, <, >).
fn xml_esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Kiểm tra dòng có ít nhất 1 ô chứa dữ liệu thực trong vùng cột 0..=content_last_col0.
fn has_content_data(row_node: roxmltree::Node, content_last_col0: usize) -> bool {
    row_node.children().filter(|c| c.tag_name().name() == "c").any(|c| {
        let col0 = c
            .attribute("r")
            .and_then(parse_cell_ref)
            .map(|(_, col)| col)
            .unwrap_or(usize::MAX);
        col0 <= content_last_col0
            && c.descendants().any(|ch| {
                (ch.tag_name().name() == "v" || ch.tag_name().name() == "t")
                    && ch.text().map_or(false, |t| !t.trim().is_empty())
            })
    })
}

/// Cập nhật chỉ số dòng trong thuộc tính `r` của cell XML sang `new_row1`.
fn with_row(cell_xml: &str, new_row1: usize) -> String {
    if let Some(r_start) = cell_xml.find("r=\"") {
        let attr_start = r_start + 3;
        if let Some(quote_end) = cell_xml[attr_start..].find('"') {
            let old_ref = &cell_xml[attr_start..attr_start + quote_end];
            let col_part: String = old_ref.chars().take_while(|c| c.is_ascii_uppercase()).collect();
            if !col_part.is_empty() {
                return format!(
                    "{}{}{}",
                    &cell_xml[..attr_start],
                    format!("{col_part}{new_row1}"),
                    &cell_xml[attr_start + quote_end..]
                );
            }
        }
    }
    cell_xml.to_string()
}

/// Clone 1 ô VN sang ô output: remap style + inline hoá shared string.
fn clone_cell(
    cell: roxmltree::Node,
    vn_sheet_xml: &str,
    target_row1: usize,
    xf_remap: &[usize],
    plain_ssi: &HashMap<usize, String>,
    rich_ssi_raw: &HashMap<usize, String>,
) -> String {
    let col0 = cell
        .attribute("r")
        .and_then(parse_cell_ref)
        .map(|(_, c)| c)
        .unwrap_or(0);
    let new_ref = format!("{}{}", col_to_letter(col0), target_row1);
    let new_s = cell
        .attribute("s")
        .and_then(|s| s.parse::<usize>().ok())
        .and_then(|o| xf_remap.get(o).copied());
    let s_part = new_s.map(|v| format!(" s=\"{v}\"")).unwrap_or_default();
    let has_formula = cell.children().any(|c| c.tag_name().name() == "f");

    if !has_formula && cell.attribute("t") == Some("s") {
        if let Some(ssi) = cell
            .children()
            .find(|c| c.tag_name().name() == "v")
            .and_then(|v| v.text())
            .and_then(|t| t.parse::<usize>().ok())
        {
            if let Some(raw) = rich_ssi_raw.get(&ssi) {
                return format!(r#"<c r="{new_ref}"{s_part} t="inlineStr"><is>{raw}</is></c>"#);
            }
            let text = plain_ssi.get(&ssi).cloned().unwrap_or_default();
            return format!(
                r#"<c r="{new_ref}"{s_part} t="inlineStr"><is><t xml:space="preserve">{}</t></is></c>"#,
                xml_esc(&text)
            );
        }
    }

    let t_part = cell.attribute("t").map(|t| format!(" t=\"{t}\"")).unwrap_or_default();
    let inner: String = cell.children().map(|ch| vn_sheet_xml[ch.range()].to_string()).collect();
    if inner.trim().is_empty() {
        format!(r#"<c r="{new_ref}"{s_part}{t_part}/>"#)
    } else {
        format!(r#"<c r="{new_ref}"{s_part}{t_part}>{inner}</c>"#)
    }
}

/// Clone sheet VN → JP output với thuật toán counter `inserted`:
///
/// Với mỗi VN row tại row1, JP row tương ứng là jp_r = row1 - inserted.
/// Sau khi xử lý xong, số dòng kết quả luôn khớp với số dòng VN.
///
/// Trả về (new_sheet_xml, rows_inserted).
fn clone_c233_sheet_for_jp(
    vn_sheet_xml: &str,
    jp_sheet_xml: &str,
    content_start_row1: usize,
    xf_remap: &[usize],
    vn_plain_ssi: &HashMap<usize, String>,
    vn_rich_ssi_raw: &HashMap<usize, String>,
    vn_changed_positions: Option<&HashSet<(usize, usize)>>,
    vn_strike_positions: Option<&HashSet<(usize, usize)>>,
) -> (String, usize) {
    // ── Build JP data structures ─────────────────────────────────────────────
    // jp_cell_lookup: (jp_row1, col0) → raw cell XML slice
    // jp_row_col0s:   jp_row1 → sorted vec of col0s (for copying full rows)
    // jp_row_attrs:   jp_row1 → attributes string of <row> element (excluding r=)
    // jp_has_content: set of jp_row1 values that have actual data in A~AR
    let mut jp_cell_lookup: HashMap<(usize, usize), &str> = HashMap::new();
    let mut jp_row_col0s: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut jp_row_attrs: HashMap<usize, String> = HashMap::new();
    let mut jp_has_content: HashSet<usize> = HashSet::new();

    if let Ok(jp_doc) = roxmltree::Document::parse(jp_sheet_xml) {
        if let Some(jp_sd) = jp_doc.descendants().find(|n| n.tag_name().name() == "sheetData") {
            for row in jp_sd.children().filter(|n| n.tag_name().name() == "row") {
                let r1 = match row.attribute("r").and_then(|s| s.parse::<usize>().ok()) {
                    Some(r) => r,
                    None => continue,
                };
                // Collect row attrs (except r=)
                let attrs: String = row
                    .attributes()
                    .filter(|a| a.name() != "r")
                    .map(|a| format!(" {}=\"{}\"", a.name(), esc_attr(a.value())))
                    .collect();
                jp_row_attrs.insert(r1, attrs);

                let mut col0s: Vec<usize> = Vec::new();
                for cell in row.children().filter(|n| n.tag_name().name() == "c") {
                    if let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) {
                        jp_cell_lookup.insert((r1, col0), &jp_sheet_xml[cell.range()]);
                        col0s.push(col0);
                        // Track if this row has real content data
                        if col0 <= CONTENT_LAST_COL0 {
                            let has_val = cell.descendants().any(|ch| {
                                (ch.tag_name().name() == "v" || ch.tag_name().name() == "t")
                                    && ch.text().map_or(false, |t| !t.trim().is_empty())
                            });
                            if has_val {
                                jp_has_content.insert(r1);
                            }
                        }
                    }
                }
                col0s.sort_unstable();
                jp_row_col0s.insert(r1, col0s);
            }
        }
    } // jp_doc drops — jp_cell_lookup vẫn hợp lệ vì slice từ jp_sheet_xml

    // ── Parse VN sheet ───────────────────────────────────────────────────────
    let Ok(vn_doc) = roxmltree::Document::parse(vn_sheet_xml) else {
        return (jp_sheet_xml.to_string(), 0);
    };
    let Some(vn_sd) = vn_doc.descendants().find(|n| n.tag_name().name() == "sheetData") else {
        return (jp_sheet_xml.to_string(), 0);
    };

    let empty_set: HashSet<(usize, usize)> = HashSet::new();
    let changed = vn_changed_positions.unwrap_or(&empty_set);
    let strike = vn_strike_positions.unwrap_or(&empty_set);

    // ── Main loop ────────────────────────────────────────────────────────────
    let mut new_sd = String::from("<sheetData>");
    let mut inserted = 0usize;
    // Dùng để lan truyền: nếu dòng ngay trước vừa được chèn mới (dòng màu hoặc blank đã chèn),
    // thì dòng blank tiếp theo cũng được coi là chèn mới (không cần kiểm tra JP).
    let mut prev_was_inserted = false;

    for row_node in vn_sd.children().filter(|n| n.tag_name().name() == "row") {
        let row1 = match row_node.attribute("r").and_then(|s| s.parse::<usize>().ok()) {
            Some(r) => r,
            None => continue,
        };

        // Header rows: clone toàn bộ VN, không thay đổi inserted
        if row1 < content_start_row1 {
            new_sd.push_str(&clone_vn_row_xml(
                row_node,
                vn_sheet_xml,
                row1,
                xf_remap,
                vn_plain_ssi,
                vn_rich_ssi_raw,
            ));
            continue;
        }

        // JP row tương ứng với VN row này
        let jp_r = row1 - inserted;
        let row0 = row1 - 1;

        // ── Dòng trống VN ────────────────────────────────────────────────────
        let vn_is_blank = !has_content_data(row_node, CONTENT_LAST_COL0);
        if vn_is_blank {
            // Chèn blank nếu: dòng ngay trước vừa được chèn (prev_was_inserted) HOẶC JP có
            // nội dung tại jp_r (VN đẩy dòng JP xuống bằng cách chèn blank ở đây).
            if prev_was_inserted || jp_has_content.contains(&jp_r) {
                new_sd.push_str(&clone_vn_row_xml(
                    row_node,
                    vn_sheet_xml,
                    row1,
                    xf_remap,
                    vn_plain_ssi,
                    vn_rich_ssi_raw,
                ));
                inserted += 1;
                prev_was_inserted = true; // blank được chèn → dòng blank kế tiếp cũng kiểm tra
            } else {
                // Cả hai đều trống (hoặc JP không có dòng này) → clone VN trống, không chèn
                new_sd.push_str(&clone_vn_row_xml(
                    row_node,
                    vn_sheet_xml,
                    row1,
                    xf_remap,
                    vn_plain_ssi,
                    vn_rich_ssi_raw,
                ));
                prev_was_inserted = false;
            }
            continue;
        }

        // ── Thu thập col0 của các ô có dữ liệu trong vùng A~AR ───────────────
        let vn_data_col0s: Vec<usize> = row_node
            .children()
            .filter(|c| c.tag_name().name() == "c")
            .filter_map(|c| {
                let (_, col0) = c.attribute("r").and_then(parse_cell_ref)?;
                if col0 > CONTENT_LAST_COL0 {
                    return None;
                }
                let has_data = c.descendants().any(|ch| {
                    (ch.tag_name().name() == "v" || ch.tag_name().name() == "t")
                        && ch.text().map_or(false, |t| !t.trim().is_empty())
                });
                if has_data { Some(col0) } else { None }
            })
            .collect();

        // ── Phân loại dòng ────────────────────────────────────────────────────

        // Dòng MỚI: tất cả ô có dữ liệu đều có màu (không đen) VÀ không bị strike
        let all_colored_non_struck = !vn_data_col0s.is_empty()
            && vn_data_col0s.iter().all(|&col0| {
                changed.contains(&(row0, col0)) && !strike.contains(&(row0, col0))
            });

        if all_colored_non_struck {
            new_sd.push_str(&clone_vn_row_xml(
                row_node,
                vn_sheet_xml,
                row1,
                xf_remap,
                vn_plain_ssi,
                vn_rich_ssi_raw,
            ));
            inserted += 1;
            prev_was_inserted = true;
            continue;
        }

        // Dòng KHÔNG THAY ĐỔI: tất cả ô đều chữ đen (không trong changed)
        let all_black = vn_data_col0s.iter().all(|&col0| !changed.contains(&(row0, col0)));

        if all_black {
            // Copy toàn bộ JP row tại jp_r sang output tại row1
            let jp_attrs = jp_row_attrs.get(&jp_r).cloned().unwrap_or_default();
            let jp_cells: String = jp_row_col0s
                .get(&jp_r)
                .iter()
                .flat_map(|cs| cs.iter())
                .filter_map(|&col0| {
                    jp_cell_lookup.get(&(jp_r, col0)).map(|raw| {
                        if jp_r != row1 { with_row(raw, row1) } else { raw.to_string() }
                    })
                })
                .collect();
            if jp_cells.is_empty() {
                new_sd.push_str(&format!(r#"<row r="{row1}"{jp_attrs}/>"#));
            } else {
                new_sd.push_str(&format!(r#"<row r="{row1}"{jp_attrs}>{jp_cells}</row>"#));
            }
            prev_was_inserted = false;
            continue;
        }

        // Dòng CHỈNH SỬA: lẫn lộn chữ đen và chữ màu → cell-level merge
        // Row attrs từ VN (với style remap)
        let mut row_attrs = String::new();
        for a in row_node.attributes() {
            match a.name() {
                "r" => {}
                "s" => {
                    let remapped = a
                        .value()
                        .parse::<usize>()
                        .ok()
                        .and_then(|o| xf_remap.get(o).copied());
                    match remapped {
                        Some(v) => row_attrs.push_str(&format!(" s=\"{v}\"")),
                        None => row_attrs.push_str(&format!(" s=\"{}\"", a.value())),
                    }
                }
                name => row_attrs.push_str(&format!(" {}=\"{}\"", name, esc_attr(a.value()))),
            }
        }

        // Tập hợp col0 từ VN (để xác định JP-only cols)
        let vn_col0s: HashSet<usize> = row_node
            .children()
            .filter(|c| c.tag_name().name() == "c")
            .filter_map(|c| c.attribute("r").and_then(parse_cell_ref).map(|(_, col0)| col0))
            .collect();

        // Cells từ VN: ô màu → clone VN; ô đen → lấy JP tại jp_r
        let mut mixed_cells: Vec<(usize, String)> = row_node
            .children()
            .filter(|c| c.tag_name().name() == "c")
            .filter_map(|c| {
                let (_, col0) = c.attribute("r").and_then(parse_cell_ref)?;
                let is_changed = changed.contains(&(row0, col0));
                let cell_xml = if is_changed {
                    clone_cell(c, vn_sheet_xml, row1, xf_remap, vn_plain_ssi, vn_rich_ssi_raw)
                } else if let Some(jp_raw) = jp_cell_lookup.get(&(jp_r, col0)) {
                    if jp_r != row1 { with_row(jp_raw, row1) } else { jp_raw.to_string() }
                } else {
                    // JP không có ô này → dùng VN
                    clone_cell(c, vn_sheet_xml, row1, xf_remap, vn_plain_ssi, vn_rich_ssi_raw)
                };
                Some((col0, cell_xml))
            })
            .collect();

        // Cells từ JP không có trong VN (bảo toàn các cột chỉ JP có)
        if let Some(jp_cols) = jp_row_col0s.get(&jp_r) {
            for &col0 in jp_cols {
                if !vn_col0s.contains(&col0) {
                    if let Some(raw) = jp_cell_lookup.get(&(jp_r, col0)) {
                        let cell_xml =
                            if jp_r != row1 { with_row(raw, row1) } else { raw.to_string() };
                        mixed_cells.push((col0, cell_xml));
                    }
                }
            }
        }

        mixed_cells.sort_unstable_by_key(|(col0, _)| *col0);
        let cells: String = mixed_cells.into_iter().map(|(_, xml)| xml).collect();

        new_sd.push_str(&format!(r#"<row r="{row1}"{row_attrs}>{cells}</row>"#));
        prev_was_inserted = false;
    }
    new_sd.push_str("</sheetData>");

    // ── Trích mergeCells của VN ──────────────────────────────────────────────
    let vn_merge_xml = vn_doc
        .descendants()
        .find(|n| n.tag_name().name() == "mergeCells")
        .map(|n| vn_sheet_xml[n.range()].to_string())
        .unwrap_or_default();

    // ── Surgery trên JP sheet XML (giữ drawing, cols, sheetView, ...) ────────
    let Ok(jp_doc_surg) = roxmltree::Document::parse(jp_sheet_xml) else {
        return (jp_sheet_xml.to_string(), inserted);
    };
    let Some(jp_sd_surg) = jp_doc_surg
        .descendants()
        .find(|n| n.tag_name().name() == "sheetData")
    else {
        return (jp_sheet_xml.to_string(), inserted);
    };

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    let jp_sd_end = jp_sd_surg.range().end;
    edits.push(SurgeryEdit {
        start: jp_sd_surg.range().start,
        end: jp_sd_surg.range().end,
        replacement: new_sd,
    });

    let jp_merge = jp_doc_surg
        .descendants()
        .find(|n| n.tag_name().name() == "mergeCells");
    match (jp_merge, vn_merge_xml.is_empty()) {
        (Some(jm), false) => edits.push(SurgeryEdit {
            start: jm.range().start,
            end: jm.range().end,
            replacement: vn_merge_xml,
        }),
        (Some(jm), true) => edits.push(SurgeryEdit {
            start: jm.range().start,
            end: jm.range().end,
            replacement: String::new(),
        }),
        (None, false) => edits.push(SurgeryEdit {
            start: jp_sd_end,
            end: jp_sd_end,
            replacement: vn_merge_xml,
        }),
        (None, true) => {}
    }

    (apply_surgery(jp_sheet_xml, edits), inserted)
}

/// Pipeline "Áp dụng" VN → JP cho C2.3.3. Xem module doc.
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

    let vn_changed_cells = find_changed_style_cells_xlsx(vn_path);
    let vn_strike_cells = find_strike_cells_xlsx(vn_path);

    // ── 5. Chuẩn bị vòng loop ─────────────────────────────────────────────────
    let cloned_names = &structure.cloned_names;
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
    let mut rows_inserted = 0usize;

    replaced.insert(
        "xl/styles.xml".to_string(),
        style_result.new_styles_xml.into_bytes(),
    );

    // ── 6. Vòng loop sheet-by-sheet ───────────────────────────────────────────
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

        let vn_changed = vn_changed_cells.get(sheet_name);
        let vn_strike = vn_strike_cells.get(sheet_name);

        let (new_sheet_xml, sheet_inserted) = clone_c233_sheet_for_jp(
            &vn_sheet_xml,
            &jp_sheet_xml,
            CONTENT_START_ROW1,
            &style_result.xf_remap,
            &vn_plain_ssi,
            &vn_rich_ssi,
            vn_changed,
            vn_strike,
        );

        rows_inserted += sheet_inserted;
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
        rows_inserted,
    })
}
