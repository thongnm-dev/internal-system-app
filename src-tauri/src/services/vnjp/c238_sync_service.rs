//! Method xử lý riêng cho C2.3.8 画面間インタフェース仕様書.
//!
//! Đây là loại tài liệu DUY NHẤT có thuật toán canh dòng khác biệt (canh dòng THEO GROUP) —
//! xem ghi chú chi tiết ngay dưới. Các xử lý CẤP THẤP dùng chung (đọc file, quét ô đỏ/strikethrough,
//! dọn dẹp, ghi ô đỏ, xuất báo cáo...) nằm ở `super::sync_service`. `apply_changes` tự lắp ráp đủ
//! pipeline "Áp dụng" ở đây (chấp nhận lặp code giữa các loại tài liệu) — canh dòng vẫn tự động
//! dùng THEO GROUP cho loại tài liệu này vì bước đó gọi vào
//! `super::sync_service::analyze_row_alignment`, tự dispatch qua `is_screen_interface_doc`.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::vnjp_sync::{ApplyResult, RowAlignmentSuggestion};

use crate::models::vnjp_sync::CellDataMismatch;

use super::sync_service::{
    apply_replace_dictionary, build_replace_dictionary, clone_vn_sheet_for_jp,
    content_bounds_for_sheet, extract_all_shared_strings, find_changed_style_cells_xlsx,
    find_fully_struck_colored_cells_xlsx, is_del_sheet_name, merged_output_path,
    merge_vn_styles_into_jp, parse_cell_ref, read_zip_entry, resolve_sheet_xml_paths,
    sync_structure, verify_data_cells_between_files, write_output_zip,
};

// ─────────────────────────────────────────────────────────────────────────────
// Canh dòng THEO GROUP cho tài liệu "画面間インタフェース仕様書" (VN → JP)
//
// Loại tài liệu này KHÔNG có ô "neo" số/mã ở header (cột A là chữ Nhật/Việt), nên strategy neo
// chung không nhận diện được nhóm mới. Thay vào đó ta chia sheet thành các GROUP:
//   • header = dòng có NỀN VÀNG NHẠT (indexed 43) ở cột A; header chữ XANH = nhóm MỚI so với JP.
//   • chi tiết = các dòng ngay dưới header (tới header kế tiếp).
// Đối ứng group VN↔JP bằng "chữ ký neo" (screen ID / mã kỹ thuật KHÔNG dịch trong dòng chi tiết —
// dùng lại `is_anchor_cell`) để không phụ thuộc ngôn ngữ. Nhóm VN có header xanh VÀ chữ ký không
// giao với bất kỳ group JP nào ⇒ nhóm mới thật sự ⇒ đề xuất chèn (clone nguyên group từ VN).
// ─────────────────────────────────────────────────────────────────────────────

/// Prefix tên file của loại tài liệu áp dụng canh dòng theo group (nhận cả bản VN "..._VN.xlsx").
pub(crate) const SCREEN_IF_DOC_PREFIX: &str = "C2.3.8 画面間インタフェース仕様書";

/// Dòng dữ liệu bắt đầu (0-based) — bỏ vùng tiêu đề đầu (画面ID / 画面名称…). Group đầu tiên từ đây.
const SCREEN_IF_DATA_START_ROW0: usize = 3;

fn is_argb_yellow(argb: &str) -> bool {
    let Some((r, g, b)) = super::sync_service::parse_rgb_triplet(argb) else {
        return false;
    };
    r > 0xE0 && g > 0xD0 && b < 0xC0 && r >= g
}

/// Parse styles.xml → tập fillId có nền vàng nhạt (indexed 43 = FFFF99, hoặc rgb vàng nhạt).
fn parse_yellow_fill_ids(styles_xml: &str) -> HashSet<usize> {
    let mut result = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(styles_xml) else {
        return result;
    };
    let mut fill_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "fill"
            && node.parent().map(|p| p.tag_name().name()) == Some("fills")
        {
            let is_yellow = node.descendants().any(|c| {
                if c.tag_name().name() != "fgColor" {
                    return false;
                }
                if c.attribute("indexed").and_then(|s| s.parse::<usize>().ok()) == Some(43) {
                    return true;
                }
                c.attribute("rgb").map(is_argb_yellow).unwrap_or(false)
            });
            if is_yellow {
                result.insert(fill_idx);
            }
            fill_idx += 1;
        }
    }
    result
}

/// Parse styles.xml → tập cellXfs index có fill vàng (dòng header).
fn parse_yellow_xf_indices(styles_xml: &str, yellow_fills: &HashSet<usize>) -> HashSet<usize> {
    let mut result = HashSet::new();
    if yellow_fills.is_empty() {
        return result;
    }
    let Ok(doc) = roxmltree::Document::parse(styles_xml) else {
        return result;
    };
    let mut xf_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "xf"
            && node.parent().map(|p| p.tag_name().name()) == Some("cellXfs")
        {
            if let Some(fid) = node.attribute("fillId").and_then(|s| s.parse::<usize>().ok()) {
                if yellow_fills.contains(&fid) {
                    result.insert(xf_idx);
                }
            }
            xf_idx += 1;
        }
    }
    result
}

/// Parse styles.xml → tập cellXfs index dùng 1 trong các `font_ids` cho trước (dùng cho font xanh).
/// KHÔNG bắt buộc `applyFont` (các công cụ WPS/Excel đặt cờ này không nhất quán; ta lấy font đã
/// resolve của cell). Có xét kế thừa qua `xfId` → cellStyleXfs.
fn parse_font_xf_indices(styles_xml: &str, font_ids: &HashSet<usize>) -> HashSet<usize> {
    let mut result = HashSet::new();
    if font_ids.is_empty() {
        return result;
    }
    let Ok(doc) = roxmltree::Document::parse(styles_xml) else {
        return result;
    };

    let mut style_xf_fonts: Vec<usize> = Vec::new();
    for node in doc.descendants() {
        if node.tag_name().name() == "xf"
            && node.parent().map(|p| p.tag_name().name()) == Some("cellStyleXfs")
        {
            let fid = node
                .attribute("fontId")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            style_xf_fonts.push(fid);
        }
    }

    let mut xf_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "xf"
            && node.parent().map(|p| p.tag_name().name()) == Some("cellXfs")
        {
            let font_id = node
                .attribute("fontId")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let direct = font_ids.contains(&font_id);
            let inherited = node
                .attribute("xfId")
                .and_then(|s| s.parse::<usize>().ok())
                .and_then(|xfid| style_xf_fonts.get(xfid).copied())
                .map_or(false, |f| font_ids.contains(&f));
            if direct || inherited {
                result.insert(xf_idx);
            }
            xf_idx += 1;
        }
    }
    result
}

/// Kiểm tra một run `<r>` có font xanh trong `<rPr><color rgb=".."/>`.
fn has_blue_font_run(run_node: &roxmltree::Node) -> bool {
    run_node.descendants().any(|child| {
        child.tag_name().name() == "color"
            && child.attribute("rgb").map(super::sync_service::is_argb_blue).unwrap_or(false)
    })
}

/// shared strings có ÍT NHẤT 1 run màu xanh (dùng nhận header nhóm mới khi header là shared rich-text).
fn parse_blue_shared_strings(sst_xml: &str) -> HashSet<usize> {
    let mut result = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sst_xml) else {
        return result;
    };
    let mut si_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "si" {
            let has_blue = node
                .children()
                .filter(|c| c.tag_name().name() == "r")
                .any(|r| has_blue_font_run(&r));
            if has_blue {
                result.insert(si_idx);
            }
            si_idx += 1;
        }
    }
    result
}

/// Với mỗi sheet: tập dòng (0-based) có NỀN VÀNG (header) và tập dòng có CHỮ XANH ở cột A.
fn find_group_style_rows(path: &str) -> HashMap<String, (HashSet<usize>, HashSet<usize>)> {
    let mut result = HashMap::new();
    let Ok(file) = File::open(path) else {
        return result;
    };
    let Ok(mut archive) = zip::ZipArchive::new(file) else {
        return result;
    };

    let styles_xml = super::sync_service::read_zip_entry(&mut archive, "xl/styles.xml").unwrap_or_default();
    let yellow_xf = parse_yellow_xf_indices(&styles_xml, &parse_yellow_fill_ids(&styles_xml));
    let blue_xf = parse_font_xf_indices(&styles_xml, &super::sync_service::parse_blue_font_ids(&styles_xml));

    let sst_xml = super::sync_service::read_zip_entry(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (blue_ssi, all_rich_ssi) = if sst_xml.is_empty() {
        (HashSet::new(), HashSet::new())
    } else {
        (
            parse_blue_shared_strings(&sst_xml),
            super::sync_service::parse_shared_strings_rich_info(&sst_xml).1,
        )
    };

    let Some(workbook_xml) = super::sync_service::read_zip_entry(&mut archive, "xl/workbook.xml") else {
        return result;
    };
    let Some(rels_xml) = super::sync_service::read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels") else {
        return result;
    };
    for (name, xml_path) in super::sync_service::resolve_sheet_xml_paths(&workbook_xml, &rels_xml) {
        if let Some(sheet_xml) = super::sync_service::read_zip_entry(&mut archive, &xml_path) {
            let flags =
                scan_col_a_style_rows(&sheet_xml, &yellow_xf, &blue_xf, &blue_ssi, &all_rich_ssi);
            result.insert(name, flags);
        }
    }
    result
}

/// Quét cột A của sheet: trả về (dòng header nền vàng, dòng chữ xanh) — cùng thứ tự ưu tiên
/// rich-text như `find_red_cells_in_sheet`.
fn scan_col_a_style_rows(
    sheet_xml: &str,
    yellow_xf: &HashSet<usize>,
    blue_xf: &HashSet<usize>,
    blue_ssi: &HashSet<usize>,
    all_rich_ssi: &HashSet<usize>,
) -> (HashSet<usize>, HashSet<usize>) {
    let mut headers = HashSet::new();
    let mut blues = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return (headers, blues);
    };

    for node in doc.descendants() {
        if node.tag_name().name() != "c" {
            continue;
        }
        let Some((row0, col0)) = node.attribute("r").and_then(super::sync_service::parse_cell_ref) else {
            continue;
        };
        if col0 != 0 {
            continue; // chỉ xét cột A
        }
        let s_idx = node.attribute("s").and_then(|s| s.parse::<usize>().ok());
        if let Some(si) = s_idx {
            if yellow_xf.contains(&si) {
                headers.insert(row0);
            }
        }

        let mut handled = false;
        if node.attribute("t") == Some("s") {
            if let Some(v) = node.descendants().find(|c| c.tag_name().name() == "v") {
                if let Some(ssi) = v.text().and_then(|t| t.parse::<usize>().ok()) {
                    if all_rich_ssi.contains(&ssi) {
                        handled = true;
                        if blue_ssi.contains(&ssi) {
                            blues.insert(row0);
                        }
                    }
                }
            }
        }
        if !handled && node.attribute("t") == Some("inlineStr") {
            if let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") {
                let runs: Vec<_> = is_node
                    .children()
                    .filter(|c| c.tag_name().name() == "r")
                    .collect();
                if !runs.is_empty() {
                    handled = true;
                    if runs.iter().any(|r| has_blue_font_run(r)) {
                        blues.insert(row0);
                    }
                }
            }
        }
        if !handled {
            if let Some(si) = s_idx {
                if blue_xf.contains(&si) {
                    blues.insert(row0);
                }
            }
        }
    }
    (headers, blues)
}

/// 1 group trong tài liệu 画面間インタフェース: header + các dòng chi tiết bên dưới.
struct DocGroup {
    header_row0: usize,
    end_row0: usize,
    is_blue: bool,
    header_text: String,
    /// Chữ ký neo (screen ID / mã kỹ thuật KHÔNG dịch) rút từ các dòng chi tiết — dùng đối ứng VN↔JP.
    sig: BTreeSet<String>,
}

fn last_content_row0(grid: &[Vec<String>]) -> Option<usize> {
    (0..grid.len())
        .rev()
        .find(|&r| grid[r].iter().any(|c| !c.trim().is_empty()))
}

/// Chia grid thành các group theo dòng header (nền vàng). Chữ ký lấy từ ô neo của dòng chi tiết.
fn build_doc_groups(
    grid: &[Vec<String>],
    header_rows: &HashSet<usize>,
    blue_rows: &HashSet<usize>,
    last_col0: usize,
) -> Vec<DocGroup> {
    let Some(last) = last_content_row0(grid) else {
        return Vec::new();
    };
    let mut headers: Vec<usize> = header_rows
        .iter()
        .copied()
        .filter(|&r| r >= SCREEN_IF_DATA_START_ROW0 && r <= last)
        .collect();
    headers.sort_unstable();

    let mut groups = Vec::new();
    for (i, &h) in headers.iter().enumerate() {
        let end = headers.get(i + 1).map(|&n| n - 1).unwrap_or(last);
        let mut sig = BTreeSet::new();
        for r in (h + 1)..=end {
            if let Some(row) = grid.get(r) {
                for cell in row.iter().take(last_col0 + 1) {
                    let t = cell.trim();
                    if super::sync_service::is_anchor_cell(t) {
                        sig.insert(t.to_string());
                    }
                }
            }
        }
        let header_text = grid
            .get(h)
            .and_then(|r| r.first())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        groups.push(DocGroup {
            header_row0: h,
            end_row0: end,
            is_blue: blue_rows.contains(&h),
            header_text,
            sig,
        });
    }
    groups
}

/// Loại các token neo XUẤT HIỆN Ở QUÁ NỬA số group (vd screen ID nguồn cố định như MJDL020 có mặt
/// gần như mọi dòng) — nếu giữ lại thì mọi group đều "giao" nhau, làm hỏng đối ứng.
fn strip_ubiquitous_tokens(vn: &mut [DocGroup], jp: &mut [DocGroup]) {
    let mut ubiq: HashSet<String> = HashSet::new();
    for groups in [&*vn, &*jp] {
        if groups.is_empty() {
            continue;
        }
        let mut count: HashMap<&str, usize> = HashMap::new();
        for g in groups.iter() {
            for t in &g.sig {
                *count.entry(t.as_str()).or_default() += 1;
            }
        }
        let threshold = groups.len() / 2;
        for (t, c) in count {
            if c > threshold {
                ubiq.insert(t.to_string());
            }
        }
    }
    for g in vn.iter_mut().chain(jp.iter_mut()) {
        g.sig.retain(|t| !ubiq.contains(t));
    }
}

fn sigs_intersect(a: &BTreeSet<String>, b: &BTreeSet<String>) -> bool {
    a.iter().any(|t| b.contains(t))
}

/// LCS đối ứng các group VN (chỉ những index trong `vn_idx`) với group JP theo tiêu chí "chữ ký có
/// giao nhau" — trả về map vn_group_index → jp_group_index.
fn lcs_align_groups(
    vn: &[DocGroup],
    jp: &[DocGroup],
    vn_idx: &[usize],
) -> HashMap<usize, usize> {
    let n = vn_idx.len();
    let m = jp.len();
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if sigs_intersect(&vn[vn_idx[i]].sig, &jp[j].sig) {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut pairs = HashMap::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if sigs_intersect(&vn[vn_idx[i]].sig, &jp[j].sig) {
            pairs.insert(vn_idx[i], j);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs
}

/// Canh dòng theo group: mỗi nhóm VN header xanh + chữ ký không có ở JP ⇒ đề xuất chèn (clone nguyên
/// group). `jp_insert_after_row` = dòng JP cuối của group JP đối ứng ngay TRƯỚC nhóm mới.
pub(crate) fn analyze_row_alignment_by_group(
    vn_path: &str,
    jp_path: &str,
) -> AppResult<Vec<RowAlignmentSuggestion>> {
    let vn_grid = super::sync_service::read_workbook_grid(vn_path)?;
    let jp_grid = super::sync_service::read_workbook_grid(jp_path)?;
    let vn_flags = find_group_style_rows(vn_path);
    let jp_flags = find_group_style_rows(jp_path);
    let empty = (HashSet::new(), HashSet::new());

    let mut suggestions = Vec::new();

    for (sheet, vn_rows) in &vn_grid {
        let Some(jp_rows) = jp_grid.get(sheet) else {
            continue;
        };
        let (vn_h, vn_b) = vn_flags.get(sheet).unwrap_or(&empty);
        let (jp_h, _jp_b) = jp_flags.get(sheet).unwrap_or(&empty);
        // Method xử lý riêng C2.3.8 (đây là loại tài liệu duy nhất dùng canh dòng theo group).
        let last_col0 = 10usize; // C2.3.8 nội dung cột A~K

        let mut vn_groups = build_doc_groups(vn_rows, vn_h, vn_b, last_col0);
        let mut jp_groups = build_doc_groups(jp_rows, jp_h, &HashSet::new(), last_col0);
        if vn_groups.is_empty() || jp_groups.is_empty() {
            continue;
        }

        strip_ubiquitous_tokens(&mut vn_groups, &mut jp_groups);

        let jp_tokens: HashSet<&str> = jp_groups
            .iter()
            .flat_map(|g| g.sig.iter().map(|s| s.as_str()))
            .collect();

        // Nhóm mới = header xanh VÀ không token neo nào trùng bất kỳ group JP.
        let is_new: Vec<bool> = vn_groups
            .iter()
            .map(|g| g.is_blue && g.sig.iter().all(|t| !jp_tokens.contains(t.as_str())))
            .collect();

        let non_new_idx: Vec<usize> = (0..vn_groups.len()).filter(|&i| !is_new[i]).collect();
        let pairs = lcs_align_groups(&vn_groups, &jp_groups, &non_new_idx);

        for (i, g) in vn_groups.iter().enumerate() {
            if !is_new[i] {
                continue;
            }
            // Group JP đối ứng của nhóm NON-NEW gần nhất phía trước → chèn ngay sau dòng cuối của nó.
            let mut jp_insert_after_row = jp_groups
                .first()
                .map(|fg| fg.header_row0) // đầu sheet: chèn ngay TRƯỚC group JP đầu tiên
                .unwrap_or(SCREEN_IF_DATA_START_ROW0);
            for k in (0..i).rev() {
                if let Some(&jp_idx) = pairs.get(&k) {
                    jp_insert_after_row = jp_groups[jp_idx].end_row0 + 1; // 0-based end → 1-based row
                    break;
                }
            }

            suggestions.push(RowAlignmentSuggestion {
                sheet: sheet.clone(),
                jp_insert_after_row,
                insert_count: g.end_row0 - g.header_row0 + 1,
                vn_row_start: g.header_row0 + 1,
                vn_row_end: g.end_row0 + 1,
                sample_vn_text: vec![g.header_text.clone()],
                has_red: true,
                has_strike: false,
            });
        }
    }

    // Thứ tự ổn định để UI hiển thị nhất quán.
    suggestions.sort_by(|a, b| {
        a.sheet
            .cmp(&b.sheet)
            .then(a.jp_insert_after_row.cmp(&b.jp_insert_after_row))
    });
    Ok(suggestions)
}


/// Dòng dữ liệu bắt đầu (1-based) — bỏ vùng header cố định (row Excel 1~3), khớp
/// `SCREEN_IF_DATA_START_ROW0 + 1`.
const CONTENT_START_ROW1: usize = SCREEN_IF_DATA_START_ROW0 + 1;

/// Các ô header ROW 3 luôn lấy từ JP: A3, C3, E3, F3 — (row1, col0).
fn jp_preserved_header_cells() -> HashSet<(usize, usize)> {
    [
        (3, 0), // A3
        (3, 2), // C3
        (3, 4), // E3
        (3, 5), // F3
    ]
    .into_iter()
    .collect()
}

/// Trích raw cell XML tại các vị trí `positions` từ sheet JP bất kỳ (sheet đầu tiên đọc được).
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

/// Pipeline "Áp dụng" VN → JP cho C2.3.8.
///
/// C2.3.8 KHÔNG có cột STT → không cần công thức cột A (`use_col_a_formula = false`, cột A xử lý
/// như cột thường). Dùng cùng chiến lược với C2.3.4: clone TOÀN BỘ nội dung VN vào JP (remap
/// style, inline string), giữ JP làm khung (drawing, sheetView, cols, page setup) để shapes
/// header không bị mất; riêng từng ô ở dòng dữ liệu — nếu ô đó KHÔNG "coi như đã thay đổi" (chữ
/// đen, không strikethrough — xem `find_changed_style_cells_xlsx`) VÀ JP đã có ô tại đúng vị trí
/// đó, GIỮ NGUYÊN ô JP thay vì ghi đè bằng VN.
///
/// Header row 3: các ô A3, C3, E3, F3 luôn lấy từ JP (`jp_preserved_header_cells`).
/// Sheet clone trực tiếp từ VN: cũng ghi đè header cells bằng nội dung từ sheet JP bất kỳ.
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
    let vn_sst_xml = read_zip_entry(&mut vn_archive, "xl/sharedStrings.xml").unwrap_or_default();
    let vn_styles_xml = read_zip_entry(&mut vn_archive, "xl/styles.xml").unwrap_or_default();
    let (vn_plain_ssi, vn_rich_ssi) = extract_all_shared_strings(&vn_sst_xml);
    let vn_sheet_map: HashMap<String, String> =
        resolve_sheet_xml_paths(&vn_wb_xml, &vn_rels_xml).into_iter().collect();

    // ── 3. Mở JP output zip (sau sync_structure) ──────────────────────────────
    let jp_file = File::open(&output_path_str)
        .map_err(|e| AppError::new(format!("Không mở được file JP output: {e}")))?;
    let mut jp_archive = zip::ZipArchive::new(jp_file)
        .map_err(|e| AppError::new(format!("File JP output không phải ZIP hợp lệ: {e}")))?;

    let jp_wb_xml = read_zip_entry(&mut jp_archive, "xl/workbook.xml").unwrap_or_default();
    let jp_rels_xml =
        read_zip_entry(&mut jp_archive, "xl/_rels/workbook.xml.rels").unwrap_or_default();
    let jp_styles_xml = read_zip_entry(&mut jp_archive, "xl/styles.xml").unwrap_or_default();
    let jp_sheet_map: HashMap<String, String> =
        resolve_sheet_xml_paths(&jp_wb_xml, &jp_rels_xml).into_iter().collect();

    // ── 4. Merge styles VN→JP ─────────────────────────────────────────────────
    let style_result = merge_vn_styles_into_jp(&jp_styles_xml, &vn_styles_xml);

    // Ô VN "coi như đã thay đổi" (strikethrough hoặc màu chữ không phải đen) theo từng sheet — ô
    // KHÔNG nằm trong set này (chữ đen, không strikethrough) sẽ được giữ nguyên bản JP tại đúng
    // vị trí khi clone (xem `clone_vn_sheet_for_jp`).
    let vn_changed_cells = find_changed_style_cells_xlsx(vn_path);
    let vn_fully_struck = find_fully_struck_colored_cells_xlsx(vn_path);

    // ── 5. Sheet chung cần xử lý ──────────────────────────────────────────────
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

    let preserved_cells = jp_preserved_header_cells();

    let mut replaced: HashMap<String, Vec<u8>> = HashMap::new();
    let mut applied_count = 0usize;
    let mut sheets_modified: Vec<String> = structure.sheets_modified.clone();

    replaced.insert(
        "xl/styles.xml".to_string(),
        style_result.new_styles_xml.into_bytes(),
    );

    // ── 6. Clone toàn bộ VN → JP (không có công thức cột A) ──────────────────
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

        // Tập ô "đã thay đổi" hiệu dụng = changed − fully_struck_colored.
        let effective_changed: Option<HashSet<(usize, usize)>> =
            vn_changed_cells.get(sheet_name).map(|changed| {
                match vn_fully_struck.get(sheet_name) {
                    Some(struck) => changed.difference(struck).copied().collect(),
                    None => changed.clone(),
                }
            });

        // use_col_a_formula = false → cột A xử lý như cột thường (không có công thức JP);
        // effective_changed → ô chữ đen, không strikethrough (không đổi) giữ nguyên bản JP cùng
        // vị trí; ô fully-struck-colored cũng giữ JP nhưng style VN (strike + color).
        // A3, C3, E3, F3 luôn lấy từ JP (jp_preserved_header_cells).
        let new_sheet_xml = clone_vn_sheet_for_jp(
            &vn_sheet_xml,
            &jp_sheet_xml,
            "", // không dùng vì use_col_a_formula = false
            0,  // không dùng vì use_col_a_formula = false
            CONTENT_START_ROW1,
            &style_result.xf_remap,
            &vn_plain_ssi,
            &vn_rich_ssi,
            effective_changed.as_ref(),
            false,
            Some(&preserved_cells),
        );

        applied_count += 1;
        replaced.insert(jp_xml_path, new_sheet_xml.into_bytes());
        if !sheets_modified.contains(sheet_name) {
            sheets_modified.push(sheet_name.clone());
        }
    }

    // ── 6b. Cloned sheets: ghi đè header cells JP ──────────────────────────
    // Sheet clone trực tiếp từ VN không có bản JP tương ứng — lấy header cells từ sheet JP bất kỳ.
    if !cloned_names.is_empty() {
        let jp_header_cells =
            extract_jp_header_cells(&mut jp_archive, &jp_sheet_map, &preserved_cells);
        if !jp_header_cells.is_empty() {
            for cloned_name in cloned_names {
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
                let patched = patch_header_cells_in_sheet(&cloned_xml, &jp_header_cells);
                replaced.insert(cloned_xml_path, patched.into_bytes());
            }
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

/// Kiểm tra output sau chuẩn hoá: so sánh sự có mặt của dữ liệu (có/không) tại từng ô
/// giữa file VN và file output — không so sánh nội dung, chỉ kiểm tra cell có hoặc không
/// dữ liệu tại cùng vị trí.
pub fn verify_output(vn_path: &str, output_path: &str) -> AppResult<Vec<CellDataMismatch>> {
    verify_data_cells_between_files(vn_path, output_path, |sheet_name| {
        let bounds = content_bounds_for_sheet(sheet_name, 10);
        (CONTENT_START_ROW1, bounds.last_col0)
    })
}

/// Thu thập từ điển replace (vn_text → jp_text) từ các ô mà output đã giữ nội dung JP.
pub fn build_dictionary(
    vn_path: &str,
    output_path: &str,
) -> AppResult<HashMap<String, String>> {
    build_replace_dictionary(vn_path, output_path, |sheet_name| {
        let bounds = content_bounds_for_sheet(sheet_name, 10);
        (CONTENT_START_ROW1, bounds.last_col0)
    })
}

/// Áp dụng từ điển replace lên file — chỉ thay thế khi nội dung cell khớp chính xác.
pub fn apply_dictionary(
    file_path: &str,
    output_path: &str,
    dictionary: &HashMap<String, String>,
) -> AppResult<usize> {
    apply_replace_dictionary(file_path, output_path, dictionary, |sheet_name| {
        let bounds = content_bounds_for_sheet(sheet_name, 10);
        (CONTENT_START_ROW1, bounds.last_col0)
    })
}
