//! Method xử lý riêng cho C2.3.3 イベント詳細設計書.
//!
//! Cùng chiến lược với C2.3.2/C2.3.4/C2.3.6/C2.3.8: pipeline "Áp dụng" ở đây clone TOÀN BỘ nội
//! dung sheet VN vào JP output thay vì chỉ ghi đè ô đỏ — đòi hỏi toàn bộ nội dung + style của VN
//! phải được phản ánh chính xác vào kết quả (thêm dòng mới, đổi style cả dòng, đổi màu nền,
//! border...). C2.3.3 KHÔNG có cột STT tự đánh số (giống C2.3.2/C2.3.8) — cột A được xử lý như
//! mọi cột thường.
//!
//! Điểm khác biệt so với C2.3.2/C2.3.8: C2.3.3 cho phép VN thêm/xóa dòng, nên pipeline PHẢI canh
//! dòng VN↔JP theo nội dung cột A (`c233_align_rows`) thay vì so khớp thuần theo số dòng.
//! Cột A chứa tên action — ổn định giữa VN và JP, dùng làm anchor cho alignment (cột B thường
//! trống). JP text thường có furigana (katakana) nối đuôi (ví dụ "初期表示ショキヒョウジ") nên
//! cần strip trước khi so khớp.
//! Nhờ vậy ô JP được giữ lại tại ĐÚNG dòng gốc (dù VN đã chèn dòng lệch vị trí), và dòng VN-only
//! (mới thêm, không có ở JP) luôn dùng nội dung VN.
//!
//! LƯU Ý: file VN chứa bản dịch tiếng Việt — nội dung chữ đen ở VN KHÁC nội dung JP (khác ngôn
//! ngữ). Không được dùng so sánh giá trị (value diff) để đánh dấu "đã thay đổi" — nếu làm vậy sẽ
//! ghi đè gần như toàn bộ ô JP bằng tiếng Việt. Chỉ dùng style (chữ đỏ / strikethrough) để xác
//! định ô đã chỉnh sửa.
//!
//! Pipeline `apply_changes`:
//! 1. `sync_structure` — dọn dẹp JP, clone sheet chỉ có ở VN, đổi tên "(DEL)", sắp xếp sheet.
//! 2. Merge styles VN→JP.
//! 3. Quét ô VN "đã thay đổi" theo style (`find_changed_style_cells_xlsx`).
//! 4. Vòng loop qua từng sheet chung (VN ∩ JP − cloned − DEL):
//!    a. Canh dòng VN↔JP theo cột A (`c233_align_rows`, strip furigana JP + strip VN annotation).
//!    b. Bổ sung mọi ô ở dòng VN-only vào tập "đã thay đổi" (dòng mới, không có JP tương ứng).
//!    c. Clone toàn bộ sheet VN vào JP output (`clone_vn_sheet_for_jp`,
//!       `use_col_a_formula = false`, truyền `jp_to_vn_row` mapping):
//!       - Hàng header (row < 4): giữ nguyên (kể cả cột A).
//!       - Hàng nội dung (row ≥ 4): remap style, inline string; ô KHÔNG "đã thay đổi" VÀ có ô JP
//!         gốc tại dòng aligned → giữ nguyên ô JP.
//! 5. Ghi output.

use std::collections::{HashMap, HashSet};
use std::fs::File;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::vnjp_sync::ApplyResult;

use super::sync_service::{
    apply_surgery, clone_vn_cell_xml, clone_vn_row_xml, col_index_to_letter,
    extract_all_shared_strings, extract_col_texts, find_changed_style_cells_xlsx, is_del_sheet_name,
    merged_output_path, merge_vn_styles_into_jp, parse_cell_ref, read_zip_entry,
    resolve_sheet_xml_paths, sync_structure, write_output_zip, xml_escape_attr, ContentAlignment,
    ContentBounds, SurgeryEdit,
};

/// Nội dung cột A ~ AR (0-based 43).
pub fn content_bounds(sheet_name: &str) -> ContentBounds {
    super::sync_service::content_bounds_for_sheet(sheet_name, 43)
}

/// Row bắt đầu vùng nội dung (1-based) — bỏ vùng header cố định (row Excel 1~3).
const CONTENT_START_ROW1: usize = 4;

/// Cột dùng để canh dòng VN↔JP (0-based). Cột A (col0 = 0) chứa tên action — nội dung ổn định
/// giữa VN và JP, phù hợp làm "anchor" cho alignment. Cột B thường trống ở C2.3.3.
const ALIGN_COL0: usize = 0;

fn is_kanji(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c) || ('\u{3400}'..='\u{4DBF}').contains(&c)
}

fn is_latin_like(c: char) -> bool {
    c.is_ascii_alphabetic()
        || ('\u{00C0}'..='\u{024F}').contains(&c)
        || ('\u{1E00}'..='\u{1EFF}').contains(&c)
}

/// Strip furigana (katakana đọc) nối đuôi text JP. Heuristic: trailing katakana chỉ bị strip nếu
/// độ dài < kanji_count * 3 (mỗi kanji ~1-3 ký tự furigana). Nếu dài hơn thì katakana đó bao gồm
/// cả từ có nghĩa (ví dụ "クリア") nên giữ nguyên.
fn strip_jp_furigana(text: &str) -> String {
    let text = text.trim();
    if text.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    let kanji_count = chars.iter().filter(|&&c| is_kanji(c)).count();
    if kanji_count == 0 {
        return text.to_string();
    }
    let mut end = chars.len();
    while end > 0 && ('\u{30A0}'..='\u{30FF}').contains(&chars[end - 1]) {
        end -= 1;
    }
    let trailing_kata_len = chars.len() - end;
    if trailing_kata_len > 0 && end > 0 && trailing_kata_len < kanji_count * 3 {
        return chars[..end].iter().collect();
    }
    text.to_string()
}

/// Strip "[…]" prefix (tên sub-screen do VN thêm) và annotation tiếng Việt nối sau space.
fn extract_vn_core(text: &str) -> String {
    let text = text.trim();
    let text = if text.starts_with('[') {
        text.find(']')
            .map(|i| &text[i + ']'.len_utf8()..])
            .unwrap_or(text)
    } else {
        text
    };
    for (i, c) in text.char_indices() {
        if c == ' ' {
            if let Some(next) = text[i + 1..].chars().next() {
                if next.is_ascii_alphabetic() || is_latin_like(next) {
                    let result = text[..i].trim();
                    if !result.is_empty() {
                        return result.to_string();
                    }
                }
            }
        }
    }
    text.trim().to_string()
}

/// So khớp text cột A cho C2.3.3: dùng starts_with / contains nhưng kiểm tra length ratio (≥ 0.4)
/// để tránh false positive khi text JP quá ngắn so với VN (ví dụ "検索" khớp "検索画面表示").
fn c233_text_similarity(vn_core: &str, jp_core: &str) -> f64 {
    if vn_core.is_empty() || jp_core.is_empty() {
        return 0.0;
    }
    if vn_core == jp_core {
        return 1.0;
    }
    let vn_len = vn_core.chars().count();
    let jp_len = jp_core.chars().count();
    let shorter = vn_len.min(jp_len);
    let longer = vn_len.max(jp_len);
    let ratio = shorter as f64 / longer as f64;

    if vn_core.starts_with(jp_core) || jp_core.starts_with(vn_core) {
        return if ratio >= 0.4 { 1.0 } else { ratio };
    }
    if vn_core.contains(jp_core) || jp_core.contains(vn_core) {
        return if ratio >= 0.4 { 0.9 } else { ratio };
    }
    let common = vn_core
        .chars()
        .zip(jp_core.chars())
        .take_while(|(a, b)| a == b)
        .count();
    if longer == 0 {
        return 0.0;
    }
    let pfx = common as f64 / longer as f64;
    if common > 0 && jp_len > 0 && common * 2 >= jp_len {
        (pfx + 0.1).min(1.0)
    } else {
        pfx
    }
}

/// Canh dòng VN↔JP cho C2.3.3: preprocess VN (strip "[…]" prefix + Vietnamese annotation) và JP
/// (strip furigana), rồi greedy match bằng `c233_text_similarity`.
fn c233_align_rows(
    vn_col: &[(usize, String)],
    jp_col: &[(usize, String)],
) -> ContentAlignment {
    const THRESHOLD: f64 = 0.45;

    let vn_cores: Vec<(usize, String)> = vn_col
        .iter()
        .map(|(r, t)| (*r, extract_vn_core(t)))
        .collect();
    let jp_cores: Vec<(usize, String)> = jp_col
        .iter()
        .map(|(r, t)| (*r, strip_jp_furigana(t)))
        .collect();

    let mut matched: Vec<(usize, usize)> = Vec::new();
    let mut jp_used = vec![false; jp_cores.len()];
    let mut jp_start_idx = 0usize;

    for (vn_r, vn_text) in &vn_cores {
        let mut best_score = THRESHOLD;
        let mut best_ji: Option<usize> = None;
        for (ji, (_, jp_text)) in jp_cores.iter().enumerate().skip(jp_start_idx) {
            if jp_used[ji] {
                continue;
            }
            let score = c233_text_similarity(vn_text, jp_text);
            if score > best_score {
                best_score = score;
                best_ji = Some(ji);
                if score >= 0.95 {
                    break;
                }
            }
        }
        if let Some(ji) = best_ji {
            matched.push((*vn_r, jp_cores[ji].0));
            jp_used[ji] = true;
            jp_start_idx = ji;
        }
    }

    let matched_vn: HashSet<usize> = matched.iter().map(|(v, _)| *v).collect();
    let vn_only: Vec<usize> = vn_col
        .iter()
        .map(|(r, _)| *r)
        .filter(|r| !matched_vn.contains(r))
        .collect();

    ContentAlignment { matched, vn_only }
}

/// Trích danh sách row1 (1-based) trong vùng nội dung (>= `start_row1`) từ sheet XML.
fn extract_content_row_numbers(sheet_xml: &str, start_row1: usize) -> Vec<usize> {
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return Vec::new();
    };
    let Some(sd) = doc.descendants().find(|n| n.tag_name().name() == "sheetData") else {
        return Vec::new();
    };
    let mut rows: Vec<usize> = sd
        .children()
        .filter(|n| n.tag_name().name() == "row")
        .filter_map(|r| r.attribute("r")?.parse::<usize>().ok())
        .filter(|&r| r >= start_row1)
        .collect();
    rows.sort_unstable();
    rows
}

/// Trích row1 có ít nhất 1 cell chứa nội dung thực (value / inline string / formula).
/// Row chỉ có style (border, background) mà không có giá trị được coi là blank → không tiêu thụ
/// JP row trong gap mapping.
fn extract_rows_with_content(sheet_xml: &str, start_row1: usize) -> HashSet<usize> {
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return HashSet::new();
    };
    let Some(sd) = doc.descendants().find(|n| n.tag_name().name() == "sheetData") else {
        return HashSet::new();
    };
    let mut result = HashSet::new();
    for row in sd.children().filter(|n| n.tag_name().name() == "row") {
        let row1 = row
            .attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        if row1 < start_row1 {
            continue;
        }
        let has_content = row
            .children()
            .filter(|c| c.tag_name().name() == "c")
            .any(|c| {
                c.children().any(|child| {
                    let name = child.tag_name().name();
                    name == "v" || name == "is" || name == "f"
                })
            });
        if has_content {
            result.insert(row1);
        }
    }
    result
}

/// Xây mapping jp_row1 → vn_row1 đầy đủ cho MỌI dòng nội dung — không chỉ các dòng alignment
/// khớp được, mà cả các dòng giữa hai anchor bằng cách ghép tuần tự trong từng khoảng giữa hai
/// anchor liền kề. Dòng VN blank (không có cell nội dung) bị bỏ qua để tránh "tiêu thụ" JP row.
fn build_complete_row_mapping(
    alignment: &ContentAlignment,
    vn_content_rows: &[usize],
    jp_content_rows: &[usize],
    vn_rows_with_content: &HashSet<usize>,
) -> HashMap<usize, usize> {
    let vn_only_set: HashSet<usize> = alignment.vn_only.iter().copied().collect();
    let matched_vn: HashSet<usize> = alignment.matched.iter().map(|&(v, _)| v).collect();
    let matched_jp: HashSet<usize> = alignment.matched.iter().map(|&(_, j)| j).collect();

    let mut result: HashMap<usize, usize> = HashMap::new();

    for &(vn_r, jp_r) in &alignment.matched {
        result.insert(jp_r, vn_r);
    }

    let anchors = &alignment.matched;
    let vn_min = vn_content_rows.first().copied().unwrap_or(0);
    let vn_max = vn_content_rows.last().copied().unwrap_or(0);
    let jp_min = jp_content_rows.first().copied().unwrap_or(0);
    let jp_max = jp_content_rows.last().copied().unwrap_or(0);

    // (vn_lo_incl, vn_hi_excl, jp_lo_incl, jp_hi_excl)
    let mut segments: Vec<(usize, usize, usize, usize)> = Vec::new();
    if anchors.is_empty() {
        segments.push((vn_min, vn_max + 1, jp_min, jp_max + 1));
    } else {
        let (fv, fj) = anchors[0];
        segments.push((vn_min, fv, jp_min, fj));
        for w in anchors.windows(2) {
            segments.push((w[0].0 + 1, w[1].0, w[0].1 + 1, w[1].1));
        }
        let (lv, lj) = *anchors.last().unwrap();
        segments.push((lv + 1, vn_max + 1, lj + 1, jp_max + 1));
    }

    for &(vs, ve, js, je) in &segments {
        let gap_vn: Vec<usize> = vn_content_rows
            .iter()
            .filter(|&&r| {
                r >= vs
                    && r < ve
                    && !vn_only_set.contains(&r)
                    && !matched_vn.contains(&r)
                    && vn_rows_with_content.contains(&r)
            })
            .copied()
            .collect();
        let gap_jp: Vec<usize> = jp_content_rows
            .iter()
            .filter(|&&r| r >= js && r < je && !matched_jp.contains(&r))
            .copied()
            .collect();
        for (&jp_r, &vn_r) in gap_jp.iter().zip(gap_vn.iter()) {
            result.insert(jp_r, vn_r);
        }
    }

    result
}

/// Sửa cell reference `r="..."` trong raw JP cell XML để khớp row đích.
/// JP cell gốc có `r="B13"` nhưng khi đặt vào VN row 19 thì phải thành `r="B19"`.
fn fix_jp_cell_row(jp_cell_xml: &str, col0: usize, target_row1: usize) -> String {
    let new_ref = format!("{}{}", col_index_to_letter(col0), target_row1);
    let attr_start = if jp_cell_xml.starts_with("<c r=\"") {
        "<c r=\"".len()
    } else if let Some(pos) = jp_cell_xml.find(" r=\"") {
        pos + " r=\"".len()
    } else {
        return jp_cell_xml.to_string();
    };
    if let Some(end_quote) = jp_cell_xml[attr_start..].find('"') {
        let mut result = String::with_capacity(jp_cell_xml.len());
        result.push_str(&jp_cell_xml[..attr_start]);
        result.push_str(&new_ref);
        result.push_str(&jp_cell_xml[attr_start + end_quote..]);
        return result;
    }
    jp_cell_xml.to_string()
}

/// Clone toàn bộ sheet VN vào khung JP, dành riêng cho C2.3.3.
/// Khác `clone_vn_sheet_for_jp` dùng chung: hỗ trợ `jp_to_vn_row` mapping để xử lý VN chèn/xóa dòng.
fn c233_clone_vn_sheet_for_jp(
    vn_sheet_xml: &str,
    jp_sheet_xml: &str,
    xf_remap: &[usize],
    content_start_row1: usize,
    vn_plain_ssi: &HashMap<usize, String>,
    vn_rich_ssi_raw: &HashMap<usize, String>,
    vn_changed_positions: Option<&HashSet<(usize, usize)>>,
    jp_to_vn_row: &HashMap<usize, usize>,
) -> String {
    let Ok(vn_doc) = roxmltree::Document::parse(vn_sheet_xml) else {
        return jp_sheet_xml.to_string();
    };
    let Some(vn_sd) = vn_doc.descendants().find(|n| n.tag_name().name() == "sheetData") else {
        return jp_sheet_xml.to_string();
    };

    // Lookup ô JP theo (vn_row1, col0): key dùng vn_row1 (qua mapping) thay vì jp_row1 gốc để xử
    // lý trường hợp VN chèn/xóa dòng lệch vị trí. JP row không có trong mapping bị bỏ qua.
    let mut jp_cell_lookup: HashMap<(usize, usize), &str> = HashMap::new();
    if let Ok(jp_doc_lookup) = roxmltree::Document::parse(jp_sheet_xml) {
        if let Some(jp_sd_lookup) =
            jp_doc_lookup.descendants().find(|n| n.tag_name().name() == "sheetData")
        {
            for row in jp_sd_lookup.children().filter(|n| n.tag_name().name() == "row") {
                let jp_row1 = row
                    .attribute("r")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                if jp_row1 == 0 {
                    continue;
                }
                let lookup_row1 = match jp_to_vn_row.get(&jp_row1) {
                    Some(&vn_r) => vn_r,
                    None => continue,
                };
                for cell in row.children().filter(|n| n.tag_name().name() == "c") {
                    if let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) {
                        jp_cell_lookup.insert((lookup_row1, col0), &jp_sheet_xml[cell.range()]);
                    }
                }
            }
        }
    }

    let mut new_sd = String::from("<sheetData>");
    for row_node in vn_sd.children().filter(|n| n.tag_name().name() == "row") {
        let row1 = row_node
            .attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        if row1 == 0 {
            continue;
        }

        if row1 < content_start_row1 {
            new_sd.push_str(&clone_vn_row_xml(
                row_node,
                vn_sheet_xml,
                row1,
                xf_remap,
                vn_plain_ssi,
                vn_rich_ssi_raw,
            ));
        } else {
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
                    name => row_attrs
                        .push_str(&format!(" {}=\"{}\"", name, xml_escape_attr(a.value()))),
                }
            }

            let vn_row0 = row1 - 1;
            let cells: String = row_node
                .children()
                .filter(|c| c.tag_name().name() == "c")
                .filter_map(|c| {
                    let col0 = c.attribute("r").and_then(parse_cell_ref).map(|(_, col)| col)?;
                    let is_edited = vn_changed_positions
                        .map(|m| m.contains(&(vn_row0, col0)))
                        .unwrap_or(true);
                    if !is_edited {
                        if let Some(jp_raw) = jp_cell_lookup.get(&(row1, col0)) {
                            return Some(fix_jp_cell_row(jp_raw, col0, row1));
                        }
                    }
                    Some(clone_vn_cell_xml(
                        c,
                        vn_sheet_xml,
                        row1,
                        xf_remap,
                        vn_plain_ssi,
                        vn_rich_ssi_raw,
                    ))
                })
                .collect();

            new_sd.push_str(&format!(r#"<row r="{row1}"{row_attrs}>{cells}</row>"#));
        }
    }
    new_sd.push_str("</sheetData>");

    // Trích mergeCells của VN
    let vn_merge_xml = vn_doc
        .descendants()
        .find(|n| n.tag_name().name() == "mergeCells")
        .map(|n| vn_sheet_xml[n.range()].to_string())
        .unwrap_or_default();

    // Surgery trên JP sheet XML (giữ drawing, cols, sheetView, ...)
    let Ok(jp_doc) = roxmltree::Document::parse(jp_sheet_xml) else {
        return jp_sheet_xml.to_string();
    };
    let Some(jp_sd) = jp_doc.descendants().find(|n| n.tag_name().name() == "sheetData") else {
        return jp_sheet_xml.to_string();
    };

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    let jp_sd_end = jp_sd.range().end;
    edits.push(SurgeryEdit {
        start: jp_sd.range().start,
        end: jp_sd.range().end,
        replacement: new_sd,
    });

    let jp_merge = jp_doc
        .descendants()
        .find(|n| n.tag_name().name() == "mergeCells");
    match (jp_merge, vn_merge_xml.is_empty()) {
        (Some(jm), false) => {
            edits.push(SurgeryEdit {
                start: jm.range().start,
                end: jm.range().end,
                replacement: vn_merge_xml,
            });
        }
        (Some(jm), true) => {
            edits.push(SurgeryEdit {
                start: jm.range().start,
                end: jm.range().end,
                replacement: String::new(),
            });
        }
        (None, false) => {
            edits.push(SurgeryEdit {
                start: jp_sd_end,
                end: jp_sd_end,
                replacement: vn_merge_xml,
            });
        }
        (None, true) => {}
    }

    apply_surgery(jp_sheet_xml, edits)
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
    let jp_sst_xml =
        read_zip_entry(&mut jp_archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (jp_plain_ssi, _jp_rich_ssi) = extract_all_shared_strings(&jp_sst_xml);
    let jp_sheet_paths = resolve_sheet_xml_paths(&jp_wb_xml, &jp_rels_xml);
    let jp_sheet_map: HashMap<String, String> = jp_sheet_paths.into_iter().collect();

    // ── 4. Merge styles VN→JP ─────────────────────────────────────────────────
    let style_result = merge_vn_styles_into_jp(&jp_styles_xml, &vn_styles_xml);

    let vn_changed_cells = find_changed_style_cells_xlsx(vn_path);

    // ── 5. Vòng loop sheet-by-sheet ───────────────────────────────────────────
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

    replaced.insert(
        "xl/styles.xml".to_string(),
        style_result.new_styles_xml.into_bytes(),
    );

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

        // ── 5a. Canh dòng VN↔JP theo cột A (c233_align_rows xử lý strip furigana + VN core) ──
        let vn_col = extract_col_texts(&vn_sheet_xml, &vn_plain_ssi, ALIGN_COL0, CONTENT_START_ROW1);
        let jp_col = extract_col_texts(&jp_sheet_xml, &jp_plain_ssi, ALIGN_COL0, CONTENT_START_ROW1);
        let alignment = c233_align_rows(&vn_col, &jp_col);

        let vn_content_rows = extract_content_row_numbers(&vn_sheet_xml, CONTENT_START_ROW1);
        let jp_content_rows = extract_content_row_numbers(&jp_sheet_xml, CONTENT_START_ROW1);
        let vn_rows_with_content = extract_rows_with_content(&vn_sheet_xml, CONTENT_START_ROW1);
        let jp_to_vn = build_complete_row_mapping(
            &alignment,
            &vn_content_rows,
            &jp_content_rows,
            &vn_rows_with_content,
        );

        // ── 5b. Gom tập "đã thay đổi" = style changes + VN-only rows ────────
        let mut all_changed: HashSet<(usize, usize)> = vn_changed_cells
            .get(sheet_name)
            .cloned()
            .unwrap_or_default();

        for &vn_r1 in &alignment.vn_only {
            for col0 in 0..=43 {
                all_changed.insert((vn_r1 - 1, col0));
            }
        }

        // ── 5c. Clone VN → JP với row mapping ────────────────────────────────
        let new_sheet_xml = c233_clone_vn_sheet_for_jp(
            &vn_sheet_xml,
            &jp_sheet_xml,
            &style_result.xf_remap,
            CONTENT_START_ROW1,
            &vn_plain_ssi,
            &vn_rich_ssi,
            Some(&all_changed),
            &jp_to_vn,
        );

        applied_count += 1;
        replaced.insert(jp_xml_path, new_sheet_xml.into_bytes());
        if !sheets_modified.contains(sheet_name) {
            sheets_modified.push(sheet_name.clone());
        }
    }

    // ── 6. Ghi output ─────────────────────────────────────────────────────────
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
