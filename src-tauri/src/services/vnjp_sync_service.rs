//! Service đồng bộ tài liệu thiết kế chi tiết VN → JP.
//!
//! Phân tích sự khác biệt giữa file Excel VN (ô đỏ = nội dung mới cần dịch)
//! và file Excel JP (ô có strikethrough = nội dung cần xóa).
//! Hỗ trợ dịch tự động qua AI (Gemini / Groq) và xuất báo cáo Excel.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use calamine::{open_workbook_auto, Data, Reader};
use regex::Regex;
use reqwest::Client;
use rust_xlsxwriter::{Color, Format, FormatBorder, Workbook};
use serde_json::{json, Value};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::vnjp_sync::*;
use crate::utils::app_config;

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Phân tích 2 file Excel VN + JP, trả về SyncAnalysis đầy đủ.
pub fn analyze(vn_path: &str, jp_path: &str) -> AppResult<SyncAnalysis> {
    // --- Kiểm tra file tồn tại ---
    for p in [vn_path, jp_path] {
        if !Path::new(p).exists() {
            return Err(AppError::new(format!("Không tìm thấy file: {p}")));
        }
        let ext = Path::new(p)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "xlsx" && ext != "xlsm" {
            return Err(AppError::new(format!(
                "Chỉ hỗ trợ file .xlsx / .xlsm. File không hợp lệ: {p}"
            )));
        }
    }

    // --- Đọc lưới dữ liệu ---
    let vn_grid = read_workbook_grid(vn_path)?;
    let jp_grid = read_workbook_grid(jp_path)?;

    // --- Tìm ô đỏ (VN) và ô strikethrough (JP) ---
    let vn_red = find_red_cells_xlsx(vn_path);
    let jp_strike = find_strike_cells_xlsx(jp_path);

    // --- Màu tab sheet ---
    let vn_tab_colors = get_sheet_tab_colors(vn_path);
    let jp_tab_colors = get_sheet_tab_colors(jp_path);

    // --- Xây dựng SheetMeta VN ---
    let vn_sheets: Vec<SheetMeta> = vn_grid
        .iter()
        .map(|(name, grid)| {
            let row_count = grid.len();
            let col_count = grid.iter().map(|r| r.len()).max().unwrap_or(0);
            let red_set = vn_red.get(name);
            let red_cell_count = red_set.map(|s| s.len()).unwrap_or(0);
            SheetMeta {
                name: name.clone(),
                tab_color: vn_tab_colors.get(name).cloned(),
                row_count,
                col_count,
                red_cell_count,
                strike_cell_count: 0,
            }
        })
        .collect();

    // --- Xây dựng SheetMeta JP ---
    let jp_sheets: Vec<SheetMeta> = jp_grid
        .iter()
        .map(|(name, grid)| {
            let row_count = grid.len();
            let col_count = grid.iter().map(|r| r.len()).max().unwrap_or(0);
            let strike_set = jp_strike.get(name);
            let strike_cell_count = strike_set.map(|s| s.len()).unwrap_or(0);
            SheetMeta {
                name: name.clone(),
                tab_color: jp_tab_colors.get(name).cloned(),
                row_count,
                col_count,
                red_cell_count: 0,
                strike_cell_count,
            }
        })
        .collect();

    // --- Xây dựng SheetCompare (union tên sheet VN + JP) ---
    let mut all_sheet_names: Vec<String> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for name in vn_grid.keys().chain(jp_grid.keys()) {
        if seen.insert(name.clone()) {
            all_sheet_names.push(name.clone());
        }
    }

    let sheet_compare: Vec<SheetCompare> = all_sheet_names
        .iter()
        .map(|name| {
            let in_vn = vn_grid.contains_key(name);
            let in_jp = jp_grid.contains_key(name);
            let vn_rows = vn_grid.get(name).map(|g| g.len()).unwrap_or(0);
            let jp_rows = jp_grid.get(name).map(|g| g.len()).unwrap_or(0);
            let vn_red_count = vn_red.get(name).map(|s| s.len()).unwrap_or(0);
            let jp_strike_count = jp_strike.get(name).map(|s| s.len()).unwrap_or(0);
            SheetCompare {
                name: name.clone(),
                in_vn,
                in_jp,
                vn_tab_color: vn_tab_colors.get(name).cloned(),
                jp_tab_color: jp_tab_colors.get(name).cloned(),
                vn_rows,
                jp_rows,
                vn_red_count,
                jp_strike_count,
            }
        })
        .collect();

    // --- Xây dựng RedCell ---
    let mut red_cells: Vec<RedCell> = Vec::new();
    for (sheet_name, red_set) in &vn_red {
        let vn_sheet_grid = match vn_grid.get(sheet_name) {
            Some(g) => g,
            None => continue,
        };
        let jp_sheet_grid = jp_grid.get(sheet_name);

        // Sắp xếp để kết quả ổn định (row asc, col asc)
        let mut sorted_cells: Vec<(usize, usize)> = red_set.iter().copied().collect();
        sorted_cells.sort();

        for (r, c) in sorted_cells {
            let vn_text = vn_sheet_grid
                .get(r)
                .and_then(|row| row.get(c))
                .cloned()
                .unwrap_or_default();
            if vn_text.trim().is_empty() {
                continue;
            }
            let jp_text = jp_sheet_grid
                .and_then(|g| g.get(r))
                .and_then(|row| row.get(c))
                .cloned()
                .unwrap_or_default();
            // Chuyển sang 1-based cho UI
            red_cells.push(RedCell {
                sheet: sheet_name.clone(),
                row: r + 1,
                col: c + 1,
                vn_text,
                jp_text,
                translation: None,
            });
        }
    }

    // --- Xây dựng StrikeCell ---
    let mut strike_cells: Vec<StrikeCell> = Vec::new();
    for (sheet_name, strike_set) in &jp_strike {
        let jp_sheet_grid = match jp_grid.get(sheet_name) {
            Some(g) => g,
            None => continue,
        };

        let mut sorted_cells: Vec<(usize, usize)> = strike_set.iter().copied().collect();
        sorted_cells.sort();

        for (r, c) in sorted_cells {
            let text = jp_sheet_grid
                .get(r)
                .and_then(|row| row.get(c))
                .cloned()
                .unwrap_or_default();
            if text.trim().is_empty() {
                continue;
            }
            strike_cells.push(StrikeCell {
                sheet: sheet_name.clone(),
                row: r + 1,
                col: c + 1,
                text,
            });
        }
    }

    // --- Kiểm tra chất lượng (JP) ---
    let quality_issues = check_quality(&jp_grid, &jp_sheets);

    Ok(SyncAnalysis {
        vn_path: vn_path.to_string(),
        jp_path: jp_path.to_string(),
        vn_sheets,
        jp_sheets,
        sheet_compare,
        red_cells,
        strike_cells,
        quality_issues,
    })
}

/// Dịch hàng loạt các đoạn văn VN → JP qua AI API (Gemini hoặc Groq).
pub async fn translate_batch(
    request: TranslateBatchRequest,
) -> AppResult<Vec<TranslateItemResult>> {
    let provider = request.provider.trim().to_lowercase();
    let api_key = resolve_api_key(&provider, request.api_key.as_deref())?;
    let client = build_http_client()?;

    let mut results: Vec<TranslateItemResult> = Vec::new();

    for item in &request.items {
        let prompt = format!(
            "あなたはプロの技術文書翻訳者です。以下のベトナム語テキストを日本語に翻訳してください。\
これは技術設計仕様書のテキストです。翻訳文のみ出力し、説明は不要です。\n\nベトナム語テキスト: {}",
            item.text
        );

        let result = match provider.as_str() {
            "gemini" => {
                call_gemini(&client, &api_key, &request.model, &prompt).await
            }
            "groq" => {
                call_groq(&client, &api_key, &request.model, &prompt).await
            }
            other => Err(AppError::new(format!(
                "Nhà cung cấp không được hỗ trợ: '{other}'."
            ))),
        };

        match result {
            Ok(translation) => {
                results.push(TranslateItemResult {
                    id: item.id.clone(),
                    translation,
                    error: None,
                });
            }
            Err(e) => {
                results.push(TranslateItemResult {
                    id: item.id.clone(),
                    translation: String::new(),
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

/// Xuất báo cáo phân tích ra file Excel (.xlsx).
pub fn export_report(analysis: &SyncAnalysis, output_path: &str) -> AppResult<String> {
    let mut workbook = Workbook::new();

    write_summary_sheet(&mut workbook, analysis)?;
    write_red_cells_sheet(&mut workbook, analysis)?;
    write_strike_cells_sheet(&mut workbook, analysis)?;
    write_quality_sheet(&mut workbook, analysis)?;

    workbook
        .save(output_path)
        .map_err(|e| AppError::new(format!("Không lưu được file báo cáo: {e}")))?;

    Ok(output_path.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Đọc workbook → lưới dữ liệu
// ─────────────────────────────────────────────────────────────────────────────

fn read_workbook_grid(path: &str) -> AppResult<HashMap<String, Vec<Vec<String>>>> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| AppError::new(format!("Không mở được Excel {path}: {e}")))?;
    let names: Vec<String> = workbook.sheet_names().to_vec();
    let mut result: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for name in names {
        let Ok(range) = workbook.worksheet_range(&name) else {
            continue;
        };
        let mut grid: Vec<Vec<String>> = Vec::with_capacity(range.height());
        for row in range.rows() {
            grid.push(row.iter().map(cell_data_to_string).collect());
        }
        result.insert(name, grid);
    }
    Ok(result)
}

fn cell_data_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => trim_float(*f),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => d.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERR:{e:?}"),
    }
}

fn trim_float(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        f.to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tìm ô đỏ (font color đỏ) trong file xlsx
// ─────────────────────────────────────────────────────────────────────────────

fn find_red_cells_xlsx(path: &str) -> HashMap<String, HashSet<(usize, usize)>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return HashMap::new(),
    };

    // Tìm fontId có font màu đỏ trong styles.xml
    let red_font_ids: HashSet<usize> = read_zip_entry(&mut archive, "xl/styles.xml")
        .map(|xml| parse_red_font_ids(&xml))
        .unwrap_or_default();

    // Tìm xf index có fontId là đỏ trong cellXfs
    let red_xf_indices: HashSet<usize> = read_zip_entry(&mut archive, "xl/styles.xml")
        .map(|xml| parse_red_xf_indices(&xml, &red_font_ids))
        .unwrap_or_default();

    // Tìm shared string index có rich-text với font đỏ
    let red_ssi: HashSet<usize> = read_zip_entry(&mut archive, "xl/sharedStrings.xml")
        .map(|xml| parse_red_shared_strings(&xml))
        .unwrap_or_default();

    if red_xf_indices.is_empty() && red_ssi.is_empty() {
        return HashMap::new();
    }

    let workbook_xml = match read_zip_entry(&mut archive, "xl/workbook.xml") {
        Some(s) => s,
        None => return HashMap::new(),
    };
    let rels_xml = match read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels") {
        Some(s) => s,
        None => return HashMap::new(),
    };

    let sheet_paths = resolve_sheet_xml_paths(&workbook_xml, &rels_xml);

    let mut result: HashMap<String, HashSet<(usize, usize)>> = HashMap::new();
    for (name, xml_path) in sheet_paths {
        if let Some(sheet_xml) = read_zip_entry(&mut archive, &xml_path) {
            let cells = find_red_cells_in_sheet(&sheet_xml, &red_xf_indices, &red_ssi);
            if !cells.is_empty() {
                result.insert(name, cells);
            }
        }
    }

    result
}

/// Parse styles.xml → tập fontId có màu đỏ.
fn parse_red_font_ids(styles_xml: &str) -> HashSet<usize> {
    let mut result = HashSet::new();
    let doc = match roxmltree::Document::parse(styles_xml) {
        Ok(d) => d,
        Err(_) => return result,
    };

    let mut font_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "font"
            && node.parent().map(|p| p.tag_name().name()) == Some("fonts")
        {
            // Kiểm tra <color rgb="..."/> trực tiếp trong <font>
            let is_red = node.children().any(|child| {
                if child.tag_name().name() == "color" {
                    if let Some(rgb) = child.attribute("rgb") {
                        return is_argb_red(rgb);
                    }
                    if let Some(theme) = child.attribute("theme") {
                        // theme color 0 thường là trắng trong nhiều Excel templates
                        let _ = theme; // bỏ qua, không thể xác định không có theme data
                    }
                }
                false
            });
            if is_red {
                result.insert(font_idx);
            }
            font_idx += 1;
        }
    }

    result
}

/// Parse styles.xml → tập xf index trong cellXfs có fontId đỏ.
fn parse_red_xf_indices(styles_xml: &str, red_font_ids: &HashSet<usize>) -> HashSet<usize> {
    let mut result = HashSet::new();
    if red_font_ids.is_empty() {
        return result;
    }

    let doc = match roxmltree::Document::parse(styles_xml) {
        Ok(d) => d,
        Err(_) => return result,
    };

    // Thu thập fontId của từng cellStyleXf (để hỗ trợ kế thừa xfId)
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
            let apply_font = node.attribute("applyFont").map_or(false, |v| v == "1" || v == "true");

            let direct_red = apply_font && red_font_ids.contains(&font_id);

            // Kiểm tra kế thừa từ cellStyleXf (xfId)
            let inherited_red = node
                .attribute("xfId")
                .and_then(|s| s.parse::<usize>().ok())
                .and_then(|xfid| style_xf_fonts.get(xfid).copied())
                .map_or(false, |f| red_font_ids.contains(&f));

            if direct_red || inherited_red {
                result.insert(xf_idx);
            }
            xf_idx += 1;
        }
    }

    result
}

/// Parse sharedStrings.xml → tập si index có tất cả run đều có font đỏ.
fn parse_red_shared_strings(sst_xml: &str) -> HashSet<usize> {
    let mut result = HashSet::new();
    let doc = match roxmltree::Document::parse(sst_xml) {
        Ok(d) => d,
        Err(_) => return result,
    };

    let mut si_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "si" {
            let runs: Vec<_> = node
                .children()
                .filter(|c| c.tag_name().name() == "r")
                .collect();
            if !runs.is_empty() {
                let all_red = runs.iter().all(|r| has_red_font_run(r));
                if all_red {
                    result.insert(si_idx);
                }
            }
            si_idx += 1;
        }
    }

    result
}

/// Kiểm tra một run `<r>` có thuộc tính font đỏ trong `<rPr><color rgb="..."/>`.
fn has_red_font_run(run_node: &roxmltree::Node) -> bool {
    run_node.descendants().any(|child| {
        if child.tag_name().name() == "color" {
            if let Some(rgb) = child.attribute("rgb") {
                return is_argb_red(rgb);
            }
        }
        false
    })
}

/// Tìm tất cả ô có font đỏ trong một sheet XML.
fn find_red_cells_in_sheet(
    sheet_xml: &str,
    red_xf: &HashSet<usize>,
    red_ssi: &HashSet<usize>,
) -> HashSet<(usize, usize)> {
    let mut result = HashSet::new();
    let doc = match roxmltree::Document::parse(sheet_xml) {
        Ok(d) => d,
        Err(_) => return result,
    };

    for node in doc.descendants() {
        if node.tag_name().name() != "c" {
            continue;
        }
        let Some(pos) = node.attribute("r").and_then(parse_cell_ref) else {
            continue;
        };

        // 1. Cell style có font đỏ
        if let Some(si) = node.attribute("s").and_then(|s| s.parse::<usize>().ok()) {
            if red_xf.contains(&si) {
                result.insert(pos);
                continue;
            }
        }

        // 2. Shared string có rich-text font đỏ (t="s")
        if node.attribute("t") == Some("s") {
            if let Some(v) = node.descendants().find(|c| c.tag_name().name() == "v") {
                if let Some(ssi) = v.text().and_then(|t| t.parse::<usize>().ok()) {
                    if red_ssi.contains(&ssi) {
                        result.insert(pos);
                        continue;
                    }
                }
            }
        }

        // 3. Inline string có rich-text font đỏ (t="inlineStr")
        if node.attribute("t") == Some("inlineStr") {
            if let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") {
                let runs: Vec<_> = is_node
                    .children()
                    .filter(|c| c.tag_name().name() == "r")
                    .collect();
                if !runs.is_empty() && runs.iter().all(|r| has_red_font_run(r)) {
                    result.insert(pos);
                }
            }
        }
    }

    result
}

/// Kiểm tra chuỗi ARGB / RGB có phải màu đỏ không.
/// Chấp nhận dạng "AARRGGBB" (8 ký tự) hoặc "RRGGBB" (6 ký tự), có thể có '#' đầu.
fn is_argb_red(argb: &str) -> bool {
    let s = argb.trim_start_matches('#');
    let (r, g, b) = match s.len() {
        8 => {
            // AARRGGBB
            let r = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[6..8], 16).unwrap_or(0);
            (r, g, b)
        }
        6 => {
            // RRGGBB
            let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
            (r, g, b)
        }
        _ => return false,
    };
    r > 0xCC && g < 0x80 && b < 0x80
}

// ─────────────────────────────────────────────────────────────────────────────
// Tìm ô strikethrough trong file xlsx (tương tự file_compare_service.rs)
// ─────────────────────────────────────────────────────────────────────────────

pub fn find_strike_cells_xlsx(path: &str) -> HashMap<String, HashSet<(usize, usize)>> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext != "xlsx" && ext != "xlsm" {
        return HashMap::new();
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return HashMap::new(),
    };

    let strike_xf = read_zip_entry(&mut archive, "xl/styles.xml")
        .map(|xml| parse_strike_xf_indices(&xml))
        .unwrap_or_default();

    let strike_ssi = read_zip_entry(&mut archive, "xl/sharedStrings.xml")
        .map(|xml| parse_strike_shared_strings(&xml))
        .unwrap_or_default();

    if strike_xf.is_empty() && strike_ssi.is_empty() {
        return HashMap::new();
    }

    let workbook_xml = match read_zip_entry(&mut archive, "xl/workbook.xml") {
        Some(s) => s,
        None => return HashMap::new(),
    };
    let rels_xml = match read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels") {
        Some(s) => s,
        None => return HashMap::new(),
    };

    let sheet_paths = resolve_sheet_xml_paths(&workbook_xml, &rels_xml);

    let mut result: HashMap<String, HashSet<(usize, usize)>> = HashMap::new();
    for (name, xml_path) in sheet_paths {
        if let Some(sheet_xml) = read_zip_entry(&mut archive, &xml_path) {
            let cells = find_strike_cells_in_sheet(&sheet_xml, &strike_xf, &strike_ssi);
            if !cells.is_empty() {
                result.insert(name, cells);
            }
        }
    }

    result
}

fn has_strike_element(node: &roxmltree::Node) -> bool {
    node.descendants().any(|child| {
        child.tag_name().name() == "strike"
            && match child.attribute("val") {
                None | Some("true") | Some("1") => true,
                _ => false,
            }
    })
}

fn parse_strike_xf_indices(styles_xml: &str) -> HashSet<usize> {
    let mut result = HashSet::new();
    let doc = match roxmltree::Document::parse(styles_xml) {
        Ok(d) => d,
        Err(_) => return result,
    };

    let mut strike_fonts: HashSet<usize> = HashSet::new();
    let mut font_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "font"
            && node.parent().map(|p| p.tag_name().name()) == Some("fonts")
        {
            if has_strike_element(&node) {
                strike_fonts.insert(font_idx);
            }
            font_idx += 1;
        }
    }

    if strike_fonts.is_empty() {
        return result;
    }

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
            let direct = node
                .attribute("fontId")
                .and_then(|s| s.parse::<usize>().ok())
                .map_or(false, |f| strike_fonts.contains(&f));
            let inherited = node
                .attribute("xfId")
                .and_then(|s| s.parse::<usize>().ok())
                .and_then(|xfid| style_xf_fonts.get(xfid).copied())
                .map_or(false, |f| strike_fonts.contains(&f));

            if direct || inherited {
                result.insert(xf_idx);
            }
            xf_idx += 1;
        }
    }

    result
}

fn parse_strike_shared_strings(sst_xml: &str) -> HashSet<usize> {
    let mut result = HashSet::new();
    let doc = match roxmltree::Document::parse(sst_xml) {
        Ok(d) => d,
        Err(_) => return result,
    };

    let mut si_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "si" {
            let runs: Vec<_> = node
                .children()
                .filter(|c| c.tag_name().name() == "r")
                .collect();
            if !runs.is_empty() {
                let all_strike = runs.iter().all(|r| has_strike_element(r));
                if all_strike {
                    result.insert(si_idx);
                }
            }
            si_idx += 1;
        }
    }

    result
}

fn find_strike_cells_in_sheet(
    sheet_xml: &str,
    strike_xf: &HashSet<usize>,
    strike_ssi: &HashSet<usize>,
) -> HashSet<(usize, usize)> {
    let mut result = HashSet::new();
    let doc = match roxmltree::Document::parse(sheet_xml) {
        Ok(d) => d,
        Err(_) => return result,
    };

    for node in doc.descendants() {
        if node.tag_name().name() != "c" {
            continue;
        }
        let Some(pos) = node.attribute("r").and_then(parse_cell_ref) else {
            continue;
        };

        // 1. Cell style có strikethrough
        if let Some(si) = node.attribute("s").and_then(|s| s.parse::<usize>().ok()) {
            if strike_xf.contains(&si) {
                result.insert(pos);
                continue;
            }
        }

        // 2. Shared string có rich-text strikethrough (t="s")
        if node.attribute("t") == Some("s") {
            if let Some(v) = node.descendants().find(|c| c.tag_name().name() == "v") {
                if let Some(ssi) = v.text().and_then(|t| t.parse::<usize>().ok()) {
                    if strike_ssi.contains(&ssi) {
                        result.insert(pos);
                        continue;
                    }
                }
            }
        }

        // 3. Inline string có rich-text strikethrough (t="inlineStr")
        if node.attribute("t") == Some("inlineStr") {
            if let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") {
                let runs: Vec<_> = is_node
                    .children()
                    .filter(|c| c.tag_name().name() == "r")
                    .collect();
                if !runs.is_empty() && runs.iter().all(|r| has_strike_element(r)) {
                    result.insert(pos);
                }
            }
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Lấy màu tab sheet từ workbook XML
// ─────────────────────────────────────────────────────────────────────────────

fn get_sheet_tab_colors(path: &str) -> HashMap<String, String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return HashMap::new(),
    };

    let workbook_xml = match read_zip_entry(&mut archive, "xl/workbook.xml") {
        Some(s) => s,
        None => return HashMap::new(),
    };
    let rels_xml = match read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels") {
        Some(s) => s,
        None => return HashMap::new(),
    };

    // Lấy danh sách sheet name → xml path
    let sheet_paths = resolve_sheet_xml_paths(&workbook_xml, &rels_xml);

    let mut result: HashMap<String, String> = HashMap::new();

    // Tab color nằm trong từng sheet XML: <sheetPr><tabColor rgb="AARRGGBB"/>
    for (name, xml_path) in sheet_paths {
        if let Some(sheet_xml) = read_zip_entry(&mut archive, &xml_path) {
            if let Some(color) = parse_sheet_tab_color(&sheet_xml) {
                result.insert(name, color);
            }
        }
    }

    result
}

/// Tìm <sheetPr><tabColor rgb="..."/> trong sheet XML.
fn parse_sheet_tab_color(sheet_xml: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(sheet_xml).ok()?;
    for node in doc.descendants() {
        if node.tag_name().name() == "tabColor" {
            // Kiểm tra node cha là sheetPr
            if node.parent().map(|p| p.tag_name().name()) == Some("sheetPr") {
                if let Some(rgb) = node.attribute("rgb") {
                    return Some(rgb.to_string());
                }
                if let Some(theme) = node.attribute("theme") {
                    return Some(format!("theme:{theme}"));
                }
            }
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Kiểm tra chất lượng JP
// ─────────────────────────────────────────────────────────────────────────────

fn check_quality(
    jp_grid: &HashMap<String, Vec<Vec<String>>>,
    _jp_sheets: &[SheetMeta],
) -> Vec<QualityIssue> {
    let vn_char_re = Regex::new(
        r"[\u{1E00}-\u{1EFF}đĐơƠưƯăĂ]",
    )
    .unwrap();

    let mut issues: Vec<QualityIssue> = Vec::new();

    for (sheet_name, grid) in jp_grid {
        for (r, row) in grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if cell.trim().is_empty() {
                    continue;
                }
                // Chỉ kiểm tra ô có ký tự non-ASCII (có thể là JP hoặc VN)
                // Bỏ qua ô thuần ASCII (số, ký hiệu kỹ thuật)
                if cell.is_ascii() {
                    continue;
                }
                if vn_char_re.is_match(cell) {
                    issues.push(QualityIssue {
                        sheet: sheet_name.clone(),
                        row: r + 1,
                        col: c + 1,
                        issue_type: "vn_char".to_string(),
                        content: cell.clone(),
                        description: "JP文書内にベトナム語の文字が含まれています。翻訳漏れの可能性があります。".to_string(),
                    });
                }
            }
        }
    }

    issues
}

// ─────────────────────────────────────────────────────────────────────────────
// Hàm tiện ích dùng chung
// ─────────────────────────────────────────────────────────────────────────────

fn read_zip_entry(archive: &mut zip::ZipArchive<File>, name: &str) -> Option<String> {
    let mut entry = archive.by_name(name).ok()?;
    let mut buf = String::new();
    entry.read_to_string(&mut buf).ok()?;
    Some(buf)
}

fn resolve_sheet_xml_paths(workbook_xml: &str, rels_xml: &str) -> Vec<(String, String)> {
    let mut sheet_rids: Vec<(String, String)> = Vec::new();
    if let Ok(doc) = roxmltree::Document::parse(workbook_xml) {
        for node in doc.descendants() {
            if node.tag_name().name() == "sheet" {
                let name = node.attribute("name").unwrap_or("").to_string();
                let rid = node
                    .attribute((
                        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                        "id",
                    ))
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() && !rid.is_empty() {
                    sheet_rids.push((name, rid));
                }
            }
        }
    }

    let mut rid_to_target: HashMap<String, String> = HashMap::new();
    if let Ok(doc) = roxmltree::Document::parse(rels_xml) {
        for node in doc.descendants() {
            if node.tag_name().name() == "Relationship" {
                if let (Some(id), Some(target)) =
                    (node.attribute("Id"), node.attribute("Target"))
                {
                    rid_to_target.insert(id.to_string(), target.to_string());
                }
            }
        }
    }

    sheet_rids
        .into_iter()
        .filter_map(|(name, rid)| {
            rid_to_target.get(&rid).map(|target| {
                let xml_path = if target.starts_with('/') {
                    target[1..].to_string()
                } else {
                    format!("xl/{target}")
                };
                (name, xml_path)
            })
        })
        .collect()
}

/// Chuyển cell reference (vd "C5") sang (row, col) 0-indexed.
fn parse_cell_ref(cell_ref: &str) -> Option<(usize, usize)> {
    let bytes = cell_ref.as_bytes();
    let mut col = 0usize;
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        col = col * 26 + (bytes[i].to_ascii_uppercase() - b'A') as usize + 1;
        i += 1;
    }
    if col == 0 || i >= bytes.len() {
        return None;
    }
    col -= 1;
    let row: usize = cell_ref[i..].parse().ok()?;
    Some((row - 1, col))
}

// ─────────────────────────────────────────────────────────────────────────────
// Áp dụng thay đổi: ghi VN text (đỏ) vào đúng vị trí ô trong file JP xlsx
// ─────────────────────────────────────────────────────────────────────────────

/// Phân tích VN file, lấy danh sách ô đỏ, rồi ghi VN text (in đỏ) vào đúng vị trí
/// tương ứng trong JP file. Kết quả lưu ra `output_path`.
/// Nội dung VN được giữ nguyên (không dịch) và tô màu đỏ để reviewer kiểm tra.
pub fn apply_changes(vn_path: &str, jp_path: &str, output_path: &str) -> AppResult<ApplyResult> {
    let analysis = analyze(vn_path, jp_path)?;

    // Group red cells by sheet: sheet_name → Vec<(row_0, col_0, vn_text)>
    let mut cells_by_sheet: HashMap<String, Vec<(usize, usize, String)>> = HashMap::new();
    for rc in &analysis.red_cells {
        if rc.vn_text.trim().is_empty() {
            continue;
        }
        cells_by_sheet
            .entry(rc.sheet.clone())
            .or_default()
            .push((rc.row - 1, rc.col - 1, rc.vn_text.clone()));
    }

    if cells_by_sheet.is_empty() {
        return Err(AppError::new(
            "Không có ô đỏ nào trong file VN để áp dụng vào file JP.",
        ));
    }

    // Open JP file as ZIP
    let jp_file = File::open(jp_path)
        .map_err(|e| AppError::new(format!("Không mở được file JP: {e}")))?;
    let mut archive = zip::ZipArchive::new(jp_file)
        .map_err(|e| AppError::new(format!("File JP không phải ZIP hợp lệ: {e}")))?;

    // Resolve sheet name → XML path
    let workbook_xml = read_zip_entry(&mut archive, "xl/workbook.xml")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/workbook.xml trong file JP."))?;
    let rels_xml = read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/_rels/workbook.xml.rels."))?;
    let sheet_path_map: HashMap<String, String> =
        resolve_sheet_xml_paths(&workbook_xml, &rels_xml)
            .into_iter()
            .collect();

    // Inject VN text into each affected sheet XML
    let mut replaced: HashMap<String, Vec<u8>> = HashMap::new();
    let mut applied_count = 0usize;
    let mut skipped_count = 0usize;
    let mut sheets_modified: Vec<String> = Vec::new();

    for (sheet_name, cells) in cells_by_sheet {
        let xml_path = match sheet_path_map.get(&sheet_name) {
            Some(p) => p.clone(),
            None => {
                skipped_count += cells.len();
                continue;
            }
        };
        let sheet_xml = match read_zip_entry(&mut archive, &xml_path) {
            Some(xml) => xml,
            None => {
                skipped_count += cells.len();
                continue;
            }
        };

        let (new_xml, n_applied) = inject_cells_into_sheet_xml(&sheet_xml, &cells);
        let n_skipped = cells.len() - n_applied;
        applied_count += n_applied;
        skipped_count += n_skipped;

        if n_applied > 0 {
            replaced.insert(xml_path, new_xml.into_bytes());
            sheets_modified.push(sheet_name);
        }
    }

    // Write output ZIP (copy JP + swap modified sheet XMLs)
    write_output_zip(&mut archive, &replaced, output_path)?;

    Ok(ApplyResult {
        output_path: output_path.to_string(),
        applied_count,
        skipped_count,
        sheets_modified,
    })
}

// ── Apply helpers ────────────────────────────────────────────────────────────

struct SurgeryEdit {
    start: usize,
    end: usize,
    replacement: String,
}

fn apply_surgery(xml: &str, mut edits: Vec<SurgeryEdit>) -> String {
    if edits.is_empty() {
        return xml.to_string();
    }
    edits.sort_by(|a, b| b.start.cmp(&a.start));
    let mut result = xml.to_string();
    for edit in edits {
        result.replace_range(edit.start..edit.end, &edit.replacement);
    }
    result
}

/// Byte-surgery: inject VN text (red color) into specific cells of a sheet XML.
/// Returns `(new_xml, applied_count)`.
fn inject_cells_into_sheet_xml(
    sheet_xml: &str,
    cells: &[(usize, usize, String)], // (row_0, col_0, vn_text)
) -> (String, usize) {
    if cells.is_empty() {
        return (sheet_xml.to_string(), 0);
    }

    let doc = match roxmltree::Document::parse(sheet_xml) {
        Ok(d) => d,
        Err(_) => return (sheet_xml.to_string(), 0),
    };

    // (row_0, col_0) → (node_range, s_attr)
    let mut existing_cells: HashMap<(usize, usize), (std::ops::Range<usize>, String)> =
        HashMap::new();
    // row_1 → (row_range, sorted_cells_vec: [(col_0, cell_range)])
    let mut row_data: HashMap<
        usize,
        (std::ops::Range<usize>, Vec<(usize, std::ops::Range<usize>)>),
    > = HashMap::new();
    let mut sheet_data_range: Option<std::ops::Range<usize>> = None;

    for node in doc.descendants() {
        match node.tag_name().name() {
            "sheetData" => {
                sheet_data_range = Some(node.range());
            }
            "row" => {
                if let Some(r) = node.attribute("r").and_then(|s| s.parse::<usize>().ok()) {
                    row_data.insert(r, (node.range(), Vec::new()));
                }
            }
            "c" => {
                if let Some((row_0, col_0)) = node.attribute("r").and_then(parse_cell_ref) {
                    let s_attr = node.attribute("s").unwrap_or("").to_string();
                    existing_cells.insert((row_0, col_0), (node.range(), s_attr));
                    let row_1 = row_0 + 1;
                    if let Some((_, cells_vec)) = row_data.get_mut(&row_1) {
                        cells_vec.push((col_0, node.range()));
                    }
                }
            }
            _ => {}
        }
    }

    for (_, cells_vec) in row_data.values_mut() {
        cells_vec.sort_by_key(|(col, _)| *col);
    }

    // Sort input cells so same-position inserts accumulate in col order
    let mut sorted_cells = cells.to_vec();
    sorted_cells.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    // Accumulate same-position insertions to avoid collisions
    let mut pos_inserts: HashMap<usize, Vec<String>> = HashMap::new();
    // New rows: row_1 → [(col_0, vn_text)] — built separately to merge multi-cell rows
    let mut new_rows: HashMap<usize, Vec<(usize, String)>> = HashMap::new();

    let mut applied_count = 0usize;

    for (row_0, col_0, vn_text) in &sorted_cells {
        let row_1 = row_0 + 1;
        let cell_ref = format!("{}{}", col_index_to_letter(*col_0), row_1);

        if let Some((cell_range, s_attr)) = existing_cells.get(&(*row_0, *col_0)) {
            // Replace existing cell with VN inline string (red)
            edits.push(SurgeryEdit {
                start: cell_range.start,
                end: cell_range.end,
                replacement: make_vn_inline_cell(&cell_ref, s_attr, vn_text),
            });
            applied_count += 1;
        } else if let Some((row_range, cells_in_row)) = row_data.get(&row_1) {
            // Insert into existing row
            let insert_pos =
                find_cell_insert_pos(sheet_xml, row_range, cells_in_row, *col_0);
            pos_inserts
                .entry(insert_pos)
                .or_default()
                .push(make_vn_inline_cell(&cell_ref, "", vn_text));
            applied_count += 1;
        } else {
            // Need new row
            new_rows
                .entry(row_1)
                .or_default()
                .push((*col_0, vn_text.clone()));
            applied_count += 1;
        }
    }

    // Build new-row insertions (one <row> per row_1, all cells sorted by col)
    if let Some(sd_range) = &sheet_data_range {
        let row_keys: Vec<usize> = row_data.keys().copied().collect();
        for (row_1, mut row_cells) in new_rows {
            row_cells.sort_by_key(|(col, _)| *col);
            let cells_xml: String = row_cells
                .iter()
                .map(|(col_0, vn_text)| {
                    let cref = format!("{}{}", col_index_to_letter(*col_0), row_1);
                    make_vn_inline_cell(&cref, "", vn_text)
                })
                .collect();
            let row_xml = format!(r#"<row r="{row_1}">{cells_xml}</row>"#);
            let insert_pos = find_row_insert_pos(sheet_xml, sd_range, &row_keys, row_1);
            pos_inserts.entry(insert_pos).or_default().push(row_xml);
        }
    }

    // Convert accumulated insertions into edits
    for (pos, xmls) in pos_inserts {
        edits.push(SurgeryEdit {
            start: pos,
            end: pos,
            replacement: xmls.join(""),
        });
    }

    (apply_surgery(sheet_xml, edits), applied_count)
}

/// Find byte position to insert a new `<c>` inside an existing `<row>`.
fn find_cell_insert_pos(
    xml: &str,
    row_range: &std::ops::Range<usize>,
    cells_in_row: &[(usize, std::ops::Range<usize>)], // sorted by col_0
    new_col: usize,
) -> usize {
    // After last cell with col < new_col
    if let Some((_, range)) = cells_in_row.iter().rev().find(|(col, _)| *col < new_col) {
        return range.end;
    }
    // Before first cell (all have col >= new_col)
    if let Some((_, range)) = cells_in_row.first() {
        return range.start;
    }
    // Empty row: insert after the opening `<row ...>` tag
    find_tag_open_end(xml, row_range.start)
}

/// Find byte position to insert a new `<row>` inside `<sheetData>`.
fn find_row_insert_pos(
    xml: &str,
    sd_range: &std::ops::Range<usize>,
    row_1s: &[usize],
    new_row_1: usize,
) -> usize {
    // After last existing row with r < new_row_1
    let last_smaller = row_1s
        .iter()
        .filter(|&&r| r < new_row_1)
        .max()
        .copied();
    if let Some(prev_r) = last_smaller {
        // Find that row's range by re-scanning
        // (row_data ranges are not passed here, so we search the raw XML)
        let tag = format!(r#"<row r="{prev_r}""#);
        if let Some(start) = xml.find(&tag) {
            // Find the matching </row>
            if let Some(close) = xml[start..].find("</row>") {
                return start + close + "</row>".len();
            }
        }
    }
    // No preceding rows → insert just after opening <sheetData> tag
    find_tag_open_end(xml, sd_range.start)
}

/// Scan forward from `from` to find the byte position right after the first `>` of an XML tag,
/// skipping over attribute values (handles quoted `>` inside attributes).
fn find_tag_open_end(xml: &str, from: usize) -> usize {
    let bytes = xml.as_bytes();
    let mut i = from;
    let mut in_quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        match in_quote {
            Some(q) if b == q => {
                in_quote = None;
            }
            Some(_) => {}
            None => {
                if b == b'"' || b == b'\'' {
                    in_quote = Some(b);
                } else if b == b'>' {
                    return i + 1;
                }
            }
        }
        i += 1;
    }
    from
}

/// Build an inline-string `<c>` element containing `vn_text` rendered in red bold.
/// Keeps the original `s` (style) attribute if present so borders/alignment are preserved.
fn make_vn_inline_cell(cell_ref: &str, s_attr: &str, vn_text: &str) -> String {
    let s_part = if !s_attr.is_empty() {
        format!(" s=\"{s_attr}\"")
    } else {
        String::new()
    };
    format!(
        r#"<c r="{cell_ref}"{s_part} t="inlineStr"><is><r><rPr><color rgb="FFFF0000"/><b/></rPr><t xml:space="preserve">{}</t></r></is></c>"#,
        xml_escape(vn_text)
    )
}

/// Convert a 0-based column index to an Excel column letter (0→"A", 26→"AA", …).
fn col_index_to_letter(col_0: usize) -> String {
    let mut result: Vec<u8> = Vec::new();
    let mut c = col_0 + 1;
    while c > 0 {
        c -= 1;
        result.push(b'A' + (c % 26) as u8);
        c /= 26;
    }
    result.reverse();
    String::from_utf8(result).unwrap_or_default()
}

/// Escape XML text content (`&`, `<`, `>`).
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Write a ZIP archive to `output_path`, substituting entries from `replaced`.
/// All other entries are copied verbatim from `archive`.
fn write_output_zip(
    archive: &mut zip::ZipArchive<File>,
    replaced: &HashMap<String, Vec<u8>>,
    output_path: &str,
) -> AppResult<()> {
    // Collect entry names first (borrows archive briefly for each, then releases)
    let mut entry_names: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            if !name.is_empty() {
                entry_names.push(name);
            }
        }
    }

    let output_file = File::create(output_path)
        .map_err(|e| AppError::new(format!("Không tạo được file output: {e}")))?;
    let mut writer = ZipWriter::new(output_file);

    for name in &entry_names {
        if name.ends_with('/') {
            writer
                .add_directory(name.as_str(), SimpleFileOptions::default())
                .map_err(|e| AppError::new(format!("Lỗi ghi directory {name}: {e}")))?;
            continue;
        }
        let options =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        writer
            .start_file(name.as_str(), options)
            .map_err(|e| AppError::new(format!("Lỗi bắt đầu entry {name}: {e}")))?;

        if let Some(bytes) = replaced.get(name.as_str()) {
            writer
                .write_all(bytes)
                .map_err(|e| AppError::new(format!("Lỗi ghi nội dung {name}: {e}")))?;
        } else {
            let mut entry = archive
                .by_name(name)
                .map_err(|e| AppError::new(format!("Lỗi đọc entry {name}: {e}")))?;
            std::io::copy(&mut entry, &mut writer)
                .map_err(|e| AppError::new(format!("Lỗi copy {name}: {e}")))?;
        }
    }

    writer
        .finish()
        .map_err(|e| AppError::new(format!("Lỗi hoàn tất ZIP: {e}")))?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// AI API calls
// ─────────────────────────────────────────────────────────────────────────────

fn build_http_client() -> AppResult<Client> {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::new(format!("Không tạo được HTTP client: {e}")))
}

fn resolve_api_key(provider: &str, override_key: Option<&str>) -> AppResult<String> {
    if let Some(key) = override_key {
        let key = key.trim();
        if !key.is_empty() {
            return Ok(key.to_string());
        }
    }

    let label = match provider {
        "gemini" => "GEMINI_API_KEY",
        "groq" => "GROQ_API_KEY",
        other => {
            return Err(AppError::new(format!(
                "Nhà cung cấp không được hỗ trợ: '{other}'."
            )))
        }
    };

    if let Some(key) = api_key_from_config(label) {
        let key = key.trim().to_string();
        if !key.is_empty() {
            return Ok(key);
        }
    }

    Err(AppError::new(format!(
        "Chưa cấu hình API key cho {provider}. Thêm '{label}' vào section [ai] trong config.ini."
    )))
}

fn api_key_from_config(label: &str) -> Option<String> {
    use ini::Ini;
    let path = app_config::config_path();
    let ini = Ini::load_from_file(&path).ok()?;
    let section = ini.section(Some("ai"))?;
    section.get(label).map(|s| s.to_string())
}

async fn call_gemini(
    client: &Client,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> AppResult<String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );
    let body = json!({
        "contents": [{ "parts": [{ "text": prompt }] }]
    });

    let resp = client
        .post(&url)
        .header("x-goog-api-key", api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::new(format!("Gemini API request thất bại: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        return Err(AppError::new(format!(
            "Gemini API lỗi {}: {body_text}",
            status.as_u16()
        )));
    }

    let value: Value = resp
        .json()
        .await
        .map_err(|e| AppError::new(format!("Không phân tích được phản hồi Gemini: {e}")))?;

    let text = value["candidates"][0]["content"]["parts"]
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter_map(|p| p["text"].as_str())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    if text.trim().is_empty() {
        return Err(AppError::new("Gemini trả về nội dung trống."));
    }

    Ok(text.trim().to_string())
}

async fn call_groq(
    client: &Client,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> AppResult<String> {
    let url = "https://api.groq.com/openai/v1/chat/completions";
    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "あなたはプロの技術文書翻訳者です。ベトナム語のテキストを日本語に翻訳します。翻訳文のみ出力してください。"
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 2048
    });

    let resp = client
        .post(url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::new(format!("Groq API request thất bại: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        return Err(AppError::new(format!(
            "Groq API lỗi {}: {body_text}",
            status.as_u16()
        )));
    }

    let value: Value = resp
        .json()
        .await
        .map_err(|e| AppError::new(format!("Không phân tích được phản hồi Groq: {e}")))?;

    value["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| AppError::new("Groq trả về nội dung trống."))
}

// ─────────────────────────────────────────────────────────────────────────────
// Xuất báo cáo Excel
// ─────────────────────────────────────────────────────────────────────────────

/// Tạo Format header chung (bold, nền xám, border).
fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x2563EB))
        .set_font_color(Color::RGB(0xFFFFFF))
        .set_border(FormatBorder::Thin)
        .set_text_wrap()
}

/// Tạo Format ô thường (border, wrap).
fn cell_format() -> Format {
    Format::new()
        .set_border(FormatBorder::Thin)
        .set_text_wrap()
}

/// Tạo Format ô trung tâm (STT).
fn center_format() -> Format {
    Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(rust_xlsxwriter::FormatAlign::Center)
}

/// Format ô ô đỏ (dòng red cell).
fn red_cell_format() -> Format {
    Format::new()
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xFFF1F2))
        .set_font_color(Color::RGB(0xBE123C))
        .set_text_wrap()
}

/// Format ô strikethrough (dòng strike cell).
fn strike_cell_format() -> Format {
    Format::new()
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xFEF9C3))
        .set_font_color(Color::RGB(0x92400E))
        .set_text_wrap()
}

/// Format ô chất lượng.
fn quality_format() -> Format {
    Format::new()
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xFFF7ED))
        .set_font_color(Color::RGB(0x9A3412))
        .set_text_wrap()
}

/// Ghi sheet "概要" (Summary).
fn write_summary_sheet(workbook: &mut Workbook, analysis: &SyncAnalysis) -> AppResult<()> {
    let sheet = workbook.add_worksheet();
    sheet
        .set_name("概要")
        .map_err(|e| AppError::new(format!("Lỗi tạo sheet: {e}")))?;

    let hfmt = header_format();
    let cfmt = cell_format();

    // Tiêu đề cột
    let headers = ["項目", "値"];
    let widths = [40.0f64, 80.0f64];
    for (c, (h, w)) in headers.iter().zip(widths.iter()).enumerate() {
        let col = c as u16;
        sheet
            .set_column_width(col, *w)
            .map_err(|e| AppError::new(format!("Lỗi đặt độ rộng cột: {e}")))?;
        sheet
            .write_string_with_format(0, col, *h, &hfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi header: {e}")))?;
    }

    let rows = [
        ("VNファイルパス", analysis.vn_path.as_str()),
        ("JPファイルパス", analysis.jp_path.as_str()),
    ];

    for (i, (label, value)) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        sheet
            .write_string_with_format(r, 0, *label, &cfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 1, *value, &cfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
    }

    let numeric_rows: &[(&str, usize)] = &[
        ("VNシート数", analysis.vn_sheets.len()),
        ("JPシート数", analysis.jp_sheets.len()),
        ("赤セル数 (翻訳対象)", analysis.red_cells.len()),
        ("取消線セル数 (削除対象)", analysis.strike_cells.len()),
        ("品質問題数", analysis.quality_issues.len()),
    ];

    let offset = rows.len() as u32 + 1;
    for (i, (label, value)) in numeric_rows.iter().enumerate() {
        let r = offset + i as u32;
        sheet
            .write_string_with_format(r, 0, *label, &cfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_number_with_format(r, 1, *value as f64, &cfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
    }

    sheet
        .set_freeze_panes(1, 0)
        .map_err(|e| AppError::new(format!("Lỗi freeze panes: {e}")))?;

    Ok(())
}

/// Ghi sheet "反映対象 (赤セル)" (Red Cells).
fn write_red_cells_sheet(workbook: &mut Workbook, analysis: &SyncAnalysis) -> AppResult<()> {
    let sheet = workbook.add_worksheet();
    sheet
        .set_name("反映対象 (赤セル)")
        .map_err(|e| AppError::new(format!("Lỗi tạo sheet: {e}")))?;

    let hfmt = header_format();
    let cfmt = cell_format();
    let rfmt = red_cell_format();
    let ctr = center_format();

    let headers = ["#", "シート名", "行", "列", "VNテキスト", "現在のJPテキスト", "AI翻訳", "アクション"];
    let widths = [6.0f64, 20.0, 8.0, 8.0, 48.0, 48.0, 48.0, 20.0];

    for (c, (h, w)) in headers.iter().zip(widths.iter()).enumerate() {
        let col = c as u16;
        sheet
            .set_column_width(col, *w)
            .map_err(|e| AppError::new(format!("Lỗi đặt độ rộng cột: {e}")))?;
        sheet
            .write_string_with_format(0, col, *h, &hfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi header: {e}")))?;
    }

    for (i, cell) in analysis.red_cells.iter().enumerate() {
        let r = (i + 1) as u32;
        sheet
            .write_number_with_format(r, 0, (i + 1) as f64, &ctr)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 1, &cell.sheet, &rfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_number_with_format(r, 2, cell.row as f64, &ctr)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_number_with_format(r, 3, cell.col as f64, &ctr)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 4, &cell.vn_text, &rfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 5, &cell.jp_text, &cfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        let translation = cell.translation.as_deref().unwrap_or("");
        sheet
            .write_string_with_format(r, 6, translation, &cfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 7, "要翻訳", &cfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
    }

    sheet
        .set_freeze_panes(1, 0)
        .map_err(|e| AppError::new(format!("Lỗi freeze panes: {e}")))?;

    Ok(())
}

/// Ghi sheet "削除対象 (取消線)" (Strikethrough Cells).
fn write_strike_cells_sheet(workbook: &mut Workbook, analysis: &SyncAnalysis) -> AppResult<()> {
    let sheet = workbook.add_worksheet();
    sheet
        .set_name("削除対象 (取消線)")
        .map_err(|e| AppError::new(format!("Lỗi tạo sheet: {e}")))?;

    let hfmt = header_format();
    let sfmt = strike_cell_format();
    let ctr = center_format();

    let headers = ["#", "シート名", "行", "列", "テキスト"];
    let widths = [6.0f64, 20.0, 8.0, 8.0, 80.0];

    for (c, (h, w)) in headers.iter().zip(widths.iter()).enumerate() {
        let col = c as u16;
        sheet
            .set_column_width(col, *w)
            .map_err(|e| AppError::new(format!("Lỗi đặt độ rộng cột: {e}")))?;
        sheet
            .write_string_with_format(0, col, *h, &hfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi header: {e}")))?;
    }

    for (i, cell) in analysis.strike_cells.iter().enumerate() {
        let r = (i + 1) as u32;
        sheet
            .write_number_with_format(r, 0, (i + 1) as f64, &ctr)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 1, &cell.sheet, &sfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_number_with_format(r, 2, cell.row as f64, &ctr)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_number_with_format(r, 3, cell.col as f64, &ctr)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 4, &cell.text, &sfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
    }

    sheet
        .set_freeze_panes(1, 0)
        .map_err(|e| AppError::new(format!("Lỗi freeze panes: {e}")))?;

    Ok(())
}

/// Ghi sheet "品質確認" (Quality Issues).
fn write_quality_sheet(workbook: &mut Workbook, analysis: &SyncAnalysis) -> AppResult<()> {
    let sheet = workbook.add_worksheet();
    sheet
        .set_name("品質確認")
        .map_err(|e| AppError::new(format!("Lỗi tạo sheet: {e}")))?;

    let hfmt = header_format();
    let qfmt = quality_format();
    let ctr = center_format();

    let headers = ["#", "シート名", "行", "列", "問題の種類", "コンテンツ", "説明"];
    let widths = [6.0f64, 20.0, 8.0, 8.0, 20.0, 48.0, 60.0];

    for (c, (h, w)) in headers.iter().zip(widths.iter()).enumerate() {
        let col = c as u16;
        sheet
            .set_column_width(col, *w)
            .map_err(|e| AppError::new(format!("Lỗi đặt độ rộng cột: {e}")))?;
        sheet
            .write_string_with_format(0, col, *h, &hfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi header: {e}")))?;
    }

    for (i, issue) in analysis.quality_issues.iter().enumerate() {
        let r = (i + 1) as u32;
        sheet
            .write_number_with_format(r, 0, (i + 1) as f64, &ctr)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 1, &issue.sheet, &qfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_number_with_format(r, 2, issue.row as f64, &ctr)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_number_with_format(r, 3, issue.col as f64, &ctr)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 4, &issue.issue_type, &qfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 5, &issue.content, &qfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
        sheet
            .write_string_with_format(r, 6, &issue.description, &qfmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi cell: {e}")))?;
    }

    sheet
        .set_freeze_panes(1, 0)
        .map_err(|e| AppError::new(format!("Lỗi freeze panes: {e}")))?;

    Ok(())
}
