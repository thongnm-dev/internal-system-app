//! Service đồng bộ tài liệu thiết kế chi tiết VN → JP.
//!
//! Phân tích sự khác biệt giữa file Excel VN (ô đỏ = nội dung mới cần dịch)
//! và file Excel JP (ô có strikethrough = nội dung cần xóa).
//! Hỗ trợ dịch tự động qua AI (Gemini / Groq) và xuất báo cáo Excel.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
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

    // --- Nhận diện loại tài liệu → vùng nội dung hợp lệ theo cột (xem `content_bounds_for`) ---
    let doc_type = detect_doc_type(vn_path, jp_path);

    // --- Tìm ô đỏ (VN) và ô strikethrough (JP) ---
    let vn_red = filter_cells_by_bounds(find_red_cells_xlsx(vn_path), doc_type);
    let jp_strike = filter_cells_by_bounds(find_strike_cells_xlsx(jp_path), doc_type);

    // --- Tìm shape/textbox nổi có chữ đỏ (VN) / gạch bỏ (JP) ---
    let vn_shapes = filter_shapes_by_bounds(find_shapes_xlsx(vn_path), doc_type);
    let jp_shapes = filter_shapes_by_bounds(find_shapes_xlsx(jp_path), doc_type);
    let vn_shape_red_counts: HashMap<String, usize> = vn_shapes
        .iter()
        .map(|(name, shapes)| {
            let n = shapes
                .iter()
                .filter(|s| s.paragraphs.iter().any(|p| p.any_red))
                .count();
            (name.clone(), n)
        })
        .collect();
    let jp_shape_strike_counts: HashMap<String, usize> = jp_shapes
        .iter()
        .map(|(name, shapes)| {
            let n = shapes
                .iter()
                .filter(|s| s.paragraphs.iter().any(|p| p.all_struck))
                .count();
            (name.clone(), n)
        })
        .collect();

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
            let red_cell_count =
                red_set.map(|s| s.len()).unwrap_or(0) + vn_shape_red_counts.get(name).copied().unwrap_or(0);
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
            let strike_cell_count = strike_set.map(|s| s.len()).unwrap_or(0)
                + jp_shape_strike_counts.get(name).copied().unwrap_or(0);
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
            let vn_red_count = vn_red.get(name).map(|s| s.len()).unwrap_or(0)
                + vn_shape_red_counts.get(name).copied().unwrap_or(0);
            let jp_strike_count = jp_strike.get(name).map(|s| s.len()).unwrap_or(0)
                + jp_shape_strike_counts.get(name).copied().unwrap_or(0);
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
                is_shape: false,
            });
        }
    }

    // --- Xây dựng RedCell cho textbox/shape nổi (VN) ---
    for (sheet_name, shapes) in &vn_shapes {
        for shape in shapes {
            let vn_text: String = shape
                .paragraphs
                .iter()
                .filter(|p| p.any_red)
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            if vn_text.trim().is_empty() {
                continue;
            }
            // Nội dung JP hiện tại của shape cùng tên (nếu có) — chỉ để hiển thị so sánh,
            // xem ghi chú "Đối ứng VN ↔ JP cho shape" đầu mục xử lý drawing.
            let jp_text = jp_shapes
                .get(sheet_name)
                .and_then(|list| {
                    (!shape.name.is_empty())
                        .then(|| list.iter().find(|s| s.name == shape.name))
                        .flatten()
                })
                .map(|s| {
                    s.paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            red_cells.push(RedCell {
                sheet: sheet_name.clone(),
                row: shape.anchor_row0 + 1,
                col: shape.anchor_col0 + 1,
                vn_text,
                jp_text,
                translation: None,
                is_shape: true,
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
                is_shape: false,
            });
        }
    }

    // --- Xây dựng StrikeCell cho textbox/shape nổi (JP) ---
    for (sheet_name, shapes) in &jp_shapes {
        for shape in shapes {
            let text: String = shape
                .paragraphs
                .iter()
                .filter(|p| p.all_struck)
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            if text.trim().is_empty() {
                continue;
            }
            strike_cells.push(StrikeCell {
                sheet: sheet_name.clone(),
                row: shape.anchor_row0 + 1,
                col: shape.anchor_col0 + 1,
                text,
                is_shape: true,
            });
        }
    }

    // --- Kiểm tra chất lượng (JP) ---
    let quality_issues = check_quality(&jp_grid, &jp_shapes, &jp_sheets, doc_type);

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

pub(crate) fn read_workbook_grid(path: &str) -> AppResult<HashMap<String, Vec<Vec<String>>>> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| AppError::new(format!("Không mở được Excel {path}: {e}")))?;
    let names: Vec<String> = workbook.sheet_names().to_vec();
    let mut result: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for name in names {
        let Ok(range) = workbook.worksheet_range(&name) else {
            continue;
        };
        // calamine trả về Range TƯƠNG ĐỐI theo range.start() (vd sheet có dimension "A2:B2"
        // thì hàng/cột đầu tiên của range.rows() là hàng 2, không phải hàng 1) — trong khi
        // find_red_cells_xlsx/find_strike_cells_xlsx lấy tọa độ TUYỆT ĐỐI trực tiếp từ XML
        // (r="B2" → row 0-based = 1). Phải pad về (0,0) để 2 nguồn tọa độ khớp nhau, nếu
        // không sẽ tra nhầm ô hoặc bỏ sót toàn bộ ô đỏ/gạch bỏ khi sheet không bắt đầu từ A1.
        let (start_row, start_col) = range.start().unwrap_or((0, 0));
        let mut grid: Vec<Vec<String>> = Vec::with_capacity(start_row as usize + range.height());
        for _ in 0..start_row {
            grid.push(Vec::new());
        }
        for row in range.rows() {
            let mut abs_row: Vec<String> = Vec::with_capacity(start_col as usize + row.len());
            for _ in 0..start_col {
                abs_row.push(String::new());
            }
            abs_row.extend(row.iter().map(cell_data_to_string));
            grid.push(abs_row);
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

    let theme_colors = read_theme_colors_from_archive(&mut archive);

    // Tìm fontId có font màu đỏ trong styles.xml
    let red_font_ids: HashSet<usize> = read_zip_entry(&mut archive, "xl/styles.xml")
        .map(|xml| parse_red_font_ids(&xml, &theme_colors))
        .unwrap_or_default();

    // Tìm xf index có fontId là đỏ trong cellXfs
    let red_xf_indices: HashSet<usize> = read_zip_entry(&mut archive, "xl/styles.xml")
        .map(|xml| parse_red_xf_indices(&xml, &red_font_ids))
        .unwrap_or_default();

    // Tìm shared string index có rich-text với font đỏ (+ tập MỌI ssi rich-text, dùng để ưu
    // tiên rich-text trên style cấp-cell khi xét từng ô — xem find_red_cells_in_sheet).
    let sst_xml = read_zip_entry(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (red_ssi_map, all_rich_ssi) = if sst_xml.is_empty() {
        (HashMap::new(), HashSet::new())
    } else {
        parse_shared_strings_rich_info(&sst_xml, &theme_colors)
    };
    let red_ssi: HashSet<usize> = red_ssi_map.into_keys().collect();

    // Lưu ý: KHÔNG được early-return dù red_xf_indices/red_ssi rỗng — ô inline string
    // (t="inlineStr") có thể mang màu đỏ nhúng trực tiếp trong run, không qua styles.xml
    // hay sharedStrings.xml, nên vẫn phải quét từng sheet để bắt được trường hợp này.
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
            let cells = find_red_cells_in_sheet(
                &sheet_xml,
                &red_xf_indices,
                &red_ssi,
                &all_rich_ssi,
                &theme_colors,
            );
            if !cells.is_empty() {
                result.insert(name, cells);
            }
        }
    }

    result
}

/// Parse styles.xml → tập fontId có màu đỏ (rgb, theme+tint, hoặc indexed — xem `resolve_color_node`).
fn parse_red_font_ids(styles_xml: &str, theme_colors: &[String]) -> HashSet<usize> {
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
            let is_red = node.children().any(|child| {
                child.tag_name().name() == "color"
                    && resolve_color_node(&child, theme_colors)
                        .map(|rgb| is_argb_red(&rgb))
                        .unwrap_or(false)
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

/// Kiểm tra một run `<r>` có thuộc tính font đỏ trong `<rPr><color .../>` (rgb, theme+tint, hoặc
/// indexed — xem `resolve_color_node`).
fn has_red_font_run(run_node: &roxmltree::Node, theme_colors: &[String]) -> bool {
    run_node.descendants().any(|child| {
        child.tag_name().name() == "color"
            && resolve_color_node(&child, theme_colors)
                .map(|rgb| is_argb_red(&rgb))
                .unwrap_or(false)
    })
}

/// Tìm tất cả ô có font đỏ trong một sheet XML. Rich-text run (nếu có) LUÔN được xét TRƯỚC
/// style cấp-cell — đúng theo cách Excel hiển thị (run đè lên font mặc định của cell), tránh
/// hiểu nhầm 1 cell có base style đỏ nhưng nội dung rich-text thật lại không đỏ (hoặc ngược lại).
fn find_red_cells_in_sheet(
    sheet_xml: &str,
    red_xf: &HashSet<usize>,
    red_ssi: &HashSet<usize>,
    all_rich_ssi: &HashSet<usize>,
    theme_colors: &[String],
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

        let mut handled_by_rich = false;

        // 1. Shared string rich-text (t="s") — ưu tiên cao nhất nếu có run
        if node.attribute("t") == Some("s") {
            if let Some(v) = node.descendants().find(|c| c.tag_name().name() == "v") {
                if let Some(ssi) = v.text().and_then(|t| t.parse::<usize>().ok()) {
                    if all_rich_ssi.contains(&ssi) {
                        handled_by_rich = true;
                        if red_ssi.contains(&ssi) {
                            result.insert(pos);
                        }
                    }
                }
            }
        }

        // 2. Inline string rich-text (t="inlineStr")
        if !handled_by_rich && node.attribute("t") == Some("inlineStr") {
            if let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") {
                let runs: Vec<_> = is_node
                    .children()
                    .filter(|c| c.tag_name().name() == "r")
                    .collect();
                if !runs.is_empty() {
                    handled_by_rich = true;
                    if runs.iter().any(|r| has_red_font_run(r, theme_colors)) {
                        result.insert(pos);
                    }
                }
            }
        }

        if handled_by_rich {
            continue;
        }

        // 3. Cell style có font đỏ (chỉ áp dụng khi ô KHÔNG có rich-text run nào)
        if let Some(si) = node.attribute("s").and_then(|s| s.parse::<usize>().ok()) {
            if red_xf.contains(&si) {
                result.insert(pos);
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

/// Kiểm tra chuỗi ARGB / RGB có phải màu XANH (blue) không — dùng nhận diện header nhóm MỚI trong
/// tài liệu "画面間インタフェース仕様書" (ví dụ `FF0070C0`). Chấp nhận "AARRGGBB"/"RRGGBB", có thể có '#'.
pub(crate) fn is_argb_blue(argb: &str) -> bool {
    let (r, g, b) = match parse_rgb_triplet(argb) {
        Some(t) => t,
        None => return false,
    };
    // Xanh dương/xanh lam: kênh B trội hẳn, R thấp, và B vượt cả R lẫn G một khoảng rõ rệt.
    b > 0x90 && r < 0x80 && b > r.saturating_add(0x30) && b > g
}

/// Tách (r, g, b) từ chuỗi ARGB "AARRGGBB" (8) hoặc "RRGGBB" (6), có thể có '#'. `None` nếu sai.
pub(crate) fn parse_rgb_triplet(argb: &str) -> Option<(u8, u8, u8)> {
    let s = argb.trim_start_matches('#');
    let off = match s.len() {
        8 => 2, // bỏ 2 ký tự alpha đầu
        6 => 0,
        _ => return None,
    };
    let r = u8::from_str_radix(&s[off..off + 2], 16).ok()?;
    let g = u8::from_str_radix(&s[off + 2..off + 4], 16).ok()?;
    let b = u8::from_str_radix(&s[off + 4..off + 6], 16).ok()?;
    Some((r, g, b))
}

// ─────────────────────────────────────────────────────────────────────────────
// Resolve màu theme (`<color theme="n" tint="...">`) và màu indexed (`<color indexed="n">`) về
// RGB thật. Trước đây mọi nơi kiểm tra màu (đỏ/đen/"đã đổi") chỉ đọc `attribute("rgb")`, bỏ qua 2
// dạng còn lại — nhưng Excel áp màu qua palette "Theme Colors" (theme+tint) hoặc palette legacy
// indexed khá phổ biến. Bỏ qua chúng khiến ô VN tô màu theo theme bị nhận nhầm là "không đổi"
// (coi như đen) và bị pipeline C234/C232/C233/C235/C236/C238 âm thầm ghi đè mất nội dung khi Áp
// dụng (xem `clone_vn_sheet_for_jp`).
// ─────────────────────────────────────────────────────────────────────────────

/// 12 màu theme mặc định (Office theme) — dùng khi file không có `xl/theme/theme1.xml` hoặc parse
/// lỗi. Thứ tự khớp với index dùng trong `<color theme="n">` (xem `parse_theme_colors`).
fn default_theme_colors() -> Vec<String> {
    [
        "FFFFFF", "000000", "E7E6E6", "44546A", "4472C4", "ED7D31", "A5A5A5", "FFC000", "5B9BD5",
        "70AD47", "0563C1", "954F72",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Parse `xl/theme/theme1.xml` → 12 màu theme theo ĐÚNG thứ tự index dùng trong
/// `<color theme="n">` của font/cell — LƯU Ý: OOXML hoán đổi dk1/lt1 so với thứ tự khai báo trong
/// `<a:clrScheme>` (quirk chuẩn mà mọi thư viện đọc xlsx phải xử lý): theme index 0 = lt1
/// (Background 1), 1 = dk1 (Text 1), rồi mới tới lt2/dk2/accent1..6/hlink/folHlink.
fn parse_theme_colors(theme_xml: &str) -> Vec<String> {
    let defaults = default_theme_colors();
    let Ok(doc) = roxmltree::Document::parse(theme_xml) else {
        return defaults;
    };
    let Some(scheme) = doc.descendants().find(|n| n.tag_name().name() == "clrScheme") else {
        return defaults;
    };

    let mut by_name: HashMap<&str, String> = HashMap::new();
    for child in scheme.children().filter(|n| n.is_element()) {
        let name = child.tag_name().name();
        let hex = child.children().find_map(|c| match c.tag_name().name() {
            "srgbClr" => c.attribute("val").map(|v| v.to_uppercase()),
            "sysClr" => c.attribute("lastClr").map(|v| v.to_uppercase()),
            _ => None,
        });
        if let Some(hex) = hex {
            by_name.insert(name, hex);
        }
    }

    let theme_index_order = [
        "lt1", "dk1", "lt2", "dk2", "accent1", "accent2", "accent3", "accent4", "accent5",
        "accent6", "hlink", "folHlink",
    ];
    theme_index_order
        .iter()
        .enumerate()
        .map(|(i, name)| by_name.get(*name).cloned().unwrap_or_else(|| defaults[i].clone()))
        .collect()
}

/// Đọc `xl/theme/theme1.xml` từ archive đã mở → 12 màu theme (xem `parse_theme_colors`). Trả về
/// palette mặc định nếu file không có theme (hiếm nhưng hợp lệ về mặt cấu trúc OOXML).
pub(crate) fn read_theme_colors_from_archive(archive: &mut zip::ZipArchive<File>) -> Vec<String> {
    match read_zip_entry(archive, "xl/theme/theme1.xml") {
        Some(xml) => parse_theme_colors(&xml),
        None => default_theme_colors(),
    }
}

/// Chuyển (r, g, b) 0-255 → (h, s, l) chuẩn hoá 0.0-1.0, dùng cho công thức tint OOXML.
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if (max - rf).abs() < f64::EPSILON {
        ((gf - bf) / d).rem_euclid(6.0)
    } else if (max - gf).abs() < f64::EPSILON {
        (bf - rf) / d + 2.0
    } else {
        (rf - gf) / d + 4.0
    } / 6.0;
    (h.rem_euclid(1.0), s, l)
}

/// Ngược lại `rgb_to_hsl`.
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s.abs() < f64::EPSILON {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue_to_rgb = |t: f64| -> f64 {
        let t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 1.0 / 2.0 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    let r = hue_to_rgb(h + 1.0 / 3.0);
    let g = hue_to_rgb(h);
    let b = hue_to_rgb(h - 1.0 / 3.0);
    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

/// Áp dụng `tint` (thuộc tính trên `<color theme="n" tint="...">`) lên màu gốc — công thức chuẩn
/// OOXML: tint âm làm tối (nhân độ sáng), tint dương làm sáng (kéo về trắng). `rgb_hex` là 6 ký
/// tự không có alpha.
fn apply_tint(rgb_hex: &str, tint: f64) -> String {
    let Some((r, g, b)) = parse_rgb_triplet(rgb_hex) else {
        return rgb_hex.to_string();
    };
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let new_l = if tint < 0.0 {
        l * (1.0 + tint)
    } else {
        l * (1.0 - tint) + tint
    };
    let (nr, ng, nb) = hsl_to_rgb(h, s, new_l.clamp(0.0, 1.0));
    format!("{nr:02X}{ng:02X}{nb:02X}")
}

/// Palette màu indexed hợp lệ (legacy, index 0-63) theo chuẩn OOXML/BIFF — dùng khi
/// `<color indexed="n"/>` xuất hiện (Excel cũ hoặc không ghi `<colors>` override tùy biến). Index
/// 64 ("System foreground"/tự động) và ngoài bảng trả về `None` — coi như màu tự động (đen), giữ
/// hành vi cũ.
fn resolve_indexed_color(idx: usize) -> Option<String> {
    const PALETTE: [&str; 64] = [
        "000000", "FFFFFF", "FF0000", "00FF00", "0000FF", "FFFF00", "FF00FF", "00FFFF",
        "000000", "FFFFFF", "FF0000", "00FF00", "0000FF", "FFFF00", "FF00FF", "00FFFF",
        "800000", "008000", "000080", "808000", "800080", "008080", "C0C0C0", "808080",
        "9999FF", "993366", "FFFFCC", "CCFFFF", "660066", "FF8080", "0066CC", "CCCCFF",
        "000080", "FF00FF", "FFFF00", "00FFFF", "800080", "800000", "008080", "0000FF",
        "00CCFF", "CCFFFF", "CCFFCC", "FFFF99", "99CCFF", "FF99CC", "CC99FF", "FFCC99",
        "3366FF", "33CCCC", "99CC00", "FFCC00", "FF9900", "FF6600", "666699", "969696",
        "003366", "339966", "003300", "333300", "993300", "993366", "333399", "333333",
    ];
    PALETTE.get(idx).map(|s| s.to_string())
}

/// Resolve 1 node `<color .../>` (bên trong `<font>`/`<rPr>`) → hex RGB (6 ký tự, không alpha),
/// theo thứ tự ưu tiên rgb > theme(+tint) > indexed. `None` nếu không xác định được (màu tự động
/// — coi như đen, giữ hành vi cũ).
fn resolve_color_node(color_node: &roxmltree::Node, theme_colors: &[String]) -> Option<String> {
    if let Some(rgb) = color_node.attribute("rgb") {
        return Some(rgb.trim_start_matches('#').to_uppercase());
    }
    if let Some(theme_str) = color_node.attribute("theme") {
        let idx: usize = theme_str.parse().ok()?;
        let base = theme_colors.get(idx)?.clone();
        let tint: f64 = color_node
            .attribute("tint")
            .and_then(|t| t.parse().ok())
            .unwrap_or(0.0);
        return Some(if tint.abs() > f64::EPSILON {
            apply_tint(&base, tint)
        } else {
            base
        });
    }
    if let Some(idx_str) = color_node.attribute("indexed") {
        let idx: usize = idx_str.parse().ok()?;
        return resolve_indexed_color(idx);
    }
    None
}

/// Parse styles.xml → tập fontId có màu xanh (blue). Dùng bởi `super::c238_sync_service` (nhận
/// diện header nhóm mới màu xanh).
pub(crate) fn parse_blue_font_ids(styles_xml: &str) -> HashSet<usize> {
    let mut result = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(styles_xml) else {
        return result;
    };
    let mut font_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "font"
            && node.parent().map(|p| p.tag_name().name()) == Some("fonts")
        {
            let is_blue = node.children().any(|child| {
                child.tag_name().name() == "color"
                    && child.attribute("rgb").map(is_argb_blue).unwrap_or(false)
            });
            if is_blue {
                result.insert(font_idx);
            }
            font_idx += 1;
        }
    }
    result
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

    let sst_xml = read_zip_entry(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();
    let all_rich_ssi = if sst_xml.is_empty() {
        HashSet::new()
    } else {
        let theme_colors = read_theme_colors_from_archive(&mut archive);
        parse_shared_strings_rich_info(&sst_xml, &theme_colors).1
    };

    // Lưu ý: KHÔNG early-return dù strike_xf/strike_ssi rỗng — ô inline string có thể mang
    // strikethrough nhúng trực tiếp trong run, không qua styles.xml/sharedStrings.xml.
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
            let cells =
                find_strike_cells_in_sheet(&sheet_xml, &strike_xf, &strike_ssi, &all_rich_ssi);
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
    all_rich_ssi: &HashSet<usize>,
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

        let mut handled_by_rich = false;

        // 1. Shared string rich-text (t="s") — ưu tiên cao nhất nếu có run
        if node.attribute("t") == Some("s") {
            if let Some(v) = node.descendants().find(|c| c.tag_name().name() == "v") {
                if let Some(ssi) = v.text().and_then(|t| t.parse::<usize>().ok()) {
                    if all_rich_ssi.contains(&ssi) {
                        handled_by_rich = true;
                        if strike_ssi.contains(&ssi) {
                            result.insert(pos);
                        }
                    }
                }
            }
        }

        // 2. Inline string rich-text (t="inlineStr")
        if !handled_by_rich && node.attribute("t") == Some("inlineStr") {
            if let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") {
                let runs: Vec<_> = is_node
                    .children()
                    .filter(|c| c.tag_name().name() == "r")
                    .collect();
                if !runs.is_empty() {
                    handled_by_rich = true;
                    if runs.iter().all(|r| has_strike_element(r)) {
                        result.insert(pos);
                    }
                }
            }
        }

        if handled_by_rich {
            continue;
        }

        // 3. Cell style có strikethrough (chỉ áp dụng khi ô KHÔNG có rich-text run nào)
        if let Some(si) = node.attribute("s").and_then(|s| s.parse::<usize>().ok()) {
            if strike_xf.contains(&si) {
                result.insert(pos);
            }
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Canh dòng VN ↔ JP: phát hiện dòng VN chưa có trong JP (lệch dòng do chèn/xóa
// nội dung), đề xuất vị trí chèn dòng trống vào JP để TL xác nhận trước khi ghi thật.
// Vì VN/JP khác ngôn ngữ nên KHÔNG so trực tiếp nội dung chữ — dùng các ô "neo"
// (số/mã kỹ thuật không dịch, ví dụ STT, mã màn hình) làm điểm khớp giữa 2 file.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum VnRowClass {
    /// Không có ô đỏ/gạch bỏ nào trong dòng — dùng làm điểm neo canh dòng.
    Unchanged,
    /// Có ô đỏ (nội dung mới) — dòng thêm mới, cần JP có dòng trống tương ứng.
    Edited,
    /// Có ô gạch bỏ, không có ô đỏ — dòng đã bị đánh dấu xóa trong VN.
    Removed,
}

/// Nhận diện ô "neo" — số/mã kỹ thuật thường KHÔNG bị dịch (STT, version, mã màn hình...),
/// dùng làm điểm khớp dòng giữa VN và JP vì không thể so trực tiếp nội dung chữ (khác ngôn ngữ).
pub(crate) fn is_anchor_cell(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() || t.chars().count() > 24 {
        return false;
    }
    let has_digit = t.chars().any(|c| c.is_ascii_digit());
    let valid_chars = t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '/'));
    has_digit && valid_chars
}

/// Khóa neo của một dòng: nối các ô "neo" theo đúng thứ tự cột. `None` nếu dòng không có neo nào.
fn row_anchor_key(row: &[String]) -> Option<String> {
    let parts: Vec<&str> = row
        .iter()
        .map(|s| s.trim())
        .filter(|s| is_anchor_cell(s))
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("|"))
    }
}

/// Phân loại từng dòng VN (0-based) theo sự hiện diện của ô đỏ/gạch bỏ trong dòng đó.
fn classify_vn_rows(
    row_count: usize,
    red_rows: &HashSet<usize>,
    strike_rows: &HashSet<usize>,
) -> Vec<VnRowClass> {
    (0..row_count)
        .map(|r| {
            if red_rows.contains(&r) {
                VnRowClass::Edited
            } else if strike_rows.contains(&r) {
                VnRowClass::Removed
            } else {
                VnRowClass::Unchanged
            }
        })
        .collect()
}

/// LCS (Longest Common Subsequence) giữa 2 dãy (index gốc, khóa neo) — trả về các cặp
/// (a_index, b_index) khớp nhau theo đúng thứ tự, dùng làm điểm neo canh dòng VN↔JP.
fn lcs_match(a: &[(usize, String)], b: &[(usize, String)]) -> Vec<(usize, usize)> {
    let n = a.len();
    let m = b.len();
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i].1 == b[j].1 {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut pairs = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if a[i].1 == b[j].1 {
            pairs.push((a[i].0, b[j].0));
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

/// Từ các cặp neo đã khớp (đã sort tăng dần theo vn_idx), duyệt qua từng khoảng hở giữa 2 neo
/// liên tiếp (kể cả đầu/cuối sheet) và đề xuất chèn dòng vào JP nếu VN có nhiều dòng hơn JP
/// trong khoảng hở đó.
fn build_alignment_suggestions(
    sheet_name: &str,
    vn_rows: &[Vec<String>],
    vn_class: &[VnRowClass],
    jp_row_count: usize,
    matched: &[(usize, usize)],
) -> Vec<RowAlignmentSuggestion> {
    let mut anchors: Vec<(isize, isize)> = matched
        .iter()
        .map(|(v, j)| (*v as isize, *j as isize))
        .collect();
    anchors.sort();
    anchors.push((vn_rows.len() as isize, jp_row_count as isize));

    let mut suggestions = Vec::new();
    let mut prev_vn: isize = -1;
    let mut prev_jp: isize = -1;

    for (vn_b, jp_b) in anchors {
        let vn_gap_len = (vn_b - prev_vn - 1).max(0) as usize;
        let jp_gap_len = (jp_b - prev_jp - 1).max(0) as usize;

        if vn_gap_len > jp_gap_len {
            let vn_gap_start = (prev_vn + 1) as usize;
            let insert_count = vn_gap_len - jp_gap_len;
            let jp_insert_after_row = (prev_jp + jp_gap_len as isize + 1) as usize;

            let range = vn_gap_start..(vn_gap_start + vn_gap_len);
            let has_red = range.clone().any(|r| vn_class[r] == VnRowClass::Edited);
            let has_strike = range.clone().any(|r| vn_class[r] == VnRowClass::Removed);
            let sample_vn_text: Vec<String> = range
                .filter_map(|r| {
                    vn_rows[r]
                        .iter()
                        .map(|c| c.trim())
                        .find(|c| !c.is_empty())
                        .map(|s| s.to_string())
                })
                .take(3)
                .collect();

            suggestions.push(RowAlignmentSuggestion {
                sheet: sheet_name.to_string(),
                jp_insert_after_row,
                insert_count,
                vn_row_start: vn_gap_start + 1,
                vn_row_end: vn_gap_start + vn_gap_len,
                sample_vn_text,
                has_red,
                has_strike,
            });
        }

        prev_vn = vn_b;
        prev_jp = jp_b;
    }

    suggestions
}

// ─────────────────────────────────────────────────────────────────────────────
// Vùng nội dung hợp lệ theo LOẠI TÀI LIỆU (C2.3.x)
//
// Quy tắc phát hiện ô đỏ/strikethrough/quality issue/canh dòng/dọn dẹp bên dưới KHÔNG đổi — điểm
// mới là mỗi loại tài liệu chỉ xét trong 1 vùng cột nội dung riêng (khác nhau giữa các loại), còn
// vùng header (row Excel 1~3, 0-based 0..2) luôn cố định và bị bỏ qua ở MỌI sheet, MỌI loại tài
// liệu. Ô/dòng ngoài vùng này (kể cả có định dạng đỏ/gạch bỏ) bị bỏ qua hoàn toàn, không tính là
// nội dung cần xử lý — xem `content_bounds_for` (bước "kiểm tra chung" áp lên kết quả quét thô).
//
// Sheet "変更履歴" (lịch sử thay đổi) có cấu trúc giống nhau ở mọi loại tài liệu nên LUÔN dùng cột
// A~K bất kể loại tài liệu đang xét, ghi đè vùng cột mặc định của loại tài liệu đó.
// ─────────────────────────────────────────────────────────────────────────────

/// Dòng dữ liệu bắt đầu (0-based) — bỏ vùng header cố định (row Excel 1~3) cho MỌI sheet/loại tài liệu.
const CONTENT_START_ROW0: usize = 3;

/// Tên sheet lịch sử thay đổi — luôn dùng cột A~K bất kể loại tài liệu đang xét.
pub(crate) const CHANGE_HISTORY_SHEET_NAME: &str = "変更履歴";
/// Cột cuối cùng (0-based, inclusive) của sheet "変更履歴" = K.
const CHANGE_HISTORY_LAST_COL0: usize = 10;

/// Loại tài liệu VN↔JP nhận diện qua tiền tố tên file — quyết định vùng cột nội dung áp dụng.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DocType {
    /// C2.3.2 プログラム処理概要図 — nội dung cột A ~ AQ.
    C232,
    /// C2.3.3 イベント詳細設計書 — nội dung cột A ~ AR.
    C233,
    /// C2.3.4 画面仕様書（編集要領） — nội dung cột A ~ M.
    C234,
    /// C2.3.5 画面仕様書（単独チェック） — nội dung cột A ~ N.
    C235,
    /// C2.3.6 画面仕様書（相関チェック） — nội dung cột A ~ N.
    C236,
    /// C2.3.8 画面間インタフェース仕様書 — nội dung cột A ~ K.
    C238,
    /// Không nhận diện được loại tài liệu qua tên file — không giới hạn cột (giữ hành vi trước khi
    /// có tính năng vùng nội dung theo loại tài liệu, tức quét toàn sheet).
    Unknown,
}

/// Tiền tố tên file (VN hoặc JP) của từng loại tài liệu — dùng để nhận diện `DocType`.
const DOC_TYPE_PREFIXES: &[(&str, DocType)] = &[
    ("C2.3.2 プログラム処理概要図", DocType::C232),
    ("C2.3.3 イベント詳細設計書", DocType::C233),
    ("C2.3.4 画面仕様書（編集要領）", DocType::C234),
    ("C2.3.5 画面仕様書（単独チェック）", DocType::C235),
    ("C2.3.6 画面仕様書（相関チェック）", DocType::C236),
    (super::c238_sync_service::SCREEN_IF_DOC_PREFIX, DocType::C238),
];

/// Nhận diện loại tài liệu qua tên file VN hoặc JP (chấp nhận cả bản VN có hậu tố "..._VN.xlsx").
pub(crate) fn detect_doc_type(vn_path: &str, jp_path: &str) -> DocType {
    for path in [vn_path, jp_path] {
        let Some(file_name) = Path::new(path).file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        for (prefix, doc_type) in DOC_TYPE_PREFIXES {
            if file_name.starts_with(prefix) {
                return *doc_type;
            }
        }
    }
    DocType::Unknown
}

/// Vùng nội dung hợp lệ của 1 sheet: dòng bắt đầu (0-based, inclusive) và cột cuối (0-based,
/// inclusive). Ô/dòng ngoài vùng này bị bỏ qua hoàn toàn ở mọi bước xử lý.
#[derive(Clone, Copy)]
pub(crate) struct ContentBounds {
    pub(crate) start_row0: usize,
    pub(crate) last_col0: usize,
}

impl ContentBounds {
    fn contains(&self, row0: usize, col0: usize) -> bool {
        row0 >= self.start_row0 && col0 <= self.last_col0
    }
}

/// Xử lý kiểm tra chung dùng bởi mọi method riêng theo loại tài liệu (`super::c232_sync_service`..
/// `super::c238_sync_service`): cùng dòng bắt đầu cố định cho mọi loại tài liệu; riêng sheet
/// "変更履歴" luôn ghim cột cuối = K bất kể loại tài liệu.
pub(crate) fn content_bounds_for_sheet(sheet_name: &str, default_last_col0: usize) -> ContentBounds {
    let last_col0 = if sheet_name == CHANGE_HISTORY_SHEET_NAME {
        CHANGE_HISTORY_LAST_COL0
    } else {
        default_last_col0
    };
    ContentBounds {
        start_row0: CONTENT_START_ROW0,
        last_col0,
    }
}

use crate::services::vnjp_sync_service::content_bounds_for;

/// Bỏ vùng cột ngoài `bounds` khỏi 1 dòng (dùng khi quét cả dòng tìm ô neo canh dòng) — không đổi
/// gì nếu không có `bounds` (loại tài liệu không nhận diện được).
fn bounded_row<'a>(row: &'a [String], bounds: Option<ContentBounds>) -> &'a [String] {
    match bounds {
        Some(b) => &row[..row.len().min(b.last_col0 + 1)],
        None => row,
    }
}

/// Giống `bounded_row` nhưng BỎ LUÔN cột A (index 0) — dùng khi tìm ô neo canh dòng
/// (`align_vn_jp_row_map`): cột A ở vùng nội dung thường là STT tự đánh số lại tuần tự, không
/// mang tín hiệu thật về chèn/xóa dòng (xem doc `align_vn_jp_row_map`).
fn anchor_slice_excl_col_a(row: &[String], bounds: ContentBounds) -> &[String] {
    let bounded = bounded_row(row, Some(bounds));
    if bounded.is_empty() {
        bounded
    } else {
        &bounded[1..]
    }
}

/// Xử lý kiểm tra chung: lọc bỏ khỏi kết quả quét thô (ô đỏ/strikethrough theo `(row0, col0)`) mọi
/// ô nằm ngoài vùng nội dung hợp lệ của sheet đó — quy tắc phát hiện đỏ/strikethrough tự thân
/// (`find_red_cells_in_sheet`/`find_strike_cells_in_sheet`) không đổi, chỉ áp thêm bước lọc này.
fn filter_cells_by_bounds(
    mut cells: HashMap<String, HashSet<(usize, usize)>>,
    doc_type: DocType,
) -> HashMap<String, HashSet<(usize, usize)>> {
    cells.retain(|sheet_name, set| {
        if let Some(bounds) = content_bounds_for(doc_type, sheet_name) {
            set.retain(|&(r, c)| bounds.contains(r, c));
        }
        !set.is_empty()
    });
    cells
}

/// Như `filter_cells_by_bounds` nhưng cho shape/textbox nổi — lọc theo vị trí Ô NEO (anchor).
fn filter_shapes_by_bounds(
    mut shapes: HashMap<String, Vec<ShapeInfo>>,
    doc_type: DocType,
) -> HashMap<String, Vec<ShapeInfo>> {
    shapes.retain(|sheet_name, list| {
        if let Some(bounds) = content_bounds_for(doc_type, sheet_name) {
            list.retain(|s| bounds.contains(s.anchor_row0, s.anchor_col0));
        }
        !list.is_empty()
    });
    shapes
}


/// Giới hạn kích thước bài toán LCS (n×m) để tránh tốn bộ nhớ/thời gian với sheet cực lớn.
const MAX_ANCHOR_LCS_CELLS: usize = 4_000_000;

/// Phát hiện các vị trí VN có dòng mà JP chưa có (lệch dòng), theo từng sheet chung giữa 2 file.
pub fn analyze_row_alignment(vn_path: &str, jp_path: &str) -> AppResult<RowAlignmentReport> {
    // Tài liệu "画面間インタフェース仕様書" có cột A là chữ Nhật (không có ô "neo" số/mã ở header) nên
    // strategy neo cũ không nhận ra nhóm mới — dùng strategy canh dòng THEO GROUP riêng cho loại này.
    if detect_doc_type(vn_path, jp_path) == DocType::C238 {
        let suggestions = super::c238_sync_service::analyze_row_alignment_by_group(vn_path, jp_path)?;
        return Ok(RowAlignmentReport { suggestions });
    }

    let doc_type = detect_doc_type(vn_path, jp_path);
    let vn_grid = read_workbook_grid(vn_path)?;
    let jp_grid = read_workbook_grid(jp_path)?;
    let vn_red = filter_cells_by_bounds(find_red_cells_xlsx(vn_path), doc_type);
    let vn_strike = filter_cells_by_bounds(find_strike_cells_xlsx(vn_path), doc_type);

    let mut suggestions = Vec::new();

    for (sheet_name, vn_rows) in &vn_grid {
        let Some(jp_rows) = jp_grid.get(sheet_name) else {
            continue;
        };
        let bounds = content_bounds_for(doc_type, sheet_name);

        let red_rows: HashSet<usize> = vn_red
            .get(sheet_name)
            .map(|s| s.iter().map(|(r, _)| *r).collect())
            .unwrap_or_default();
        let strike_rows: HashSet<usize> = vn_strike
            .get(sheet_name)
            .map(|s| s.iter().map(|(r, _)| *r).collect())
            .unwrap_or_default();

        let vn_class = classify_vn_rows(vn_rows.len(), &red_rows, &strike_rows);

        let vn_anchors: Vec<(usize, String)> = vn_rows
            .iter()
            .enumerate()
            .filter(|(r, _)| vn_class[*r] == VnRowClass::Unchanged)
            .filter(|(r, _)| bounds.map_or(true, |b| *r >= b.start_row0))
            .filter_map(|(r, row)| row_anchor_key(bounded_row(row, bounds)).map(|k| (r, k)))
            .collect();
        let jp_anchors: Vec<(usize, String)> = jp_rows
            .iter()
            .enumerate()
            .filter(|(r, _)| bounds.map_or(true, |b| *r >= b.start_row0))
            .filter_map(|(r, row)| row_anchor_key(bounded_row(row, bounds)).map(|k| (r, k)))
            .collect();

        if vn_anchors.is_empty() || jp_anchors.is_empty() {
            continue; // Không có điểm neo nào để canh dòng cho sheet này.
        }
        if vn_anchors.len().saturating_mul(jp_anchors.len()) > MAX_ANCHOR_LCS_CELLS {
            continue; // Sheet quá lớn — bỏ qua canh dòng tự động, TL tự kiểm tra thủ công.
        }

        let matched = lcs_match(&vn_anchors, &jp_anchors);
        suggestions.extend(build_alignment_suggestions(
            sheet_name,
            vn_rows,
            &vn_class,
            jp_rows.len(),
            &matched,
        ));
    }

    Ok(RowAlignmentReport { suggestions })
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
    jp_shapes: &HashMap<String, Vec<ShapeInfo>>,
    _jp_sheets: &[SheetMeta],
    doc_type: DocType,
) -> Vec<QualityIssue> {
    let vn_char_re = Regex::new(
        r"[\u{1E00}-\u{1EFF}đĐơƠưƯăĂ]",
    )
    .unwrap();

    let mut issues: Vec<QualityIssue> = Vec::new();

    for (sheet_name, grid) in jp_grid {
        let bounds = content_bounds_for(doc_type, sheet_name);
        for (r, row) in grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if bounds.map_or(false, |b| !b.contains(r, c)) {
                    continue;
                }
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
                        is_shape: false,
                    });
                }
            }
        }
    }

    for (sheet_name, shapes) in jp_shapes {
        for shape in shapes {
            for p in &shape.paragraphs {
                let cell = p.text.as_str();
                if cell.trim().is_empty() || cell.is_ascii() {
                    continue;
                }
                if vn_char_re.is_match(cell) {
                    issues.push(QualityIssue {
                        sheet: sheet_name.clone(),
                        row: shape.anchor_row0 + 1,
                        col: shape.anchor_col0 + 1,
                        issue_type: "vn_char".to_string(),
                        content: cell.to_string(),
                        description: "JP文書内にベトナム語の文字が含まれています。翻訳漏れの可能性があります。".to_string(),
                        is_shape: true,
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

pub(crate) fn read_zip_entry(archive: &mut zip::ZipArchive<File>, name: &str) -> Option<String> {
    let mut entry = archive.by_name(name).ok()?;
    let mut buf = String::new();
    entry.read_to_string(&mut buf).ok()?;
    Some(buf)
}

/// Đọc nội dung 1 entry: ưu tiên bản đã có trong `replaced` (đã seed trước vòng lặp — vd sheet
/// mới tạo từ khung "(DEL)", xem `clone_missing_sheets_from_del_template`), nếu chưa có thì đọc
/// từ archive ZIP gốc.
fn read_current(
    archive: &mut zip::ZipArchive<File>,
    replaced: &HashMap<String, Vec<u8>>,
    path: &str,
) -> Option<String> {
    if let Some(bytes) = replaced.get(path) {
        return String::from_utf8(bytes.clone()).ok();
    }
    read_zip_entry(archive, path)
}

pub(crate) fn resolve_sheet_xml_paths(workbook_xml: &str, rels_xml: &str) -> Vec<(String, String)> {
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
pub(crate) fn parse_cell_ref(cell_ref: &str) -> Option<(usize, usize)> {
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
// Xử lý textbox/shape nổi (xl/drawings/drawingN.xml). Excel cho phép chữ đỏ (VN)
// / gạch bỏ (JP) nằm trong textbox nổi trên sheet, không chỉ trong cell — vì
// vậy phải đọc riêng DrawingML (namespace a:/xdr:) thay vì cấu trúc <c>/<row>
// thường. `row`/`col` hiển thị cho shape luôn là ô NEO (anchor, xdr:from) — xem
// `RedCell::is_shape`.
//
// Đối ứng VN ↔ JP cho shape KHÔNG dùng vị trí neo để khớp — nhiều shape có thể
// dùng chung 1 neo khi nằm trong cùng 1 group (`xdr:grpSp`, ví dụ nhiều textbox
// tiêu đề gộp thành 1 khối). Thay vào đó dùng TÊN shape (`xdr:cNvPr/@name`, ví
// dụ "Text Box 13") làm khóa đối ứng — tên này do Excel gán khi tạo shape và
// thường được giữ nguyên khi JP được sao chép từ VN (cùng cấu trúc gốc).
// ─────────────────────────────────────────────────────────────────────────────

/// Một đoạn văn (`<a:p>`) trong textbox, cùng thuộc tính tổng hợp cần cho phát hiện.
struct ShapeParagraph {
    text: String,
    /// `true` nếu có ÍT NHẤT 1 run chữ đỏ (giống tiêu chí "any" của cell — xem `find_red_cells_in_sheet`).
    any_red: bool,
    /// `true` nếu có run và TOÀN BỘ run đều gạch bỏ (giống tiêu chí "all" của cell strikethrough).
    all_struck: bool,
}

/// 1 textbox (`<xdr:sp>`) nổi trên sheet, neo tại ô `anchor_row0`/`anchor_col0`.
struct ShapeInfo {
    /// Tên shape (`xdr:cNvPr/@name`) — khóa đối ứng VN↔JP, xem ghi chú đầu mục.
    name: String,
    anchor_row0: usize,
    anchor_col0: usize,
    paragraphs: Vec<ShapeParagraph>,
}

/// Kiểm tra 1 `<a:rPr>` (DrawingML) có gạch bỏ hay không.
fn is_struck_run_drawingml(rpr: &roxmltree::Node) -> bool {
    matches!(rpr.attribute("strike"), Some(v) if v != "noStrike" && !v.is_empty())
}

/// Tìm node `<a:srgbClr>` mang màu chữ của 1 run (con của `<a:solidFill>` trực tiếp trong rPr).
fn run_fill_srgb<'a, 'input>(
    rpr: &roxmltree::Node<'a, 'input>,
) -> Option<roxmltree::Node<'a, 'input>> {
    rpr.children()
        .find(|c| c.tag_name().name() == "solidFill")
        .and_then(|sf| sf.children().find(|c| c.tag_name().name() == "srgbClr"))
}

fn is_red_run_drawingml(rpr: &roxmltree::Node) -> bool {
    run_fill_srgb(rpr)
        .and_then(|c| c.attribute("val"))
        .map(is_argb_red)
        .unwrap_or(false)
}

/// Parse 1 file `xl/drawings/drawingN.xml` → danh sách shape kèm vị trí neo + nội dung từng đoạn văn.
fn parse_shapes_in_drawing(drawing_xml: &str) -> Vec<ShapeInfo> {
    let mut result = Vec::new();
    let Ok(doc) = roxmltree::Document::parse(drawing_xml) else {
        return result;
    };

    for anchor in doc.descendants().filter(|n| {
        matches!(n.tag_name().name(), "twoCellAnchor" | "oneCellAnchor")
    }) {
        let Some(from) = anchor.children().find(|c| c.tag_name().name() == "from") else {
            continue;
        };
        let anchor_row0 = from
            .children()
            .find(|c| c.tag_name().name() == "row")
            .and_then(|n| n.text())
            .and_then(|t| t.parse::<usize>().ok())
            .unwrap_or(0);
        let anchor_col0 = from
            .children()
            .find(|c| c.tag_name().name() == "col")
            .and_then(|n| n.text())
            .and_then(|t| t.parse::<usize>().ok())
            .unwrap_or(0);

        // `.descendants()` bắt được cả `<xdr:sp>` lồng bên trong `<xdr:grpSp>` — mọi shape con
        // trong 1 group đều chia sẻ CHUNG 1 neo (row/col của group), xem ghi chú đầu mục.
        for sp in anchor.descendants().filter(|n| n.tag_name().name() == "sp") {
            let name = sp
                .descendants()
                .find(|n| n.tag_name().name() == "cNvPr")
                .and_then(|n| n.attribute("name"))
                .unwrap_or("")
                .to_string();
            let Some(tx_body) = sp.children().find(|c| c.tag_name().name() == "txBody") else {
                continue;
            };

            let mut paragraphs = Vec::new();
            for p in tx_body.children().filter(|c| c.tag_name().name() == "p") {
                let mut text = String::new();
                let mut any_red = false;
                let mut run_count = 0usize;
                let mut struck_count = 0usize;
                for run in p.children().filter(|c| c.tag_name().name() == "r") {
                    let rpr = run.children().find(|c| c.tag_name().name() == "rPr");
                    let t = run
                        .children()
                        .find(|c| c.tag_name().name() == "t")
                        .and_then(|n| n.text())
                        .unwrap_or("");
                    text.push_str(t);
                    if t.trim().is_empty() {
                        continue;
                    }
                    run_count += 1;
                    if let Some(rpr) = &rpr {
                        if is_red_run_drawingml(rpr) {
                            any_red = true;
                        }
                        if is_struck_run_drawingml(rpr) {
                            struck_count += 1;
                        }
                    }
                }
                if text.trim().is_empty() {
                    continue;
                }
                paragraphs.push(ShapeParagraph {
                    text,
                    any_red,
                    all_struck: run_count > 0 && struck_count == run_count,
                });
            }

            if !paragraphs.is_empty() {
                result.push(ShapeInfo {
                    name,
                    anchor_row0,
                    anchor_col0,
                    paragraphs,
                });
            }
        }
    }

    result
}

/// "xl/worksheets/sheet1.xml" → "xl/worksheets/_rels/sheet1.xml.rels".
fn rels_path_for(xml_path: &str) -> String {
    match xml_path.rfind('/') {
        Some(idx) => format!("{}/_rels/{}.rels", &xml_path[..idx], &xml_path[idx + 1..]),
        None => format!("_rels/{xml_path}.rels"),
    }
}

/// Giải quyết target tương đối trong 1 file `.rels` (vd "../drawings/drawing1.xml", đặt trong
/// `xl/worksheets/_rels/sheet1.xml.rels`) thành đường dẫn tuyệt đối trong ZIP (vd
/// "xl/drawings/drawing1.xml"). `base_xml_path` là đường dẫn của phần XML CHỨA quan hệ này
/// (vd "xl/worksheets/sheet1.xml") — target luôn tương đối so với THƯ MỤC CHỨA phần đó.
fn resolve_relative_part_path(base_xml_path: &str, target: &str) -> String {
    if let Some(stripped) = target.strip_prefix('/') {
        return stripped.to_string();
    }
    let base_dir = match base_xml_path.rfind('/') {
        Some(idx) => &base_xml_path[..idx],
        None => "",
    };
    let mut parts: Vec<&str> = base_dir.split('/').filter(|s| !s.is_empty()).collect();
    for seg in target.split('/') {
        match seg {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            other => parts.push(other),
        }
    }
    parts.join("/")
}

/// Tìm đường dẫn ZIP của drawing XML gắn với 1 sheet, thông qua `<drawing r:id="..."/>` trong
/// sheet XML rồi tra `.rels` tương ứng. `None` nếu sheet không có shape/textbox nổi nào.
fn resolve_drawing_path(
    archive: &mut zip::ZipArchive<File>,
    sheet_xml: &str,
    sheet_xml_path: &str,
) -> Option<String> {
    let doc = roxmltree::Document::parse(sheet_xml).ok()?;
    let rid = doc
        .descendants()
        .find(|n| n.tag_name().name() == "drawing")
        .and_then(|n| {
            n.attribute((
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                "id",
            ))
        })?
        .to_string();

    let rels_path = rels_path_for(sheet_xml_path);
    let rels_xml = read_zip_entry(archive, &rels_path)?;
    let rels_doc = roxmltree::Document::parse(&rels_xml).ok()?;
    let target = rels_doc
        .descendants()
        .find(|n| {
            n.tag_name().name() == "Relationship" && n.attribute("Id") == Some(rid.as_str())
        })
        .and_then(|n| n.attribute("Target"))?;

    Some(resolve_relative_part_path(sheet_xml_path, target))
}

/// Quét toàn bộ file xlsx, trả về `sheet_name -> danh sách shape` cho MỌI sheet có textbox nổi.
fn find_shapes_xlsx(path: &str) -> HashMap<String, Vec<ShapeInfo>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return HashMap::new(),
    };

    let Some(workbook_xml) = read_zip_entry(&mut archive, "xl/workbook.xml") else {
        return HashMap::new();
    };
    let Some(rels_xml) = read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels") else {
        return HashMap::new();
    };
    let sheet_paths = resolve_sheet_xml_paths(&workbook_xml, &rels_xml);

    let mut result: HashMap<String, Vec<ShapeInfo>> = HashMap::new();
    for (sheet_name, xml_path) in sheet_paths {
        let Some(sheet_xml) = read_zip_entry(&mut archive, &xml_path) else {
            continue;
        };
        let Some(drawing_path) = resolve_drawing_path(&mut archive, &sheet_xml, &xml_path) else {
            continue;
        };
        let Some(drawing_xml) = read_zip_entry(&mut archive, &drawing_path) else {
            continue;
        };
        let shapes = parse_shapes_in_drawing(&drawing_xml);
        if !shapes.is_empty() {
            result.insert(sheet_name, shapes);
        }
    }
    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Dọn dẹp file JP: xóa hẳn nội dung strikethrough cũ (từ bản tablet cũ),
// tô đen lại chữ đỏ cũ còn tồn đọng. Đây là bước bắt buộc phải làm trước khi
// phản ánh chữ đỏ mới từ VN sang (xem TrinhTuDichThietKeChiTiet_HuongDan.xlsx, mục 3.1).
// ─────────────────────────────────────────────────────────────────────────────

/// Style context (xf indices) dùng để phân loại các ô "phẳng" (không có rich-text run)
/// theo font ở mức cell: đỏ hay strikethrough.
struct CleanupContext {
    red_xf: HashSet<usize>,
    strike_xf: HashSet<usize>,
    /// Xf có font khác đen (MỌI màu, không chỉ đỏ) — dùng riêng cho sheet "(DEL)",
    /// xem `blacken_all_cells_sheet_xml`.
    colored_xf: HashSet<usize>,
}

enum StyleFlag {
    None,
    Strike,
    Red,
}

fn build_cleanup_context(archive: &mut zip::ZipArchive<File>) -> CleanupContext {
    let styles_xml = read_zip_entry(archive, "xl/styles.xml").unwrap_or_default();
    if styles_xml.is_empty() {
        return CleanupContext {
            red_xf: HashSet::new(),
            strike_xf: HashSet::new(),
            colored_xf: HashSet::new(),
        };
    }
    let theme_colors = read_theme_colors_from_archive(archive);
    let red_font_ids = parse_red_font_ids(&styles_xml, &theme_colors);
    let red_xf = parse_red_xf_indices(&styles_xml, &red_font_ids);
    let strike_xf = parse_strike_xf_indices(&styles_xml);
    let font_infos = parse_font_infos(&styles_xml, &theme_colors);
    let colored_xf = parse_colored_xf_indices(&styles_xml, &font_infos);
    CleanupContext {
        red_xf,
        strike_xf,
        colored_xf,
    }
}

fn style_flag(s_attr: &str, ctx: &CleanupContext) -> StyleFlag {
    let s_idx: usize = if s_attr.is_empty() {
        0
    } else {
        s_attr.parse().unwrap_or(0)
    };
    if ctx.strike_xf.contains(&s_idx) {
        StyleFlag::Strike
    } else if ctx.red_xf.contains(&s_idx) {
        StyleFlag::Red
    } else {
        StyleFlag::None
    }
}

/// Escape giá trị thuộc tính XML (`&`, `<`, `>`, `"`).
pub(crate) fn xml_escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Re-serialize một element rPr con đơn giản (chỉ có attribute, không có children) — vd `<b/>`, `<sz val="11"/>`.
fn serialize_simple_element(node: &roxmltree::Node) -> String {
    let name = node.tag_name().name();
    let attrs: String = node
        .attributes()
        .map(|a| format!(" {}=\"{}\"", a.name(), xml_escape_attr(a.value())))
        .collect();
    format!("<{name}{attrs}/>")
}

/// Xây `<rPr>` mới: luôn ép màu đen, bỏ `<color>`/`<strike>` gốc, giữ nguyên các thuộc tính khác (bold, size, font...).
fn rebuild_rpr_black(rpr: Option<roxmltree::Node>) -> String {
    let mut inner = String::from(r#"<color rgb="FF000000"/>"#);
    if let Some(rpr) = rpr {
        for child in rpr.children().filter(|c| c.is_element()) {
            let name = child.tag_name().name();
            if name == "color" || name == "strike" {
                continue;
            }
            inner.push_str(&serialize_simple_element(&child));
        }
    }
    format!("<rPr>{inner}</rPr>")
}

/// Duyệt các `<r>` run con của một node rich-text (`<is>` hoặc `<si>`):
/// - Run có strikethrough → loại bỏ hẳn (xóa nội dung).
/// - Run còn lại → ép màu đen (nếu trước đó có `<color>` thì tính là "đã tô đen lại").
/// Trả về (inner XML mới, số run bị xóa, số run được tô đen lại).
/// Dọn dẹp các run rich-text: XÓA run gạch bỏ (strike), TÔ ĐEN run chữ ĐỎ (hoàn tất bản cũ), và
/// GIỮ NGUYÊN các run khác (đặc biệt là chữ XANH — đánh dấu nội dung mới/đã sửa, không được đen hoá;
/// xem `is_edit_color`). `src` là XML gốc chứa `parent` để copy nguyên văn run không đổi.
fn transform_rich_runs(parent: roxmltree::Node, src: &str) -> (String, usize, usize) {
    let mut out = String::new();
    let mut removed = 0usize;
    let mut blackened = 0usize;
    for run in parent.children().filter(|c| c.tag_name().name() == "r") {
        let rpr = run.children().find(|c| c.tag_name().name() == "rPr");
        let text = run
            .children()
            .find(|c| c.tag_name().name() == "t")
            .and_then(|t| t.text())
            .unwrap_or("")
            .to_string();

        if has_strike_element(&run) {
            if !text.trim().is_empty() {
                removed += 1;
            }
            continue;
        }

        let is_red = rpr
            .map(|p| {
                p.children().any(|c| {
                    c.tag_name().name() == "color"
                        && c.attribute("rgb").map(is_argb_red).unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if is_red {
            // Chỉ tô đen chữ ĐỎ (finalize bản cũ).
            blackened += 1;
            let new_rpr = rebuild_rpr_black(rpr);
            out.push_str(&format!(
                r#"<r>{new_rpr}<t xml:space="preserve">{}</t></r>"#,
                xml_escape(&text)
            ));
        } else {
            // Giữ NGUYÊN VĂN run (chữ xanh / đen / định dạng khác) — không đổi màu.
            out.push_str(&src[run.range()]);
        }
    }
    (out, removed, blackened)
}

// ── XML surgery helper ───────────────────────────────────────────────────────
// Áp nhiều thay đổi (thay/chèn/xóa theo byte range) lên 1 chuỗi XML gốc trong 1 lượt — sắp xếp
// giảm dần theo `start` trước khi áp để offset các edit trước không bị lệch bởi edit sau.

pub(crate) struct SurgeryEdit {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) replacement: String,
}

pub(crate) fn apply_surgery(xml: &str, mut edits: Vec<SurgeryEdit>) -> String {
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

/// Đọc `xl/sharedStrings.xml`, tô đen/xóa các `<si>` rich-text; trả về:
/// - `new_xml`: nội dung mới nếu có thay đổi
/// - đếm số run bị xóa / tô đen
/// - `rich_ssi`: tập index các `<si>` có rich-text run (được xử lý tại đây, không cần xử lý lại theo từng cell)
/// - `plain_text`: text của các `<si>` không có rich-text run (dùng để tái tạo ô khi cell đó bị flag đỏ/strike theo style)
fn cleanup_shared_strings(
    archive: &mut zip::ZipArchive<File>,
) -> Option<(String, usize, usize, HashSet<usize>, HashMap<usize, String>)> {
    let sst_xml = read_zip_entry(archive, "xl/sharedStrings.xml")?;
    let doc = roxmltree::Document::parse(&sst_xml).ok()?;

    let mut rich_ssi: HashSet<usize> = HashSet::new();
    let mut plain_text: HashMap<usize, String> = HashMap::new();
    let mut edits: Vec<SurgeryEdit> = Vec::new();
    let mut total_removed = 0usize;
    let mut total_blackened = 0usize;

    let mut si_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() != "si" {
            continue;
        }
        let has_runs = node.children().any(|c| c.tag_name().name() == "r");
        if has_runs {
            let (new_inner, removed, blackened) = transform_rich_runs(node, &sst_xml);
            if removed > 0 || blackened > 0 {
                let replacement = if new_inner.is_empty() {
                    r#"<si><t xml:space="preserve"></t></si>"#.to_string()
                } else {
                    format!("<si>{new_inner}</si>")
                };
                edits.push(SurgeryEdit {
                    start: node.range().start,
                    end: node.range().end,
                    replacement,
                });
                total_removed += removed;
                total_blackened += blackened;
            }
            rich_ssi.insert(si_idx);
        } else {
            let text = node
                .children()
                .find(|c| c.tag_name().name() == "t")
                .and_then(|t| t.text())
                .unwrap_or("")
                .to_string();
            plain_text.insert(si_idx, text);
        }
        si_idx += 1;
    }

    let new_xml = apply_surgery(&sst_xml, edits);
    Some((new_xml, total_removed, total_blackened, rich_ssi, plain_text))
}

/// Tạo một `<c>` rỗng (xóa hẳn nội dung) — dùng khi ô có strikethrough cần xóa hẳn.
fn make_empty_cell(cell_ref: &str, s_attr: &str) -> String {
    let s_part = if !s_attr.is_empty() {
        format!(" s=\"{s_attr}\"")
    } else {
        String::new()
    };
    format!(r#"<c r="{cell_ref}"{s_part}/>"#)
}

/// Tạo một `<c>` inline-string với text giữ nguyên nhưng tô màu đen — dùng khi ô có
/// chữ đỏ cũ tồn đọng (không phải rich-text run) cần tô đen lại.
fn make_black_inline_cell(cell_ref: &str, s_attr: &str, text: &str) -> String {
    let s_part = if !s_attr.is_empty() {
        format!(" s=\"{s_attr}\"")
    } else {
        String::new()
    };
    format!(
        r#"<c r="{cell_ref}"{s_part} t="inlineStr"><is><r><rPr><color rgb="FF000000"/></rPr><t xml:space="preserve">{}</t></r></is></c>"#,
        xml_escape(text)
    )
}

/// Dọn dẹp toàn bộ ô trong một sheet XML: xóa hẳn nội dung strikethrough, tô đen chữ đỏ cũ.
/// Trả về `(new_xml, strike_removed_count, red_blackened_count, skipped_count)`.
fn cleanup_sheet_xml(
    sheet_xml: &str,
    ctx: &CleanupContext,
    rich_ssi: &HashSet<usize>,
    plain_text: &HashMap<usize, String>,
    bounds: Option<ContentBounds>,
) -> (String, usize, usize, usize) {
    let doc = match roxmltree::Document::parse(sheet_xml) {
        Ok(d) => d,
        Err(_) => return (sheet_xml.to_string(), 0, 0, 0),
    };

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    let mut strike_removed = 0usize;
    let mut red_blackened = 0usize;
    let mut skipped = 0usize;

    for node in doc.descendants() {
        if node.tag_name().name() != "c" {
            continue;
        }
        let Some(cell_ref) = node.attribute("r") else {
            continue;
        };
        // Ô ngoài vùng nội dung hợp lệ (xem `content_bounds_for`) — bỏ qua hoàn toàn, không dọn dẹp.
        if let Some(b) = bounds {
            if parse_cell_ref(cell_ref).map_or(true, |(r, c)| !b.contains(r, c)) {
                continue;
            }
        }
        if node.children().any(|c| c.tag_name().name() == "f") {
            continue; // Không đụng vào ô công thức
        }
        let s_attr = node.attribute("s").unwrap_or("");
        let t_attr = node.attribute("t");

        match t_attr {
            Some("inlineStr") => {
                let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") else {
                    continue;
                };
                let has_runs = is_node.children().any(|c| c.tag_name().name() == "r");
                if has_runs {
                    let (new_inner, removed, blackened) = transform_rich_runs(is_node, sheet_xml);
                    if removed > 0 || blackened > 0 {
                        let replacement = if new_inner.is_empty() {
                            make_empty_cell(cell_ref, s_attr)
                        } else {
                            let s_part = if !s_attr.is_empty() {
                                format!(" s=\"{s_attr}\"")
                            } else {
                                String::new()
                            };
                            format!(r#"<c r="{cell_ref}"{s_part} t="inlineStr"><is>{new_inner}</is></c>"#)
                        };
                        edits.push(SurgeryEdit {
                            start: node.range().start,
                            end: node.range().end,
                            replacement,
                        });
                        strike_removed += removed;
                        red_blackened += blackened;
                    }
                } else {
                    let text = is_node
                        .children()
                        .find(|c| c.tag_name().name() == "t")
                        .and_then(|t| t.text())
                        .unwrap_or("");
                    match style_flag(s_attr, ctx) {
                        StyleFlag::Strike => {
                            edits.push(SurgeryEdit {
                                start: node.range().start,
                                end: node.range().end,
                                replacement: make_empty_cell(cell_ref, s_attr),
                            });
                            strike_removed += 1;
                        }
                        StyleFlag::Red => {
                            edits.push(SurgeryEdit {
                                start: node.range().start,
                                end: node.range().end,
                                replacement: make_black_inline_cell(cell_ref, s_attr, text),
                            });
                            red_blackened += 1;
                        }
                        StyleFlag::None => {}
                    }
                }
            }
            Some("s") => {
                let ssi = node
                    .children()
                    .find(|c| c.tag_name().name() == "v")
                    .and_then(|v| v.text())
                    .and_then(|t| t.parse::<usize>().ok());
                let Some(ssi) = ssi else { continue };
                if rich_ssi.contains(&ssi) {
                    continue; // Đã xử lý ở mức sharedStrings.xml
                }
                match style_flag(s_attr, ctx) {
                    StyleFlag::Strike => {
                        edits.push(SurgeryEdit {
                            start: node.range().start,
                            end: node.range().end,
                            replacement: make_empty_cell(cell_ref, s_attr),
                        });
                        strike_removed += 1;
                    }
                    StyleFlag::Red => {
                        let text = plain_text.get(&ssi).cloned().unwrap_or_default();
                        edits.push(SurgeryEdit {
                            start: node.range().start,
                            end: node.range().end,
                            replacement: make_black_inline_cell(cell_ref, s_attr, &text),
                        });
                        red_blackened += 1;
                    }
                    StyleFlag::None => {}
                }
            }
            _ => {
                // Ô số/boolean/khác: không tự sửa (tránh vỡ công thức/định dạng số),
                // chỉ đếm để người dùng biết cần tự kiểm tra lại.
                if matches!(style_flag(s_attr, ctx), StyleFlag::Strike | StyleFlag::Red) {
                    skipped += 1;
                }
            }
        }
    }

    (
        apply_surgery(sheet_xml, edits),
        strike_removed,
        red_blackened,
        skipped,
    )
}

/// Dọn dẹp 1 file drawing XML (textbox/shape nổi): xóa hẳn run bị gạch bỏ, tô đen run chữ đỏ
/// còn tồn đọng — tương tự `cleanup_sheet_xml` nhưng ở cấp run DrawingML (`a:r`/`a:rPr`) thay vì
/// cấp cell. Trả về `(new_xml, strike_removed_count, red_blackened_count)`.
fn cleanup_drawing_xml(drawing_xml: &str) -> (String, usize, usize) {
    let doc = match roxmltree::Document::parse(drawing_xml) {
        Ok(d) => d,
        Err(_) => return (drawing_xml.to_string(), 0, 0),
    };

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    let mut strike_removed = 0usize;
    let mut red_blackened = 0usize;

    for run in doc.descendants().filter(|n| n.tag_name().name() == "r") {
        let Some(rpr) = run.children().find(|c| c.tag_name().name() == "rPr") else {
            continue;
        };
        let text = run
            .children()
            .find(|c| c.tag_name().name() == "t")
            .and_then(|t| t.text())
            .unwrap_or("");

        if is_struck_run_drawingml(&rpr) {
            if !text.trim().is_empty() {
                edits.push(SurgeryEdit {
                    start: run.range().start,
                    end: run.range().end,
                    replacement: String::new(),
                });
                strike_removed += 1;
            }
            continue;
        }

        if let Some(srgb) = run_fill_srgb(&rpr) {
            if let Some(val_attr) = srgb.attributes().find(|a| a.name() == "val") {
                if is_argb_red(val_attr.value()) {
                    edits.push(SurgeryEdit {
                        start: val_attr.range_value().start,
                        end: val_attr.range_value().end,
                        replacement: "000000".to_string(),
                    });
                    red_blackened += 1;
                }
            }
        }
    }

    (apply_surgery(drawing_xml, edits), strike_removed, red_blackened)
}

// ─────────────────────────────────────────────────────────────────────────────
// Sheet JP có hậu tố "(DEL)" trong tên: sheet này sắp bị xóa/thay thế, KHÔNG cần dọn dẹp
// strikethrough hay phản ánh nội dung VN mới như luồng thông thường — chỉ cần bỏ màu TOÀN BỘ
// chữ trong sheet về đen (giữ nguyên mọi định dạng khác: bold, gạch bỏ, border, fill...).
// ─────────────────────────────────────────────────────────────────────────────

/// Kiểm tra tên sheet có hậu tố "(DEL)" hay không (cho phép khoảng trắng trước dấu ngoặc,
/// không phân biệt hoa/thường).
pub(crate) fn is_del_sheet_name(name: &str) -> bool {
    name.trim().to_ascii_uppercase().ends_with("(DEL)")
}

/// Kiểm tra 1 mã màu (6/8 hex ARGB/RGB) có phải màu đen (gần như đen) hay không.
fn is_argb_black(argb: &str) -> bool {
    let s = argb.trim_start_matches('#');
    let (r, g, b) = match s.len() {
        8 => (
            u8::from_str_radix(&s[2..4], 16).unwrap_or(0xFF),
            u8::from_str_radix(&s[4..6], 16).unwrap_or(0xFF),
            u8::from_str_radix(&s[6..8], 16).unwrap_or(0xFF),
        ),
        6 => (
            u8::from_str_radix(&s[0..2], 16).unwrap_or(0xFF),
            u8::from_str_radix(&s[2..4], 16).unwrap_or(0xFF),
            u8::from_str_radix(&s[4..6], 16).unwrap_or(0xFF),
        ),
        _ => return false,
    };
    r < 0x10 && g < 0x10 && b < 0x10
}

/// Giống `parse_red_xf_indices` nhưng tổng quát cho MỌI màu khác đen (không chỉ đỏ) — dùng
/// riêng cho sheet "(DEL)" (xem `is_del_sheet_name`), nơi cần bỏ màu TOÀN BỘ nội dung sheet.
fn parse_colored_xf_indices(styles_xml: &str, font_infos: &[FontInfo]) -> HashSet<usize> {
    let mut result = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(styles_xml) else {
        return result;
    };

    let is_colored = |font_id: usize| -> bool {
        font_infos
            .get(font_id)
            .and_then(|f| f.color.as_deref())
            .map(|c| !is_argb_black(c))
            .unwrap_or(false)
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
            let apply_font = node.attribute("applyFont").map_or(false, |v| v == "1" || v == "true");
            let direct = apply_font && is_colored(font_id);
            let inherited = node
                .attribute("xfId")
                .and_then(|s| s.parse::<usize>().ok())
                .and_then(|xfid| style_xf_fonts.get(xfid).copied())
                .map_or(false, is_colored);
            if direct || inherited {
                result.insert(xf_idx);
            }
            xf_idx += 1;
        }
    }

    result
}

/// Bỏ màu TOÀN BỘ nội dung 1 sheet về đen — KHÔNG xóa strikethrough, KHÔNG ghi nội dung mới,
/// chỉ đổi màu chữ. Dùng riêng cho sheet "(DEL)" (xem ghi chú đầu mục). Ô số/công thức/boolean
/// không bị đụng vào (tránh vỡ công thức/định dạng số) — đếm riêng vào skipped.
/// Trả về `(new_xml, blackened_count, skipped_count)`.
fn blacken_all_cells_sheet_xml(
    sheet_xml: &str,
    colored_xf: &HashSet<usize>,
    plain_text: &HashMap<usize, String>,
    rich_ssi: &HashSet<usize>,
    bounds: Option<ContentBounds>,
) -> (String, usize, usize) {
    let doc = match roxmltree::Document::parse(sheet_xml) {
        Ok(d) => d,
        Err(_) => return (sheet_xml.to_string(), 0, 0),
    };

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    let mut blackened = 0usize;
    let mut skipped = 0usize;

    for node in doc.descendants() {
        if node.tag_name().name() != "c" {
            continue;
        }
        let Some(cell_ref) = node.attribute("r") else {
            continue;
        };
        // Ô ngoài vùng nội dung hợp lệ (xem `content_bounds_for`) — bỏ qua hoàn toàn, không tô đen.
        if let Some(b) = bounds {
            if parse_cell_ref(cell_ref).map_or(true, |(r, c)| !b.contains(r, c)) {
                continue;
            }
        }
        if node.children().any(|c| c.tag_name().name() == "f") {
            continue; // Không đụng vào ô công thức
        }
        let s_attr = node.attribute("s").unwrap_or("");
        let s_idx = s_attr.parse::<usize>().ok();
        let t_attr = node.attribute("t");

        match t_attr {
            Some("inlineStr") => {
                let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") else {
                    continue;
                };
                let runs: Vec<_> = is_node.children().filter(|c| c.tag_name().name() == "r").collect();
                if !runs.is_empty() {
                    for run in runs {
                        let Some(rpr) = run.children().find(|c| c.tag_name().name() == "rPr") else {
                            continue;
                        };
                        let Some(color_node) = rpr.children().find(|c| c.tag_name().name() == "color")
                        else {
                            continue;
                        };
                        let Some(rgb_attr) = color_node.attributes().find(|a| a.name() == "rgb") else {
                            continue;
                        };
                        if !is_argb_black(rgb_attr.value()) {
                            let r = rgb_attr.range_value();
                            edits.push(SurgeryEdit {
                                start: r.start,
                                end: r.end,
                                replacement: "FF000000".to_string(),
                            });
                            blackened += 1;
                        }
                    }
                } else if s_idx.map_or(false, |i| colored_xf.contains(&i)) {
                    let text = is_node
                        .children()
                        .find(|c| c.tag_name().name() == "t")
                        .and_then(|t| t.text())
                        .unwrap_or("");
                    edits.push(SurgeryEdit {
                        start: node.range().start,
                        end: node.range().end,
                        replacement: make_black_inline_cell(cell_ref, s_attr, text),
                    });
                    blackened += 1;
                }
            }
            Some("s") => {
                let ssi = node
                    .children()
                    .find(|c| c.tag_name().name() == "v")
                    .and_then(|v| v.text())
                    .and_then(|t| t.parse::<usize>().ok());
                let Some(ssi) = ssi else { continue };
                if rich_ssi.contains(&ssi) {
                    continue; // Rich-text đã được tô đen ở mức sharedStrings.xml (toàn workbook).
                }
                if s_idx.map_or(false, |i| colored_xf.contains(&i)) {
                    let text = plain_text.get(&ssi).cloned().unwrap_or_default();
                    edits.push(SurgeryEdit {
                        start: node.range().start,
                        end: node.range().end,
                        replacement: make_black_inline_cell(cell_ref, s_attr, &text),
                    });
                    blackened += 1;
                }
            }
            _ => {
                // Ô số/boolean/khác: không tự sửa, chỉ đếm để người dùng biết cần tự kiểm tra lại.
                if s_idx.map_or(false, |i| colored_xf.contains(&i)) {
                    skipped += 1;
                }
            }
        }
    }

    (apply_surgery(sheet_xml, edits), blackened, skipped)
}

/// Dọn dẹp file JP: xóa hẳn nội dung strikethrough cũ + tô đen chữ đỏ cũ tồn đọng từ
/// bản tablet cũ, trên MỌI sheet. Kết quả lưu ra `output_path`.
/// Đây là bước bắt buộc thực hiện trước khi phản ánh chữ đỏ mới từ VN sang (bước riêng biệt,
/// xem `apply_changes`, cũng tự gọi bước dọn dẹp này trước khi ghi).
pub fn cleanup_jp(jp_path: &str, output_path: &str) -> AppResult<CleanupResult> {
    let ext = Path::new(jp_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext != "xlsx" && ext != "xlsm" {
        return Err(AppError::new(format!(
            "Chỉ hỗ trợ file .xlsx / .xlsm. File không hợp lệ: {jp_path}"
        )));
    }

    let doc_type = detect_doc_type(jp_path, jp_path);

    let jp_file =
        File::open(jp_path).map_err(|e| AppError::new(format!("Không mở được file JP: {e}")))?;
    let mut archive = zip::ZipArchive::new(jp_file)
        .map_err(|e| AppError::new(format!("File JP không phải ZIP hợp lệ: {e}")))?;

    let ctx = build_cleanup_context(&mut archive);
    let mut replaced: HashMap<String, Vec<u8>> = HashMap::new();
    let (rich_ssi, plain_text, mut strike_removed, mut red_blackened) =
        match cleanup_shared_strings(&mut archive) {
            Some((new_xml, removed, blackened, rich_ssi, plain_text)) => {
                if removed > 0 || blackened > 0 {
                    replaced.insert("xl/sharedStrings.xml".to_string(), new_xml.into_bytes());
                }
                (rich_ssi, plain_text, removed, blackened)
            }
            None => (HashSet::new(), HashMap::new(), 0, 0),
        };

    let workbook_xml = read_zip_entry(&mut archive, "xl/workbook.xml")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/workbook.xml trong file JP."))?;
    let rels_xml = read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/_rels/workbook.xml.rels."))?;
    let sheet_path_map = resolve_sheet_xml_paths(&workbook_xml, &rels_xml);

    let mut sheets_modified: Vec<String> = Vec::new();
    let mut skipped_count = 0usize;
    let mut del_sheet_count = 0usize;

    for (sheet_name, xml_path) in &sheet_path_map {
        let Some(sheet_xml) = read_zip_entry(&mut archive, xml_path) else {
            continue;
        };
        let bounds = content_bounds_for(doc_type, sheet_name);

        // Sheet "(DEL)": chỉ bỏ màu chữ về đen, KHÔNG xóa strikethrough, KHÔNG đụng shape —
        // xem ghi chú đầu mục "Sheet JP có hậu tố (DEL)".
        if is_del_sheet_name(sheet_name) {
            del_sheet_count += 1;
            let (new_xml, blackened, skip) = blacken_all_cells_sheet_xml(
                &sheet_xml,
                &ctx.colored_xf,
                &plain_text,
                &rich_ssi,
                bounds,
            );
            skipped_count += skip;
            if blackened > 0 {
                red_blackened += blackened;
                replaced.insert(xml_path.clone(), new_xml.into_bytes());
                sheets_modified.push(sheet_name.clone());
            }
            continue;
        }

        let (new_xml, s_removed, r_blackened, skip) =
            cleanup_sheet_xml(&sheet_xml, &ctx, &rich_ssi, &plain_text, bounds);
        skipped_count += skip;
        let mut sheet_changed = false;
        if s_removed > 0 || r_blackened > 0 {
            strike_removed += s_removed;
            red_blackened += r_blackened;
            replaced.insert(xml_path.clone(), new_xml.into_bytes());
            sheet_changed = true;
        }

        // Dọn dẹp textbox/shape nổi (nếu sheet có) — cùng nguyên tắc như cell ở trên.
        if let Some(drawing_path) = resolve_drawing_path(&mut archive, &sheet_xml, xml_path) {
            if let Some(drawing_xml) = read_zip_entry(&mut archive, &drawing_path) {
                let (new_drawing_xml, d_removed, d_blackened) = cleanup_drawing_xml(&drawing_xml);
                if d_removed > 0 || d_blackened > 0 {
                    strike_removed += d_removed;
                    red_blackened += d_blackened;
                    replaced.insert(drawing_path, new_drawing_xml.into_bytes());
                    sheet_changed = true;
                }
            }
        }

        if sheet_changed {
            sheets_modified.push(sheet_name.clone());
        }
    }

    write_output_zip(&mut archive, &replaced, output_path)?;

    Ok(CleanupResult {
        output_path: output_path.to_string(),
        sheets_modified,
        strike_removed_count: strike_removed,
        red_blackened_count: red_blackened,
        skipped_count,
        del_sheet_count,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Giữ style gốc của ô đỏ VN khi ghi sang JP (màu chữ, bold/italic, strikethrough)
// thay vì ép cứng thành 1 kiểu duy nhất (đỏ đậm) như trước — vì VN có thể có ô
// với rich-text run đan xen (vd: 1 phần gạch bỏ, 1 phần chữ đỏ mới trong cùng ô).
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct FontInfo {
    color: Option<String>,
    bold: bool,
    italic: bool,
    strike: bool,
}

/// Parse styles.xml → FontInfo cho từng font, theo đúng thứ tự trong `<fonts>`.
fn parse_font_infos(styles_xml: &str, theme_colors: &[String]) -> Vec<FontInfo> {
    let mut result = Vec::new();
    let Ok(doc) = roxmltree::Document::parse(styles_xml) else {
        return result;
    };
    for node in doc.descendants() {
        if node.tag_name().name() == "font"
            && node.parent().map(|p| p.tag_name().name()) == Some("fonts")
        {
            let mut info = FontInfo::default();
            for child in node.children().filter(|c| c.is_element()) {
                match child.tag_name().name() {
                    "color" => {
                        info.color = resolve_color_node(&child, theme_colors);
                    }
                    "b" => {
                        info.bold = child
                            .attribute("val")
                            .map_or(true, |v| v == "1" || v == "true");
                    }
                    "i" => {
                        info.italic = child
                            .attribute("val")
                            .map_or(true, |v| v == "1" || v == "true");
                    }
                    "strike" => {
                        info.strike = child
                            .attribute("val")
                            .map_or(true, |v| v == "1" || v == "true");
                    }
                    _ => {}
                }
            }
            result.push(info);
        }
    }
    result
}

/// Giống `parse_red_shared_strings` nhưng lấy luôn XML thô của các run bên trong
/// (thay vì chỉ đánh dấu index), để copy y nguyên khi ghi sang JP.
/// Trả về (ssi có run đỏ → raw XML của các run, tập hợp MỌI ssi có rich-text run bất kể màu).
/// Tập thứ 2 dùng để biết 1 cell dùng shared string rich-text hay không — rich-text LUÔN ưu
/// tiên hơn style cấp-cell, kể cả khi bản thân rich-text đó không có run nào đỏ.
pub(crate) fn parse_shared_strings_rich_info(
    sst_xml: &str,
    theme_colors: &[String],
) -> (HashMap<usize, String>, HashSet<usize>) {
    let mut red_rich = HashMap::new();
    let mut all_rich = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sst_xml) else {
        return (red_rich, all_rich);
    };

    let mut si_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "si" {
            let runs: Vec<_> = node
                .children()
                .filter(|c| c.tag_name().name() == "r")
                .collect();
            if !runs.is_empty() {
                all_rich.insert(si_idx);
                if runs.iter().any(|r| has_red_font_run(r, theme_colors)) {
                    if let (Some(first), Some(last)) = (runs.first(), runs.last()) {
                        red_rich.insert(
                            si_idx,
                            sst_xml[first.range().start..last.range().end].to_string(),
                        );
                    }
                }
            }
            si_idx += 1;
        }
    }
    (red_rich, all_rich)
}

/// Run `<r>` có strikethrough HOẶC màu chữ KHÔNG phải đen (bất kỳ màu nào, không chỉ đỏ/xanh).
/// Dùng để nhận diện ô "coi như đã thay đổi" dù nội dung text giống hệt JP — xem
/// `find_changed_style_cells_xlsx`.
fn has_changed_style_run(run_node: &roxmltree::Node, theme_colors: &[String]) -> bool {
    if has_strike_element(run_node) {
        return true;
    }
    run_node.descendants().any(|child| {
        child.tag_name().name() == "color"
            && resolve_color_node(&child, theme_colors)
                .map(|c| !is_argb_black(&c))
                .unwrap_or(false)
    })
}

/// Giống `parse_shared_strings_rich_info` nhưng trả về ssi "coi như đã thay đổi" (strikethrough
/// HOẶC màu không đen ở BẤT KỲ run nào) thay vì chỉ đỏ. Trả về `(mọi ssi rich-text, ssi đã đổi)`.
fn parse_shared_strings_changed_info(
    sst_xml: &str,
    theme_colors: &[String],
) -> (HashSet<usize>, HashSet<usize>) {
    let mut all_rich = HashSet::new();
    let mut changed = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sst_xml) else {
        return (all_rich, changed);
    };
    let mut si_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "si" {
            let runs: Vec<_> = node.children().filter(|c| c.tag_name().name() == "r").collect();
            if !runs.is_empty() {
                all_rich.insert(si_idx);
                if runs.iter().any(|r| has_changed_style_run(r, theme_colors)) {
                    changed.insert(si_idx);
                }
            }
            si_idx += 1;
        }
    }
    (all_rich, changed)
}

/// Tìm trong 1 sheet XML các ô "coi như đã thay đổi": có strikethrough HOẶC màu chữ không phải
/// đen — dù nội dung text không đổi so với JP, style khác biệt vẫn cần phản ánh sang JP thay vì
/// giữ nguyên ô JP cũ. Rich-text run được ưu tiên trước style cấp-cell, giống `find_red_cells_in_sheet`.
fn find_changed_style_cells_in_sheet(
    sheet_xml: &str,
    changed_xf: &HashSet<usize>,
    changed_ssi: &HashSet<usize>,
    all_rich_ssi: &HashSet<usize>,
    theme_colors: &[String],
) -> HashSet<(usize, usize)> {
    let mut result = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return result;
    };

    for node in doc.descendants() {
        if node.tag_name().name() != "c" {
            continue;
        }
        let Some(pos) = node.attribute("r").and_then(parse_cell_ref) else {
            continue;
        };

        let mut handled_by_rich = false;

        if node.attribute("t") == Some("s") {
            if let Some(v) = node.descendants().find(|c| c.tag_name().name() == "v") {
                if let Some(ssi) = v.text().and_then(|t| t.parse::<usize>().ok()) {
                    if all_rich_ssi.contains(&ssi) {
                        handled_by_rich = true;
                        if changed_ssi.contains(&ssi) {
                            result.insert(pos);
                        }
                    }
                }
            }
        }

        if !handled_by_rich && node.attribute("t") == Some("inlineStr") {
            if let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") {
                let runs: Vec<_> = is_node
                    .children()
                    .filter(|c| c.tag_name().name() == "r")
                    .collect();
                if !runs.is_empty() {
                    handled_by_rich = true;
                    if runs.iter().any(|r| has_changed_style_run(r, theme_colors)) {
                        result.insert(pos);
                    }
                }
            }
        }

        if handled_by_rich {
            continue;
        }

        if let Some(si) = node.attribute("s").and_then(|s| s.parse::<usize>().ok()) {
            if changed_xf.contains(&si) {
                result.insert(pos);
            }
        }
    }

    result
}

/// Quét toàn bộ file VN, trả về theo từng sheet tập vị trí ô "coi như đã thay đổi": có
/// strikethrough HOẶC màu chữ KHÔNG phải đen (mọi màu, không riêng đỏ/xanh — bao trùm luôn
/// `find_red_cells_with_style_xlsx` vì đỏ/xanh vốn không phải đen). Dùng bởi
/// `clone_vn_sheet_for_jp` (qua `c234_sync_service`/`c238_sync_service`) để quyết định GIỮ
/// NGUYÊN ô JP (khi ô KHÔNG nằm trong tập này) hay ghi đè bằng VN (khi CÓ trong tập này).
pub(crate) fn find_changed_style_cells_xlsx(path: &str) -> HashMap<String, HashSet<(usize, usize)>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return HashMap::new(),
    };

    let theme_colors = read_theme_colors_from_archive(&mut archive);

    let styles_xml = read_zip_entry(&mut archive, "xl/styles.xml").unwrap_or_default();
    let font_infos = parse_font_infos(&styles_xml, &theme_colors);
    let colored_xf = parse_colored_xf_indices(&styles_xml, &font_infos);
    let strike_xf = parse_strike_xf_indices(&styles_xml);
    let changed_xf: HashSet<usize> = colored_xf.union(&strike_xf).copied().collect();

    let sst_xml = read_zip_entry(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (all_rich_ssi, changed_ssi) = if sst_xml.is_empty() {
        (HashSet::new(), HashSet::new())
    } else {
        parse_shared_strings_changed_info(&sst_xml, &theme_colors)
    };

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
            let cells = find_changed_style_cells_in_sheet(
                &sheet_xml,
                &changed_xf,
                &changed_ssi,
                &all_rich_ssi,
                &theme_colors,
            );
            if !cells.is_empty() {
                result.insert(name, cells);
            }
        }
    }

    result
}

/// Run `<r>` có CẢ HAI: strikethrough VÀ màu chữ KHÔNG phải đen.
fn is_fully_struck_colored_run(run_node: &roxmltree::Node, theme_colors: &[String]) -> bool {
    has_strike_element(run_node)
        && run_node.descendants().any(|child| {
            child.tag_name().name() == "color"
                && resolve_color_node(&child, theme_colors)
                    .map(|c| !is_argb_black(&c))
                    .unwrap_or(false)
        })
}

/// Tìm shared string indices mà TẤT CẢ runs đều có cả strike lẫn màu không đen.
/// Trả về `(mọi ssi rich-text, ssi fully-struck-colored)`.
fn parse_shared_strings_fully_struck_colored(
    sst_xml: &str,
    theme_colors: &[String],
) -> (HashSet<usize>, HashSet<usize>) {
    let mut all_rich = HashSet::new();
    let mut fully_struck = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sst_xml) else {
        return (all_rich, fully_struck);
    };
    let mut si_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "si" {
            let runs: Vec<_> = node
                .children()
                .filter(|c| c.tag_name().name() == "r")
                .collect();
            if !runs.is_empty() {
                all_rich.insert(si_idx);
                if runs.iter().all(|r| is_fully_struck_colored_run(r, theme_colors)) {
                    fully_struck.insert(si_idx);
                }
            }
            si_idx += 1;
        }
    }
    (all_rich, fully_struck)
}

/// Tìm trong 1 sheet XML các ô mà TẤT CẢ nội dung đều bị gạch bỏ VÀ có màu không đen.
fn find_fully_struck_colored_cells_in_sheet(
    sheet_xml: &str,
    struck_colored_xf: &HashSet<usize>,
    fully_struck_ssi: &HashSet<usize>,
    all_rich_ssi: &HashSet<usize>,
    theme_colors: &[String],
) -> HashSet<(usize, usize)> {
    let mut result = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return result;
    };

    for node in doc.descendants() {
        if node.tag_name().name() != "c" {
            continue;
        }
        let Some(pos) = node.attribute("r").and_then(parse_cell_ref) else {
            continue;
        };

        let mut handled_by_rich = false;

        if node.attribute("t") == Some("s") {
            if let Some(v) = node.descendants().find(|c| c.tag_name().name() == "v") {
                if let Some(ssi) = v.text().and_then(|t| t.parse::<usize>().ok()) {
                    if all_rich_ssi.contains(&ssi) {
                        handled_by_rich = true;
                        if fully_struck_ssi.contains(&ssi) {
                            result.insert(pos);
                        }
                    }
                }
            }
        }

        if !handled_by_rich && node.attribute("t") == Some("inlineStr") {
            if let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") {
                let runs: Vec<_> = is_node
                    .children()
                    .filter(|c| c.tag_name().name() == "r")
                    .collect();
                if !runs.is_empty() {
                    handled_by_rich = true;
                    if runs.iter().all(|r| is_fully_struck_colored_run(r, theme_colors)) {
                        result.insert(pos);
                    }
                }
            }
        }

        if handled_by_rich {
            continue;
        }

        if let Some(si) = node.attribute("s").and_then(|s| s.parse::<usize>().ok()) {
            if struck_colored_xf.contains(&si) {
                result.insert(pos);
            }
        }
    }

    result
}

/// Quét file VN, trả về theo từng sheet tập vị trí ô mà TẤT CẢ nội dung đều bị gạch bỏ (strike)
/// VÀ có màu chữ KHÔNG phải đen. Các ô này được coi là "không thay đổi nội dung" — giữ nguyên
/// nội dung JP nhưng format strike/color từ VN được phản ánh qua cell style.
pub(crate) fn find_fully_struck_colored_cells_xlsx(
    path: &str,
) -> HashMap<String, HashSet<(usize, usize)>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return HashMap::new(),
    };

    let theme_colors = read_theme_colors_from_archive(&mut archive);

    let styles_xml = read_zip_entry(&mut archive, "xl/styles.xml").unwrap_or_default();
    let font_infos = parse_font_infos(&styles_xml, &theme_colors);
    let colored_xf = parse_colored_xf_indices(&styles_xml, &font_infos);
    let strike_xf = parse_strike_xf_indices(&styles_xml);
    let struck_colored_xf: HashSet<usize> = colored_xf.intersection(&strike_xf).copied().collect();

    let sst_xml =
        read_zip_entry(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (all_rich_ssi, fully_struck_ssi) = if sst_xml.is_empty() {
        (HashSet::new(), HashSet::new())
    } else {
        parse_shared_strings_fully_struck_colored(&sst_xml, &theme_colors)
    };

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
            let cells = find_fully_struck_colored_cells_in_sheet(
                &sheet_xml,
                &struck_colored_xf,
                &fully_struck_ssi,
                &all_rich_ssi,
                &theme_colors,
            );
            if !cells.is_empty() {
                result.insert(name, cells);
            }
        }
    }

    result
}

/// Tìm shared string indices mà TẤT CẢ runs đều "đã đổi" (strike HOẶC màu không đen — không cần
/// cả 2 như `parse_shared_strings_fully_struck_colored`). Trả về `(mọi ssi rich-text, ssi mà MỌI
/// run đều đã đổi)` — dùng cho rule "dòng hoàn toàn mới ở VN" (`compute_row_insertions`): một ô có
/// run màu đen/không-gạch (dù chỉ 1 phần nội dung, ví dụ tên cũ bị gạch bỏ ghép với tên mới tô đỏ
/// — phần "tên cũ" đó nếu KHÔNG gạch/không đổi màu vẫn là run đen) thì KHÔNG đủ điều kiện "toàn ô
/// đã đổi", phân biệt với `find_changed_style_cells_xlsx` (ANY run đổi màu/gạch → coi cả ô đã đổi
/// — dùng cho quyết định "giữ JP hay dùng VN", ngữ nghĩa khác, lỏng hơn).
fn parse_shared_strings_fully_changed(
    sst_xml: &str,
    theme_colors: &[String],
) -> (HashSet<usize>, HashSet<usize>) {
    let mut all_rich = HashSet::new();
    let mut fully_changed = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sst_xml) else {
        return (all_rich, fully_changed);
    };
    let mut si_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() == "si" {
            let runs: Vec<_> = node
                .children()
                .filter(|c| c.tag_name().name() == "r")
                .collect();
            if !runs.is_empty() {
                all_rich.insert(si_idx);
                if runs.iter().all(|r| has_changed_style_run(r, theme_colors)) {
                    fully_changed.insert(si_idx);
                }
            }
            si_idx += 1;
        }
    }
    (all_rich, fully_changed)
}

/// Giống `find_fully_struck_colored_cells_in_sheet` nhưng dùng tiêu chí "toàn ô đã đổi" (xem
/// `parse_shared_strings_fully_changed`).
fn find_fully_changed_cells_in_sheet(
    sheet_xml: &str,
    changed_xf: &HashSet<usize>,
    fully_changed_ssi: &HashSet<usize>,
    all_rich_ssi: &HashSet<usize>,
    theme_colors: &[String],
) -> HashSet<(usize, usize)> {
    let mut result = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return result;
    };

    for node in doc.descendants() {
        if node.tag_name().name() != "c" {
            continue;
        }
        let Some(pos) = node.attribute("r").and_then(parse_cell_ref) else {
            continue;
        };

        let mut handled_by_rich = false;

        if node.attribute("t") == Some("s") {
            if let Some(v) = node.descendants().find(|c| c.tag_name().name() == "v") {
                if let Some(ssi) = v.text().and_then(|t| t.parse::<usize>().ok()) {
                    if all_rich_ssi.contains(&ssi) {
                        handled_by_rich = true;
                        if fully_changed_ssi.contains(&ssi) {
                            result.insert(pos);
                        }
                    }
                }
            }
        }

        if !handled_by_rich && node.attribute("t") == Some("inlineStr") {
            if let Some(is_node) = node.children().find(|c| c.tag_name().name() == "is") {
                let runs: Vec<_> = is_node
                    .children()
                    .filter(|c| c.tag_name().name() == "r")
                    .collect();
                if !runs.is_empty() {
                    handled_by_rich = true;
                    if runs.iter().all(|r| has_changed_style_run(r, theme_colors)) {
                        result.insert(pos);
                    }
                }
            }
        }

        if handled_by_rich {
            continue;
        }

        if let Some(si) = node.attribute("s").and_then(|s| s.parse::<usize>().ok()) {
            if changed_xf.contains(&si) {
                result.insert(pos);
            }
        }
    }

    result
}

/// Quét file VN, trả về theo từng sheet tập vị trí ô mà TOÀN BỘ nội dung (mọi run, không phải chỉ
/// 1 phần) đã đổi (đỏ/màu khác đen HOẶC gạch bỏ) — dùng cho rule "dòng hoàn toàn mới ở VN" của
/// `compute_row_insertions`. Nghiêm ngặt hơn `find_changed_style_cells_xlsx` (ANY-based): một ô có
/// LẪN run chữ đen bình thường (ví dụ tên cũ bị gạch bỏ ghép tên mới tô đỏ trong CÙNG 1 ô — phần
/// tên cũ nếu không tự nó gạch/đổi màu) sẽ KHÔNG nằm trong tập này.
pub(crate) fn find_fully_changed_cells_xlsx(path: &str) -> HashMap<String, HashSet<(usize, usize)>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return HashMap::new(),
    };

    let theme_colors = read_theme_colors_from_archive(&mut archive);

    let styles_xml = read_zip_entry(&mut archive, "xl/styles.xml").unwrap_or_default();
    let font_infos = parse_font_infos(&styles_xml, &theme_colors);
    let colored_xf = parse_colored_xf_indices(&styles_xml, &font_infos);
    let strike_xf = parse_strike_xf_indices(&styles_xml);
    let changed_xf: HashSet<usize> = colored_xf.union(&strike_xf).copied().collect();

    let sst_xml = read_zip_entry(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (all_rich_ssi, fully_changed_ssi) = if sst_xml.is_empty() {
        (HashSet::new(), HashSet::new())
    } else {
        parse_shared_strings_fully_changed(&sst_xml, &theme_colors)
    };

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
            let cells = find_fully_changed_cells_in_sheet(
                &sheet_xml,
                &changed_xf,
                &fully_changed_ssi,
                &all_rich_ssi,
                &theme_colors,
            );
            if !cells.is_empty() {
                result.insert(name, cells);
            }
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Verify data presence: so sánh sự có mặt của dữ liệu (có/không) tại từng ô giữa file VN và
// file output — KHÔNG so sánh nội dung, chỉ kiểm tra ô có dữ liệu hay trống. Dùng để phát hiện
// các ô bị mất dữ liệu sau quá trình chuẩn hoá / merge (xem `apply_dictionary_and_verify_data`).
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// Từ điển replace: thu thập cặp VN text → JP text từ các ô mà output đã giữ nguyên nội dung JP
// (ô VN "không thay đổi" hoặc "fully struck colored") để tái sử dụng cho các tài liệu khác.
// ─────────────────────────────────────────────────────────────────────────────

/// Trích plain text từ 1 cell XML bất kể loại (shared string / inline string / value).
fn extract_cell_plain_text(
    cell: roxmltree::Node,
    plain_ssi: &HashMap<usize, String>,
) -> Option<String> {
    if cell.attribute("t") == Some("s") {
        let v = cell.children().find(|c| c.tag_name().name() == "v")?;
        let ssi = v.text()?.parse::<usize>().ok()?;
        return plain_ssi.get(&ssi).cloned();
    }
    if cell.attribute("t") == Some("inlineStr") {
        let is_node = cell.children().find(|c| c.tag_name().name() == "is")?;
        let mut text = String::new();
        for child in is_node.children() {
            match child.tag_name().name() {
                "t" => {
                    if let Some(t) = child.text() {
                        text.push_str(t);
                    }
                }
                "r" => {
                    if let Some(t) = child.children().find(|c| c.tag_name().name() == "t") {
                        if let Some(txt) = t.text() {
                            text.push_str(txt);
                        }
                    }
                }
                _ => {}
            }
        }
        if text.is_empty() {
            return None;
        }
        return Some(text);
    }
    let v = cell.children().find(|c| c.tag_name().name() == "v")?;
    v.text().map(|s| s.to_string())
}

/// So sánh nội dung text giữa file VN và file output, thu thập các cặp (vn_text → jp_text)
/// tại các ô mà nội dung khác nhau (output đã giữ bản JP thay vì dùng VN).
/// Kết quả dùng làm từ điển replace cho tài liệu khác.
pub(crate) fn build_replace_dictionary(
    vn_path: &str,
    output_path: &str,
    bounds_for_sheet: impl Fn(&str) -> (usize, usize),
) -> AppResult<HashMap<String, String>> {
    let vn_file = File::open(vn_path)
        .map_err(|e| AppError::new(format!("Không mở được file VN: {e}")))?;
    let mut vn_archive = zip::ZipArchive::new(vn_file)
        .map_err(|e| AppError::new(format!("File VN không phải ZIP hợp lệ: {e}")))?;

    let out_file = File::open(output_path)
        .map_err(|e| AppError::new(format!("Không mở được file output: {e}")))?;
    let mut out_archive = zip::ZipArchive::new(out_file)
        .map_err(|e| AppError::new(format!("File output không phải ZIP hợp lệ: {e}")))?;

    let vn_sst_xml =
        read_zip_entry(&mut vn_archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (vn_plain_ssi, _) = extract_all_shared_strings(&vn_sst_xml);

    let out_sst_xml =
        read_zip_entry(&mut out_archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (out_plain_ssi, _) = extract_all_shared_strings(&out_sst_xml);

    let vn_wb = read_zip_entry(&mut vn_archive, "xl/workbook.xml").unwrap_or_default();
    let vn_rels =
        read_zip_entry(&mut vn_archive, "xl/_rels/workbook.xml.rels").unwrap_or_default();
    let vn_sheet_map: HashMap<String, String> =
        resolve_sheet_xml_paths(&vn_wb, &vn_rels).into_iter().collect();

    let out_wb = read_zip_entry(&mut out_archive, "xl/workbook.xml").unwrap_or_default();
    let out_rels =
        read_zip_entry(&mut out_archive, "xl/_rels/workbook.xml.rels").unwrap_or_default();
    let out_sheet_map: HashMap<String, String> =
        resolve_sheet_xml_paths(&out_wb, &out_rels).into_iter().collect();

    let mut dict: HashMap<String, String> = HashMap::new();

    let mut sheet_names: Vec<&String> = vn_sheet_map.keys().collect();
    sheet_names.sort();

    for sheet_name in sheet_names {
        if is_del_sheet_name(sheet_name) {
            continue;
        }
        let Some(out_xml_path) = out_sheet_map.get(sheet_name.as_str()) else {
            continue;
        };
        let vn_xml_path = &vn_sheet_map[sheet_name];

        let Some(vn_xml) = read_zip_entry(&mut vn_archive, vn_xml_path) else {
            continue;
        };
        let Some(out_xml) = read_zip_entry(&mut out_archive, out_xml_path) else {
            continue;
        };

        let (start_row1, max_col0) = bounds_for_sheet(sheet_name);

        let Ok(out_doc) = roxmltree::Document::parse(&out_xml) else {
            continue;
        };
        let mut out_text_map: HashMap<(usize, usize), String> = HashMap::new();
        if let Some(sd) = out_doc
            .descendants()
            .find(|n| n.tag_name().name() == "sheetData")
        {
            for row in sd.children().filter(|n| n.tag_name().name() == "row") {
                let row1 = row
                    .attribute("r")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                if row1 < start_row1 {
                    continue;
                }
                for cell in row.children().filter(|n| n.tag_name().name() == "c") {
                    if let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) {
                        if col0 > max_col0 {
                            continue;
                        }
                        if let Some(text) = extract_cell_plain_text(cell, &out_plain_ssi) {
                            let trimmed = text.trim().to_string();
                            if !trimmed.is_empty() {
                                out_text_map.insert((row1, col0), trimmed);
                            }
                        }
                    }
                }
            }
        }

        let Ok(vn_doc) = roxmltree::Document::parse(&vn_xml) else {
            continue;
        };
        if let Some(sd) = vn_doc
            .descendants()
            .find(|n| n.tag_name().name() == "sheetData")
        {
            for row in sd.children().filter(|n| n.tag_name().name() == "row") {
                let row1 = row
                    .attribute("r")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                if row1 < start_row1 {
                    continue;
                }
                for cell in row.children().filter(|n| n.tag_name().name() == "c") {
                    if let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) {
                        if col0 > max_col0 {
                            continue;
                        }
                        if let Some(vn_text) = extract_cell_plain_text(cell, &vn_plain_ssi) {
                            let vn_trimmed = vn_text.trim().to_string();
                            if vn_trimmed.is_empty() {
                                continue;
                            }
                            if let Some(out_text) = out_text_map.get(&(row1, col0)) {
                                if &vn_trimmed != out_text {
                                    dict.entry(vn_trimmed)
                                        .or_insert_with(|| out_text.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(dict)
}

/// Áp dụng từ điển replace lên file xlsx, ĐỒNG THỜI kiểm tra sự có mặt dữ liệu (có/không) so
/// với `vn_path` tại từng ô — gộp chung 1 lượt loop sheet → row → cell (thay vì 2 lượt scan
/// riêng của "áp từ điển" và "kiểm tra output" trước đây) để tránh parse/duyệt lại cùng 1 sheet
/// XML hai lần. Với mỗi ô trong vùng nội dung, theo đúng thứ tự:
/// 1. Áp từ điển — nội dung text khớp chính xác (exact match) với 1 key trong `dictionary` thì
///    thay bằng value tương ứng (đếm vào số ô đã thay thế).
/// 2. Kiểm tra output — so sự có mặt dữ liệu (có/không, không so nội dung) với ô cùng vị trí ở
///    `vn_path`, ghi nhận mismatch nếu khác nhau.
///
/// Trả về `(số ô đã thay thế, danh sách mismatch)`; ghi file đã áp dụng ra `output_path` (an
/// toàn cả khi `output_path` trùng `file_path`, xem [`write_output_zip`]).
pub(crate) fn apply_dictionary_and_verify_data(
    file_path: &str,
    vn_path: &str,
    output_path: &str,
    dictionary: &HashMap<String, String>,
    bounds_for_sheet: impl Fn(&str) -> (usize, usize),
) -> AppResult<(usize, Vec<CellDataMismatch>)> {
    let vn_file = File::open(vn_path)
        .map_err(|e| AppError::new(format!("Không mở được file VN: {e}")))?;
    let mut vn_archive = zip::ZipArchive::new(vn_file)
        .map_err(|e| AppError::new(format!("File VN không phải ZIP hợp lệ: {e}")))?;
    let vn_wb = read_zip_entry(&mut vn_archive, "xl/workbook.xml").unwrap_or_default();
    let vn_rels =
        read_zip_entry(&mut vn_archive, "xl/_rels/workbook.xml.rels").unwrap_or_default();
    let vn_sheet_map: HashMap<String, String> =
        resolve_sheet_xml_paths(&vn_wb, &vn_rels).into_iter().collect();
    let vn_sst_xml =
        read_zip_entry(&mut vn_archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (vn_plain_ssi, _) = extract_all_shared_strings(&vn_sst_xml);

    let file = File::open(file_path)
        .map_err(|e| AppError::new(format!("Không mở được file: {e}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::new(format!("File không phải ZIP hợp lệ: {e}")))?;

    let sst_xml =
        read_zip_entry(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (plain_ssi, _) = extract_all_shared_strings(&sst_xml);

    let wb_xml = read_zip_entry(&mut archive, "xl/workbook.xml").unwrap_or_default();
    let rels_xml =
        read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels").unwrap_or_default();
    let sheet_map: HashMap<String, String> =
        resolve_sheet_xml_paths(&wb_xml, &rels_xml).into_iter().collect();

    let mut replaced: HashMap<String, Vec<u8>> = HashMap::new();
    let mut applied_count = 0usize;
    let mut mismatches: Vec<CellDataMismatch> = Vec::new();

    let mut sheet_names: Vec<(&String, &String)> = sheet_map.iter().collect();
    sheet_names.sort_by_key(|(name, _)| name.as_str());

    for (sheet_name, xml_path) in sheet_names {
        if is_del_sheet_name(sheet_name) {
            continue;
        }
        let Some(sheet_xml) = read_zip_entry(&mut archive, xml_path) else {
            continue;
        };

        let (start_row1, max_col0) = bounds_for_sheet(sheet_name);

        // Vị trí + nội dung text (best-effort) của các ô VN có dữ liệu, trong vùng nội dung —
        // dùng để so verify VÀ hiển thị nội dung ô bị lệch (ô output đang thiếu).
        let mut vn_filtered: HashSet<(usize, usize)> = HashSet::new();
        let mut vn_text: HashMap<(usize, usize), String> = HashMap::new();
        if let Some(vn_xml) = vn_sheet_map
            .get(sheet_name.as_str())
            .and_then(|p| read_zip_entry(&mut vn_archive, p))
        {
            if let Ok(vn_doc) = roxmltree::Document::parse(&vn_xml) {
                if let Some(vn_sd) = vn_doc
                    .descendants()
                    .find(|n| n.tag_name().name() == "sheetData")
                {
                    for row in vn_sd.children().filter(|n| n.tag_name().name() == "row") {
                        let row1 = row
                            .attribute("r")
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(0);
                        if row1 < start_row1 {
                            continue;
                        }
                        for cell in row.children().filter(|n| n.tag_name().name() == "c") {
                            let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) else {
                                continue;
                            };
                            if col0 > max_col0 {
                                continue;
                            }
                            let has_data = cell
                                .children()
                                .any(|ch| matches!(ch.tag_name().name(), "v" | "is" | "f"));
                            if !has_data {
                                continue;
                            }
                            vn_filtered.insert((row1, col0));
                            if let Some(text) = extract_cell_plain_text(cell, &vn_plain_ssi) {
                                let trimmed = text.trim();
                                if !trimmed.is_empty() {
                                    vn_text.insert((row1, col0), trimmed.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        let Ok(doc) = roxmltree::Document::parse(&sheet_xml) else {
            continue;
        };
        let Some(sd) = doc
            .descendants()
            .find(|n| n.tag_name().name() == "sheetData")
        else {
            continue;
        };

        let mut edits: Vec<SurgeryEdit> = Vec::new();
        let mut out_filtered: HashSet<(usize, usize)> = HashSet::new();
        // Mọi ô <c> có mặt trong XML output, bất kể có dữ liệu hay không — dùng để xác định ô
        // nào THỰC SỰ không tồn tại trong output (khác với ô có mặt nhưng rỗng, đã được đối chiếu
        // ở vòng lặp chính bên dưới; nếu chỉ diff với out_filtered thì ô rỗng-nhưng-có-mặt sẽ bị
        // tính mismatch 2 lần — 1 lần ở vòng lặp chính, 1 lần ở vòng lặp diff phía dưới).
        let mut out_present: HashSet<(usize, usize)> = HashSet::new();

        for row in sd.children().filter(|n| n.tag_name().name() == "row") {
            let row1 = row
                .attribute("r")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            if row1 < start_row1 {
                continue;
            }
            for cell in row.children().filter(|n| n.tag_name().name() == "c") {
                let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) else {
                    continue;
                };
                if col0 > max_col0 {
                    continue;
                }
                out_present.insert((row1, col0));

                let has_data = cell
                    .children()
                    .any(|ch| matches!(ch.tag_name().name(), "v" | "is" | "f"));
                if has_data {
                    out_filtered.insert((row1, col0));
                }

                // 1. Áp từ điển replace.
                if let Some(text) = extract_cell_plain_text(cell, &plain_ssi) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if let Some(jp_text) = dictionary.get(trimmed) {
                            let cell_ref = cell.attribute("r").unwrap_or("");
                            let s_attr = cell
                                .attribute("s")
                                .map(|s| format!(" s=\"{s}\""))
                                .unwrap_or_default();
                            let new_cell = format!(
                                r#"<c r="{cell_ref}"{s_attr} t="inlineStr"><is><t xml:space="preserve">{}</t></is></c>"#,
                                xml_escape(jp_text)
                            );
                            edits.push(SurgeryEdit {
                                start: cell.range().start,
                                end: cell.range().end,
                                replacement: new_cell,
                            });
                            applied_count += 1;
                        }
                    }
                }

                // 2. Kiểm tra output: so sự có mặt dữ liệu với VN tại đúng ô này.
                let vn_has_data = vn_filtered.contains(&(row1, col0));
                if vn_has_data != has_data {
                    // Nội dung hiển thị lấy từ phía ĐANG có dữ liệu (phía kia rỗng theo định nghĩa).
                    let content = if has_data {
                        extract_cell_plain_text(cell, &plain_ssi)
                            .map(|t| t.trim().to_string())
                            .unwrap_or_default()
                    } else {
                        vn_text.get(&(row1, col0)).cloned().unwrap_or_default()
                    };
                    mismatches.push(CellDataMismatch {
                        sheet: sheet_name.clone(),
                        cell_ref: format!("{}{}", col_index_to_letter(col0), row1),
                        vn_has_data,
                        output_has_data: has_data,
                        content,
                    });
                }
            }
        }

        // Ô có dữ liệu ở VN nhưng không xuất hiện trong sheetData của output (không chỉ khác
        // nội dung — hoàn toàn không có ô này) — vòng lặp trên chỉ duyệt được ô CÓ MẶT ở output.
        for &(r1, c0) in vn_filtered.difference(&out_present) {
            mismatches.push(CellDataMismatch {
                sheet: sheet_name.clone(),
                cell_ref: format!("{}{}", col_index_to_letter(c0), r1),
                vn_has_data: true,
                output_has_data: false,
                content: vn_text.get(&(r1, c0)).cloned().unwrap_or_default(),
            });
        }

        if !edits.is_empty() {
            let new_xml = apply_surgery(&sheet_xml, edits);
            replaced.insert(xml_path.clone(), new_xml.into_bytes());
        }
    }

    mismatches.sort_by(|a, b| a.sheet.cmp(&b.sheet).then(a.cell_ref.cmp(&b.cell_ref)));
    write_output_zip(&mut archive, &replaced, output_path)?;
    Ok((applied_count, mismatches))
}

// ─────────────────────────────────────────────────────────────────────────────
// Sheet chỉ có ở VN sang JP: BẤT KỲ sheet nào tồn tại ở VN mà JP chưa có (trừ sheet VN tự đánh
// dấu đã xóa, xem `is_del_sheet_name` + `compute_del_renames` bên dưới) sẽ được CLONE TRỰC TIẾP
// TỪ VN — toàn bộ nội dung (kể cả ô không đỏ, vd header), định dạng (chiều cao dòng, wrap text,
// border, fill, merge cell), số dòng, và textbox/shape nổi (kèm ảnh nhúng nếu có) đều lấy nguyên
// từ VN, để khớp 100% với VN. Vì font/fill/border/numFmt của VN không tồn tại sẵn trong
// styles.xml JP, phải MERGE (chỉ APPEND, không sửa index hiện có — an toàn cho các sheet JP
// khác) trước khi remap `s=` của sheet mới theo bảng đã merge.
//
// Sheet mới này KHÔNG đi qua `cleanup_sheet_xml`/`inject_cells_into_sheet_xml` như sheet thường
// trong vòng lặp chính — nội dung (kể cả ô đỏ) đã đúng 100% từ VN ngay khi clone, chạy lại 2
// bước đó sẽ vô tình tô đen nhầm ô đỏ mới hoặc ghi đè mất style vừa merge.
//
// Việc này HOÀN TOÀN ĐỘC LẬP với xử lý sheet "(DEL)" (xem `is_del_sheet_name` — chỉ đơn giản là
// 1 sheet JP sắp bị xóa, được bỏ màu chữ về đen như bình thường, không liên quan gì đến sheet mới
// được tạo ở đây).
// ─────────────────────────────────────────────────────────────────────────────

/// Bỏ hậu tố "(DEL)" (xem `is_del_sheet_name`) khỏi tên sheet, trả về tên gốc. Không nhạy
/// hoa/thường, cho phép có/không khoảng trắng trước dấu ngoặc.
fn strip_del_suffix(name: &str) -> String {
    let trimmed = name.trim();
    let upper = trimmed.to_ascii_uppercase();
    match upper.rfind("(DEL)") {
        Some(idx) => trimmed[..idx].trim_end().to_string(),
        None => trimmed.to_string(),
    }
}

/// Tính danh sách sheet JP cần đổi tên thêm hậu tố "(DEL)": VN đánh dấu 1 sheet đã bị xóa bằng
/// cách đặt tên sheet đó kết thúc bằng "(DEL)" — nếu JP vẫn còn sheet tên GỐC (chưa đổi) thì đổi
/// tên sheet đó trong JP cho khớp, KHÔNG xóa hẳn sheet. Trả về `Vec<(tên JP cũ, tên JP mới)>`.
fn compute_del_renames(
    vn_sheet_names: &[String],
    jp_sheet_names: &HashSet<String>,
) -> Vec<(String, String)> {
    let mut renames = Vec::new();
    for name in vn_sheet_names {
        if !is_del_sheet_name(name) {
            continue;
        }
        let base = strip_del_suffix(name);
        if base.is_empty() {
            continue;
        }
        if jp_sheet_names.contains(&base) {
            renames.push((base.clone(), format!("{base} (DEL)")));
        }
    }
    renames
}

/// Đổi tên 1 `<sheet>` trong `xl/workbook.xml` JP cho khớp `compute_del_renames`. Không đụng gì
/// khác (rId, sheetId, thứ tự...) — xem `reorder_sheets_to_match_vn` cho bước sắp xếp lại sau đó.
fn rename_sheet_in_workbook_xml(workbook_xml: &str, old_name: &str, new_name: &str) -> String {
    let Ok(doc) = roxmltree::Document::parse(workbook_xml) else {
        return workbook_xml.to_string();
    };
    let Some(sheet_node) = doc
        .descendants()
        .find(|n| n.tag_name().name() == "sheet" && n.attribute("name") == Some(old_name))
    else {
        return workbook_xml.to_string();
    };
    let Some(name_attr) = sheet_node.attributes().find(|a| a.name() == "name") else {
        return workbook_xml.to_string();
    };
    let r = name_attr.range_value();
    apply_surgery(
        workbook_xml,
        vec![SurgeryEdit {
            start: r.start,
            end: r.end,
            replacement: xml_escape(new_name),
        }],
    )
}

/// Đọc nhị phân 1 entry trong ZIP (dùng cho ảnh nhúng — không thể đọc dạng String như
/// `read_zip_entry` vì dữ liệu ảnh không phải UTF-8).
fn read_zip_entry_bytes(archive: &mut zip::ZipArchive<File>, name: &str) -> Option<Vec<u8>> {
    let mut entry = archive.by_name(name).ok()?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// Trích toàn bộ nội dung `sharedStrings.xml` — dùng khi clone sheet sang file KHÁC, vì không
/// thể giữ nguyên chỉ số `t="s"` gốc (bảng sharedStrings của VN và JP là 2 bảng hoàn toàn khác
/// nhau). Trả về `(ssi -> text thuần cho si KHÔNG rich-text, ssi -> raw XML run cho si CÓ
/// rich-text)` — dùng để dựng lại cell dưới dạng inline string, không phụ thuộc bảng gốc nữa.
pub(crate) fn extract_all_shared_strings(sst_xml: &str) -> (HashMap<usize, String>, HashMap<usize, String>) {
    let mut plain = HashMap::new();
    let mut rich_raw = HashMap::new();
    let Ok(doc) = roxmltree::Document::parse(sst_xml) else {
        return (plain, rich_raw);
    };

    let mut si_idx = 0usize;
    for node in doc.descendants() {
        if node.tag_name().name() != "si" {
            continue;
        }
        let runs: Vec<_> = node.children().filter(|c| c.tag_name().name() == "r").collect();
        if !runs.is_empty() {
            if let (Some(first), Some(last)) = (runs.first(), runs.last()) {
                rich_raw.insert(si_idx, sst_xml[first.range().start..last.range().end].to_string());
            }
            // Cũng trích plain text từ các run (nối toàn bộ <t>) để có thể so khớp nội dung
            let rich_plain: String = runs
                .iter()
                .flat_map(|r| r.children())
                .filter(|c| c.tag_name().name() == "t")
                .filter_map(|t| t.text())
                .collect::<Vec<_>>()
                .join("");
            if !rich_plain.is_empty() {
                plain.insert(si_idx, rich_plain);
            }
        } else {
            let text = node
                .children()
                .find(|c| c.tag_name().name() == "t")
                .and_then(|t| t.text())
                .unwrap_or("")
                .to_string();
            plain.insert(si_idx, text);
        }
        si_idx += 1;
    }
    (plain, rich_raw)
}

/// Kết quả merge styles.xml của VN vào JP khi clone sheet.
pub(crate) struct StyleMergeResult {
    pub(crate) new_styles_xml: String,
    /// Chỉ số cellXfs GỐC của VN (0-based) → chỉ số MỚI trong styles.xml JP đã merge — dùng để
    /// remap thuộc tính `s=`/`style=` của mọi cell/row/col trong sheet VN được clone.
    pub(crate) xf_remap: Vec<usize>,
    /// Border của mọi xf trong styles.xml đã merge, tra theo chỉ số CUỐI CÙNG (JP gốc
    /// 0..cell_xfs_offset, VN đã remap từ cell_xfs_offset..) — input cho `BorderUnionExtender`.
    pub(crate) xf_border_lookup: HashMap<usize, BorderDef>,
    /// Raw `<xf .../>` (đã remap) theo chỉ số VN GỐC (0-based, trước khi cộng `cell_xfs_offset`).
    pub(crate) vn_xf_raw: Vec<String>,
    pub(crate) cell_xfs_offset: usize,
    pub(crate) borders_count: usize,
    pub(crate) cell_xfs_count: usize,
}

/// Tìm phần tử con trực tiếp có tên `tag` — dùng hàm rời (không phải closure) để tránh vướng
/// lifetime khi gọi với 2 `Document` khác nhau (JP/VN) trong cùng 1 hàm.
fn find_child<'a, 'input>(parent: roxmltree::Node<'a, 'input>, tag: &str) -> Option<roxmltree::Node<'a, 'input>> {
    parent.children().find(|c| c.tag_name().name() == tag)
}

fn count_child_elements(node: Option<roxmltree::Node>, child_tag: &str) -> usize {
    node.map(|n| n.children().filter(|c| c.tag_name().name() == child_tag).count())
        .unwrap_or(0)
}

/// Copy verbatim toàn bộ phần tử con của `node` (dùng cho fonts/fills/borders — tự chứa hoàn
/// toàn, không tham chiếu bảng khác nên không cần remap).
fn copy_children_raw(node: Option<roxmltree::Node>, src_xml: &str) -> String {
    node.map(|n| {
        n.children()
            .filter(|c| c.is_element())
            .map(|c| src_xml[c.range()].to_string())
            .collect::<String>()
    })
    .unwrap_or_default()
}

/// Remap attribute (numFmtId/fontId/fillId/borderId[/xfId]) của 1 node `<xf>`, giữ nguyên mọi
/// phần khác (children như `<alignment>`/`<protection>` — nơi wrapText/border thật sự được khai
/// báo, giữ nguyên y hệt VN, thuộc tính applyXxx...).
fn remap_xf_element(
    node: roxmltree::Node,
    src_xml: &str,
    fonts_offset: usize,
    fills_offset: usize,
    borders_offset: usize,
    cell_style_xfs_offset: usize,
    remap_xfid: bool,
    numfmt_remap: &HashMap<usize, usize>,
) -> String {
    let range = node.range();
    let raw = &src_xml[range.clone()];
    let mut edits: Vec<SurgeryEdit> = Vec::new();
    for attr in node.attributes() {
        let new_val = match attr.name() {
            "numFmtId" => attr.value().parse::<usize>().ok().map(|v| {
                if v < 164 { v } else { *numfmt_remap.get(&v).unwrap_or(&v) }.to_string()
            }),
            "fontId" => attr.value().parse::<usize>().ok().map(|v| (v + fonts_offset).to_string()),
            "fillId" => attr.value().parse::<usize>().ok().map(|v| (v + fills_offset).to_string()),
            "borderId" => attr
                .value()
                .parse::<usize>()
                .ok()
                .map(|v| (v + borders_offset).to_string()),
            "xfId" if remap_xfid => attr
                .value()
                .parse::<usize>()
                .ok()
                .map(|v| (v + cell_style_xfs_offset).to_string()),
            _ => None,
        };
        if let Some(new_val) = new_val {
            let r = attr.range_value();
            edits.push(SurgeryEdit {
                start: r.start - range.start,
                end: r.end - range.start,
                replacement: new_val,
            });
        }
    }
    apply_surgery(raw, edits)
}

/// Cập nhật `count="N"` và chèn thêm nội dung mới vào cuối 1 section của styles.xml JP
/// (fonts/fills/borders/cellStyleXfs/cellXfs) — chỉ APPEND, không đụng record hiện có.
fn append_style_section(
    edits: &mut Vec<SurgeryEdit>,
    jp_styles_xml: &str,
    node: Option<roxmltree::Node>,
    added_raw: &str,
    added_count: usize,
) {
    if added_count == 0 {
        return;
    }
    let Some(node) = node else { return };
    if let Some(count_attr) = node.attributes().find(|a| a.name() == "count") {
        let old_count: usize = count_attr.value().parse().unwrap_or(0);
        let r = count_attr.range_value();
        edits.push(SurgeryEdit {
            start: r.start,
            end: r.end,
            replacement: (old_count + added_count).to_string(),
        });
    }
    if let Some(insert_pos) = jp_styles_xml[..node.range().end].rfind('<') {
        edits.push(SurgeryEdit {
            start: insert_pos,
            end: insert_pos,
            replacement: added_raw.to_string(),
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Border union: khi giữ nội dung JP cho ô "không đổi" nhưng áp STYLE từ VN (để phản ánh
// border/fill VN có thể đã THÊM — xem `clone_vn_sheet_for_jp`), ghi đè `s=` toàn bộ như trước
// (`restyle_raw_cell_xml`) làm MẤT border JP nếu style VN tại vị trí đó không có cạnh border JP
// từng có (rất phổ biến khi VN chỉnh sửa không giữ đúng format gốc). Cơ chế dưới đây HỢP NHẤT
// border: cạnh nào VN có thì dùng VN (tôn trọng border VN mới thêm), cạnh nào VN KHÔNG có thì lấy
// từ JP (không làm mất border gốc) — chỉ tạo `<border>`/`<xf>` mới trong styles.xml khi union thực
// sự khác style VN gốc (đa số trường hợp không cần, tái dùng ngay style VN đã remap).
// ─────────────────────────────────────────────────────────────────────────────

/// 4 cạnh + đường chéo của 1 `<border>` — mỗi cạnh lưu RAW XML đầy đủ (kể cả `<color>` con) để
/// tái tạo `<border>` mới y hệt, `None` = cạnh không có style (không vẽ đường viền).
#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct BorderDef {
    left: Option<String>,
    right: Option<String>,
    top: Option<String>,
    bottom: Option<String>,
    diagonal: Option<String>,
    /// Attribute `diagonalUp`/`diagonalDown` trên chính `<border>` (nếu có).
    diagonal_attrs: String,
}

/// Parse 1 phần tử `<border>` → `BorderDef`. Cạnh coi là "có" chỉ khi phần tử con tồn tại VÀ có
/// attribute `style` (thiếu `style` = không vẽ viền dù có `<color>`, theo đúng ngữ nghĩa OOXML).
fn parse_border_def(border_node: roxmltree::Node, src_xml: &str) -> BorderDef {
    let side = |tag: &str| -> Option<String> {
        border_node
            .children()
            .find(|c| c.tag_name().name() == tag)
            .filter(|n| n.attribute("style").is_some())
            .map(|n| src_xml[n.range()].to_string())
    };
    let mut diagonal_attrs = String::new();
    if let Some(v) = border_node.attribute("diagonalUp") {
        diagonal_attrs.push_str(&format!(r#" diagonalUp="{v}""#));
    }
    if let Some(v) = border_node.attribute("diagonalDown") {
        diagonal_attrs.push_str(&format!(r#" diagonalDown="{v}""#));
    }
    BorderDef {
        left: side("left"),
        right: side("right"),
        top: side("top"),
        bottom: side("bottom"),
        diagonal: side("diagonal"),
        diagonal_attrs,
    }
}

/// Parse toàn bộ `<borders>` → `Vec<BorderDef>` theo đúng thứ tự index (borderId tra theo vị trí).
fn parse_borders_table(borders_node: Option<roxmltree::Node>, src_xml: &str) -> Vec<BorderDef> {
    let Some(node) = borders_node else { return Vec::new() };
    node.children()
        .filter(|c| c.tag_name().name() == "border")
        .map(|n| parse_border_def(n, src_xml))
        .collect()
}

/// Hợp nhất 2 border: cạnh nào `primary` có thì giữ `primary`, cạnh nào `primary` KHÔNG có thì
/// lấy từ `fallback`. Dùng để "VN thắng nếu có, JP bù nếu VN thiếu".
fn union_border_def(primary: &BorderDef, fallback: &BorderDef) -> BorderDef {
    BorderDef {
        left: primary.left.clone().or_else(|| fallback.left.clone()),
        right: primary.right.clone().or_else(|| fallback.right.clone()),
        top: primary.top.clone().or_else(|| fallback.top.clone()),
        bottom: primary.bottom.clone().or_else(|| fallback.bottom.clone()),
        diagonal: primary.diagonal.clone().or_else(|| fallback.diagonal.clone()),
        diagonal_attrs: if primary.diagonal_attrs.is_empty() {
            fallback.diagonal_attrs.clone()
        } else {
            primary.diagonal_attrs.clone()
        },
    }
}

/// Render `BorderDef` thành raw XML `<border ...>...</border>` — cạnh `None` render thành phần tử
/// rỗng (`<left/>`) đúng theo cấu trúc OOXML (5 phần tử con luôn đủ, có/không có `style`).
fn render_border_def(def: &BorderDef) -> String {
    let side = |s: &Option<String>, tag: &str| s.clone().unwrap_or_else(|| format!("<{tag}/>"));
    format!(
        "<border{}>{}{}{}{}{}</border>",
        def.diagonal_attrs,
        side(&def.left, "left"),
        side(&def.right, "right"),
        side(&def.top, "top"),
        side(&def.bottom, "bottom"),
        side(&def.diagonal, "diagonal"),
    )
}

/// Trích `s="N"` từ 1 đoạn raw cell XML — `None` nếu không có (style mặc định 0).
fn extract_raw_cell_style(cell_xml: &str) -> Option<usize> {
    let s_start = cell_xml.find(" s=\"")?;
    let val_start = s_start + 4;
    let val_len = cell_xml[val_start..].find('"')?;
    cell_xml[val_start..val_start + val_len].parse().ok()
}

/// Thay phần SỐ DÒNG trong attribute `r="X{row}"` của 1 đoạn raw cell XML bằng `new_row1` — giữ
/// nguyên phần chữ cột. BẮT BUỘC phải gọi khi ghép nguyên trạng 1 ô JP (giữ `r=` gốc theo dòng
/// JP) vào 1 dòng VN có SỐ DÒNG KHÁC (do `vn_to_jp_row` bù lệch dòng — xem `clone_vn_sheet_for_jp`)
/// — nếu không sửa, `<c r="D19">` nằm trong `<row r="20">` là cấu trúc SAI theo OOXML: Excel coi
/// file hỏng, mở lên báo "cần sửa" (repair) rồi TỰ XÓA cả dòng/ô vi phạm — đây chính là nguyên
/// nhân request trước gây mất nội dung khi bật ánh xạ bù lệch dòng.
fn retarget_raw_cell_row(cell_xml: &str, new_row1: usize) -> String {
    let Some(r_start) = cell_xml.find(" r=\"") else {
        return cell_xml.to_string();
    };
    let val_start = r_start + 4;
    let Some(val_len) = cell_xml[val_start..].find('"') else {
        return cell_xml.to_string();
    };
    let old_ref = &cell_xml[val_start..val_start + val_len];
    let col_letters: String = old_ref.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    let new_ref = format!("{col_letters}{new_row1}");
    let mut r = String::with_capacity(cell_xml.len());
    r.push_str(&cell_xml[..val_start]);
    r.push_str(&new_ref);
    r.push_str(&cell_xml[val_start + val_len..]);
    r
}

/// Thay `borderId="N"` trong 1 raw `<xf .../>` và đảm bảo `applyBorder="1"` (để Excel áp border
/// mới thay vì bỏ qua) — thêm nếu thiếu, chèn nếu chưa có attribute nào trong 2 attribute này
/// (hiếm — mọi `<xf>` trong `cellXfs` theo spec luôn có đủ numFmtId/fontId/fillId/borderId).
fn apply_union_border_to_xf_raw(xf_raw: &str, new_border_id: usize) -> String {
    fn replace_or_insert_attr(xml: &str, attr: &str, value: &str) -> String {
        let needle = format!("{attr}=\"");
        if let Some(start) = xml.find(&needle) {
            let val_start = start + needle.len();
            if let Some(len) = xml[val_start..].find('"') {
                let mut r = String::with_capacity(xml.len());
                r.push_str(&xml[..val_start]);
                r.push_str(value);
                r.push_str(&xml[val_start + len..]);
                return r;
            }
        }
        if let Some(pos) = xml.find("<xf") {
            let insert = pos + 3;
            let mut r = String::with_capacity(xml.len() + attr.len() + value.len() + 4);
            r.push_str(&xml[..insert]);
            r.push_str(&format!(r#" {attr}="{value}""#));
            r.push_str(&xml[insert..]);
            return r;
        }
        xml.to_string()
    }
    let with_border = replace_or_insert_attr(xf_raw, "borderId", &new_border_id.to_string());
    replace_or_insert_attr(&with_border, "applyBorder", "1")
}

/// Trạng thái tích lũy các `<border>`/`<xf>` mới cần thêm vào styles.xml khi ô "không đổi" cần
/// giữ border JP mà style VN (đã remap) không có — xem module doc phía trên. Tạo 1 lần mỗi lượt
/// `apply_changes`, dùng xuyên suốt cả vòng loop clone nhiều sheet, rồi `finish()` một lần cuối để
/// ghép các bản ghi mới vào styles.xml đã merge trước đó.
pub(crate) struct BorderUnionExtender {
    /// Border của MỌI xf trong styles.xml đã merge (JP gốc 0..cell_xfs_offset, VN đã remap từ
    /// cell_xfs_offset..) — tra theo đúng chỉ số cuối cùng dùng trong sheet XML.
    xf_border_lookup: HashMap<usize, BorderDef>,
    /// Raw `<xf .../>` (đã remap borderId/fontId/fillId/numFmtId) theo chỉ số VN gốc (0-based,
    /// TRƯỚC khi cộng `cell_xfs_offset`) — dùng làm khung khi cần tạo bản xf union mới.
    vn_xf_raw: Vec<String>,
    cell_xfs_offset: usize,
    next_border_id: usize,
    next_xf_id: usize,
    /// (jp_xf, vn_xf_remapped) → xf cuối cùng nên dùng (chính vn_xf_remapped nếu không cần union,
    /// hoặc xf union mới tạo) — tránh tạo trùng nhiều bản ghi cho cùng 1 cặp xf.
    cache: HashMap<(usize, usize), usize>,
    new_borders: Vec<String>,
    new_xfs: Vec<String>,
}

impl BorderUnionExtender {
    pub(crate) fn new(style_result: &StyleMergeResult) -> Self {
        BorderUnionExtender {
            xf_border_lookup: style_result.xf_border_lookup.clone(),
            vn_xf_raw: style_result.vn_xf_raw.clone(),
            cell_xfs_offset: style_result.cell_xfs_offset,
            next_border_id: style_result.borders_count,
            next_xf_id: style_result.cell_xfs_count,
            cache: HashMap::new(),
            new_borders: Vec::new(),
            new_xfs: Vec::new(),
        }
    }

    /// Trả về xf CUỐI CÙNG nên dùng khi giữ nội dung JP tại 1 ô: `jp_xf` là style gốc của ô JP
    /// (trích từ raw cell JP), `vn_xf_remapped` là style VN đã remap muốn áp (bảo toàn border/fill
    /// VN thêm). Nếu border VN đã bao trùm border JP (không thiếu cạnh nào), trả nguyên
    /// `vn_xf_remapped`; nếu không, tạo (hoặc tái dùng cache) 1 xf union mới.
    pub(crate) fn resolve_style_for_kept_cell(&mut self, jp_xf: usize, vn_xf_remapped: usize) -> usize {
        if let Some(&resolved) = self.cache.get(&(jp_xf, vn_xf_remapped)) {
            return resolved;
        }
        let empty = BorderDef::default();
        let jp_border = self.xf_border_lookup.get(&jp_xf).unwrap_or(&empty).clone();
        let vn_border = self.xf_border_lookup.get(&vn_xf_remapped).unwrap_or(&empty).clone();
        let union = union_border_def(&vn_border, &jp_border);

        let resolved = if union == vn_border {
            vn_xf_remapped
        } else {
            let new_border_id = self.next_border_id;
            self.new_borders.push(render_border_def(&union));
            self.next_border_id += 1;

            let vn_orig_idx = vn_xf_remapped.saturating_sub(self.cell_xfs_offset);
            let base_xf_raw = self
                .vn_xf_raw
                .get(vn_orig_idx)
                .cloned()
                .unwrap_or_else(|| r#"<xf numFmtId="0" fontId="0" fillId="0" borderId="0"/>"#.to_string());
            let new_xf_raw = apply_union_border_to_xf_raw(&base_xf_raw, new_border_id);
            self.new_xfs.push(new_xf_raw);

            let new_xf_idx = self.next_xf_id;
            self.next_xf_id += 1;
            self.xf_border_lookup.insert(new_xf_idx, union);
            new_xf_idx
        };

        self.cache.insert((jp_xf, vn_xf_remapped), resolved);
        resolved
    }

    /// Ghép các `<border>`/`<xf>` mới (nếu có) vào styles.xml đã merge trước đó — gọi 1 lần sau
    /// khi đã clone xong TOÀN BỘ sheet (không còn lời gọi `resolve_style_for_kept_cell` nào nữa).
    pub(crate) fn finish(self, merged_styles_xml: &str) -> String {
        if self.new_borders.is_empty() {
            return merged_styles_xml.to_string();
        }
        let Ok(doc) = roxmltree::Document::parse(merged_styles_xml) else {
            return merged_styles_xml.to_string();
        };
        let root = doc.root_element();
        let mut edits: Vec<SurgeryEdit> = Vec::new();
        append_style_section(
            &mut edits,
            merged_styles_xml,
            find_child(root, "borders"),
            &self.new_borders.join(""),
            self.new_borders.len(),
        );
        append_style_section(
            &mut edits,
            merged_styles_xml,
            find_child(root, "cellXfs"),
            &self.new_xfs.join(""),
            self.new_xfs.len(),
        );
        apply_surgery(merged_styles_xml, edits)
    }
}

pub(crate) fn merge_vn_styles_into_jp(jp_styles_xml: &str, vn_styles_xml: &str) -> StyleMergeResult {
    let jp_doc = match roxmltree::Document::parse(jp_styles_xml) {
        Ok(d) => d,
        Err(_) => {
            return StyleMergeResult {
                new_styles_xml: jp_styles_xml.to_string(),
                xf_remap: Vec::new(),
                xf_border_lookup: HashMap::new(),
                vn_xf_raw: Vec::new(),
                cell_xfs_offset: 0,
                borders_count: 0,
                cell_xfs_count: 0,
            }
        }
    };
    let vn_doc = match roxmltree::Document::parse(vn_styles_xml) {
        Ok(d) => d,
        Err(_) => {
            return StyleMergeResult {
                new_styles_xml: jp_styles_xml.to_string(),
                xf_remap: Vec::new(),
                xf_border_lookup: HashMap::new(),
                vn_xf_raw: Vec::new(),
                cell_xfs_offset: 0,
                borders_count: 0,
                cell_xfs_count: 0,
            }
        }
    };

    let jp_root = jp_doc.root_element();
    let vn_root = vn_doc.root_element();

    let jp_fonts = find_child(jp_root, "fonts");
    let jp_fills = find_child(jp_root, "fills");
    let jp_borders = find_child(jp_root, "borders");
    let jp_cell_style_xfs = find_child(jp_root, "cellStyleXfs");
    let jp_cell_xfs = find_child(jp_root, "cellXfs");
    let jp_num_fmts = find_child(jp_root, "numFmts");

    let fonts_offset = count_child_elements(jp_fonts, "font");
    let fills_offset = count_child_elements(jp_fills, "fill");
    let borders_offset = count_child_elements(jp_borders, "border");
    let cell_style_xfs_offset = count_child_elements(jp_cell_style_xfs, "xf");
    let cell_xfs_offset = count_child_elements(jp_cell_xfs, "xf");

    let mut jp_max_numfmt = 163usize;
    if let Some(nf) = &jp_num_fmts {
        for n in nf.children().filter(|c| c.tag_name().name() == "numFmt") {
            if let Some(id) = n.attribute("numFmtId").and_then(|s| s.parse::<usize>().ok()) {
                jp_max_numfmt = jp_max_numfmt.max(id);
            }
        }
    }

    // --- numFmt tùy biến (id >= 164) của VN: gán id mới, không đụng id built-in (0-163) ---
    let vn_num_fmts = find_child(vn_root, "numFmts");
    let mut numfmt_remap: HashMap<usize, usize> = HashMap::new();
    let mut next_numfmt_id = jp_max_numfmt + 1;
    let mut new_numfmt_entries: Vec<String> = Vec::new();
    if let Some(nf) = &vn_num_fmts {
        for n in nf.children().filter(|c| c.tag_name().name() == "numFmt") {
            let Some(old_id) = n.attribute("numFmtId").and_then(|s| s.parse::<usize>().ok()) else {
                continue;
            };
            if old_id < 164 {
                continue;
            }
            let new_id = *numfmt_remap.entry(old_id).or_insert_with(|| {
                let id = next_numfmt_id;
                next_numfmt_id += 1;
                id
            });
            let format_code = n.attribute("formatCode").unwrap_or("");
            new_numfmt_entries.push(format!(
                r#"<numFmt numFmtId="{new_id}" formatCode="{}"/>"#,
                xml_escape_attr(format_code)
            ));
        }
    }

    // --- Copy verbatim: fonts/fills/borders (không tham chiếu bảng khác, tự chứa hoàn toàn) ---
    let new_fonts_raw = copy_children_raw(find_child(vn_root, "fonts"), vn_styles_xml);
    let new_fills_raw = copy_children_raw(find_child(vn_root, "fills"), vn_styles_xml);
    let new_borders_raw = copy_children_raw(find_child(vn_root, "borders"), vn_styles_xml);
    let vn_fonts_count = count_child_elements(find_child(vn_root, "fonts"), "font");
    let vn_fills_count = count_child_elements(find_child(vn_root, "fills"), "fill");
    let vn_borders_count = count_child_elements(find_child(vn_root, "borders"), "border");

    // --- Border theo từng xf (JP giữ chỉ số gốc, VN đánh theo chỉ số TRƯỚC remap) — dùng bởi
    // `BorderUnionExtender` để hợp nhất border JP+VN cho ô "không đổi" giữ nội dung JP nhưng đổi
    // style theo VN (không được làm mất cạnh border JP vốn có — xem module doc phía trên).
    let jp_borders_defs = parse_borders_table(jp_borders, jp_styles_xml);
    let vn_borders_defs = parse_borders_table(find_child(vn_root, "borders"), vn_styles_xml);
    let mut xf_border_lookup: HashMap<usize, BorderDef> = HashMap::new();
    if let Some(jp_cxfs) = jp_cell_xfs {
        for (i, n) in jp_cxfs.children().filter(|c| c.tag_name().name() == "xf").enumerate() {
            let border_id = n.attribute("borderId").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let def = jp_borders_defs.get(border_id).cloned().unwrap_or_default();
            xf_border_lookup.insert(i, def);
        }
    }

    let mut new_cell_style_xfs_raw = String::new();
    let mut vn_cell_style_xfs_count = 0usize;
    if let Some(vn_csx) = find_child(vn_root, "cellStyleXfs") {
        for n in vn_csx.children().filter(|c| c.tag_name().name() == "xf") {
            new_cell_style_xfs_raw.push_str(&remap_xf_element(
                n,
                vn_styles_xml,
                fonts_offset,
                fills_offset,
                borders_offset,
                cell_style_xfs_offset,
                false,
                &numfmt_remap,
            ));
            vn_cell_style_xfs_count += 1;
        }
    }

    let mut new_cell_xfs_raw = String::new();
    let mut xf_remap: Vec<usize> = Vec::new();
    let mut vn_xf_raw: Vec<String> = Vec::new();
    if let Some(vn_cxfs) = find_child(vn_root, "cellXfs") {
        for (i, n) in vn_cxfs.children().filter(|c| c.tag_name().name() == "xf").enumerate() {
            let remapped = remap_xf_element(
                n,
                vn_styles_xml,
                fonts_offset,
                fills_offset,
                borders_offset,
                cell_style_xfs_offset,
                true,
                &numfmt_remap,
            );
            new_cell_xfs_raw.push_str(&remapped);
            vn_xf_raw.push(remapped);
            xf_remap.push(cell_xfs_offset + i);

            let border_id = n.attribute("borderId").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let def = vn_borders_defs.get(border_id).cloned().unwrap_or_default();
            xf_border_lookup.insert(cell_xfs_offset + i, def);
        }
    }

    // --- Ghép các đoạn mới vào đúng vị trí trong styles.xml JP (chỉ APPEND cuối mỗi bảng) ---
    let mut edits: Vec<SurgeryEdit> = Vec::new();

    // numFmts: JP có thể CHƯA có section này — nếu vậy và VN có numFmt cần thêm, chèn mới hẳn.
    if !new_numfmt_entries.is_empty() {
        let joined = new_numfmt_entries.join("");
        if jp_num_fmts.is_some() {
            append_style_section(&mut edits, jp_styles_xml, jp_num_fmts, &joined, new_numfmt_entries.len());
        } else if let Some(fonts_node) = jp_fonts {
            let insert_pos = fonts_node.range().start;
            edits.push(SurgeryEdit {
                start: insert_pos,
                end: insert_pos,
                replacement: format!(
                    r#"<numFmts count="{}">{joined}</numFmts>"#,
                    new_numfmt_entries.len()
                ),
            });
        }
    }

    append_style_section(&mut edits, jp_styles_xml, jp_fonts, &new_fonts_raw, vn_fonts_count);
    append_style_section(&mut edits, jp_styles_xml, jp_fills, &new_fills_raw, vn_fills_count);
    append_style_section(&mut edits, jp_styles_xml, jp_borders, &new_borders_raw, vn_borders_count);
    append_style_section(
        &mut edits,
        jp_styles_xml,
        jp_cell_style_xfs,
        &new_cell_style_xfs_raw,
        vn_cell_style_xfs_count,
    );
    append_style_section(&mut edits, jp_styles_xml, jp_cell_xfs, &new_cell_xfs_raw, xf_remap.len());

    let cell_xfs_count = cell_xfs_offset + xf_remap.len();
    let borders_count = borders_offset + vn_borders_count;

    StyleMergeResult {
        new_styles_xml: apply_surgery(jp_styles_xml, edits),
        xf_remap,
        xf_border_lookup,
        vn_xf_raw,
        cell_xfs_offset,
        borders_count,
        cell_xfs_count,
    }
}

/// Các phần tử con cấp-1 của `<worksheet>` cần LOẠI BỎ khi clone sang JP: nằm ngoài phạm vi
/// best-effort đã thống nhất (conditional formatting, data validation) hoặc tham chiếu 1 part
/// riêng không được copy theo (legacyDrawing — VML cho comment cũ, hiếm gặp). `drawing` KHÔNG
/// nằm trong danh sách này — được xử lý riêng ở `build_cloned_sheet_xml` (giữ lại + đổi r:id nếu
/// clone được, xem `clone_vn_drawing`).
const CLONE_STRIP_TOP_LEVEL_TAGS: &[&str] = &[
    "legacyDrawing",
    "legacyDrawingHF",
    "oleObjects",
    "controls",
    "conditionalFormatting",
    "dataValidations",
    "extLst",
    "hyperlinks",
    "pageSetup",
];

/// Trả về `<Default Extension="..".../>` phù hợp cho 1 phần mở rộng ảnh — dùng khi đăng ký ảnh
/// nhúng mới vào `[Content_Types].xml` (nếu Extension đó chưa được khai báo).
fn content_type_default_for_ext(ext: &str) -> String {
    let ext_lower = ext.to_ascii_lowercase();
    let content_type = match ext_lower.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "emf" => "image/x-emf",
        "wmf" => "image/x-wmf",
        _ => "application/octet-stream",
    };
    format!(r#"<Default Extension="{ext_lower}" ContentType="{content_type}"/>"#)
}

/// Kết quả clone drawing (textbox/shape nổi, kèm ảnh nhúng nếu có) của 1 sheet VN sang JP.
struct ClonedDrawing {
    /// Đường dẫn part drawing mới trong JP (vd "xl/drawings/drawing5.xml").
    part_path: String,
    /// rId cục bộ trong file rels riêng của SHEET mới — luôn "rId1" vì rels đó chỉ có đúng 1 quan hệ.
    sheet_rid: &'static str,
}

/// Clone drawing (+ ảnh nhúng nếu có) của 1 sheet VN sang JP: copy nguyên XML drawing (không cần
/// đổi gì bên trong vì file rels riêng mới sẽ giữ đúng `rId` gốc), copy từng ảnh nhúng sang
/// `xl/media/` (đổi tên nếu trùng với ảnh JP hiện có) và viết lại Target trong rels cho khớp.
/// `jp_existing_names`: danh sách part đã có sẵn trong JP (từ trước khi clone) — dùng để tránh
/// trùng tên part mới; `replaced`: cũng được kiểm tra tránh trùng vì có thể đã thêm ở sheet trước
/// đó trong CÙNG 1 lượt clone nhiều sheet. `content_types_additions`: các `<Default>`/`<Override>`
/// cần thêm vào `[Content_Types].xml` (bên gọi tự khử trùng trước khi chèn thật).
fn clone_vn_drawing(
    vn_archive: &mut zip::ZipArchive<File>,
    jp_existing_names: &HashSet<String>,
    vn_drawing_path: &str,
    replaced: &mut HashMap<String, Vec<u8>>,
    content_types_additions: &mut Vec<String>,
) -> Option<ClonedDrawing> {
    let drawing_xml = read_zip_entry(vn_archive, vn_drawing_path)?;

    let mut idx = 1usize;
    let mut part_path = format!("xl/drawings/drawing{idx}.xml");
    while jp_existing_names.contains(&part_path) || replaced.contains_key(&part_path) {
        idx += 1;
        part_path = format!("xl/drawings/drawing{idx}.xml");
    }

    // --- Ảnh nhúng (nếu drawing gốc có rels riêng, vd <a:blip r:embed="rIdX"/>) ---
    let vn_drawing_rels_path = rels_path_for(vn_drawing_path);
    if let Some(vn_rels_xml) = read_zip_entry(vn_archive, &vn_drawing_rels_path) {
        if let Ok(doc) = roxmltree::Document::parse(&vn_rels_xml) {
            let mut edits: Vec<SurgeryEdit> = Vec::new();
            let mut media_idx = 1usize;
            for node in doc.descendants() {
                if node.tag_name().name() != "Relationship" {
                    continue;
                }
                let is_image = node.attribute("Type").map_or(false, |t| t.ends_with("/image"));
                if !is_image {
                    continue; // Quan hệ khác (chart/ole...) — ngoài phạm vi best-effort, bỏ qua.
                }
                let Some(target) = node.attribute("Target") else {
                    continue;
                };
                let vn_image_path = resolve_relative_part_path(vn_drawing_path, target);
                let Some(image_bytes) = read_zip_entry_bytes(vn_archive, &vn_image_path) else {
                    continue;
                };
                let ext = vn_image_path
                    .rsplit('.')
                    .next()
                    .unwrap_or("png")
                    .to_ascii_lowercase();

                let mut new_media_path;
                loop {
                    new_media_path = format!("xl/media/image{media_idx}.{ext}");
                    media_idx += 1;
                    if !jp_existing_names.contains(&new_media_path) && !replaced.contains_key(&new_media_path) {
                        break;
                    }
                }
                replaced.insert(new_media_path.clone(), image_bytes);
                content_types_additions.push(content_type_default_for_ext(&ext));

                if let Some(target_attr) = node.attributes().find(|a| a.name() == "Target") {
                    let r = target_attr.range_value();
                    let media_name = new_media_path.rsplit('/').next().unwrap_or(&new_media_path);
                    edits.push(SurgeryEdit {
                        start: r.start,
                        end: r.end,
                        replacement: format!("../media/{media_name}"),
                    });
                }
            }
            if !edits.is_empty() {
                let new_rels_xml = apply_surgery(&vn_rels_xml, edits);
                let new_rels_path = rels_path_for(&part_path);
                replaced.insert(new_rels_path, new_rels_xml.into_bytes());
            }
        }
    }

    replaced.insert(part_path.clone(), drawing_xml.into_bytes());
    content_types_additions.push(format!(
        r#"<Override PartName="/{part_path}" ContentType="application/vnd.openxmlformats-officedocument.drawing+xml"/>"#
    ));

    Some(ClonedDrawing {
        part_path,
        sheet_rid: "rId1",
    })
}

/// Dựng lại XML sheet VN để clone sang JP: remap `s=`/`style=` theo `xf_remap`, chuyển shared
/// string (t="s") thành inline string (không phụ thuộc bảng sharedStrings gốc), giữ lại
/// `<drawing>` và đổi `r:id` cho khớp rels riêng của sheet mới nếu `drawing_rid` có giá trị (xem
/// `clone_vn_drawing`) — nếu không có (sheet VN không có drawing, hoặc clone thất bại) thì bỏ hẳn
/// thẻ này. Các phần tử khác ngoài phạm vi best-effort bị loại bỏ (`CLONE_STRIP_TOP_LEVEL_TAGS`).
fn build_cloned_sheet_xml(
    vn_sheet_xml: &str,
    plain_ssi: &HashMap<usize, String>,
    rich_ssi_raw: &HashMap<usize, String>,
    xf_remap: &[usize],
    drawing_rid: Option<&str>,
) -> String {
    let doc = match roxmltree::Document::parse(vn_sheet_xml) {
        Ok(d) => d,
        Err(_) => return vn_sheet_xml.to_string(),
    };

    let mut edits: Vec<SurgeryEdit> = Vec::new();

    for node in doc.descendants() {
        let tag = node.tag_name().name();

        if tag == "row" || tag == "col" {
            let attr_name = if tag == "row" { "s" } else { "style" };
            if let Some(attr) = node.attributes().find(|a| a.name() == attr_name) {
                if let Ok(old) = attr.value().parse::<usize>() {
                    if let Some(&new_idx) = xf_remap.get(old) {
                        let r = attr.range_value();
                        edits.push(SurgeryEdit {
                            start: r.start,
                            end: r.end,
                            replacement: new_idx.to_string(),
                        });
                    }
                }
            }
            continue;
        }

        if tag != "c" {
            continue;
        }

        let has_formula = node.children().any(|c| c.tag_name().name() == "f");
        let s_attr_val = node.attribute("s").and_then(|s| s.parse::<usize>().ok());
        let new_s = s_attr_val.and_then(|old| xf_remap.get(old).copied());

        if !has_formula && node.attribute("t") == Some("s") {
            let ssi = node
                .children()
                .find(|c| c.tag_name().name() == "v")
                .and_then(|v| v.text())
                .and_then(|t| t.parse::<usize>().ok());
            if let Some(ssi) = ssi {
                let cell_ref = node.attribute("r").unwrap_or("");
                let s_part = new_s.map(|v| format!(" s=\"{v}\"")).unwrap_or_default();
                let replacement = if let Some(raw_runs) = rich_ssi_raw.get(&ssi) {
                    format!(r#"<c r="{cell_ref}"{s_part} t="inlineStr"><is>{raw_runs}</is></c>"#)
                } else {
                    let text = plain_ssi.get(&ssi).cloned().unwrap_or_default();
                    format!(
                        r#"<c r="{cell_ref}"{s_part} t="inlineStr"><is><t xml:space="preserve">{}</t></is></c>"#,
                        xml_escape(&text)
                    )
                };
                edits.push(SurgeryEdit {
                    start: node.range().start,
                    end: node.range().end,
                    replacement,
                });
                continue;
            }
        }

        // Mọi cell khác (số, boolean, inlineStr sẵn có, công thức...): chỉ remap `s=`.
        if let (Some(_old), Some(new_idx)) = (s_attr_val, new_s) {
            if let Some(s_attr_node) = node.attributes().find(|a| a.name() == "s") {
                let r = s_attr_node.range_value();
                edits.push(SurgeryEdit {
                    start: r.start,
                    end: r.end,
                    replacement: new_idx.to_string(),
                });
            }
        }
    }

    // <drawing r:id="..."/>: giữ lại + đổi r:id nếu clone được, không thì bỏ hẳn (tránh treo rel).
    if let Some(drawing_node) = doc.root_element().children().find(|c| c.tag_name().name() == "drawing") {
        match drawing_rid {
            Some(new_rid) => {
                let r_ns = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
                if let Some(attr) = drawing_node
                    .attributes()
                    .find(|a| a.name() == "id" && a.namespace() == Some(r_ns))
                {
                    let r = attr.range_value();
                    edits.push(SurgeryEdit {
                        start: r.start,
                        end: r.end,
                        replacement: new_rid.to_string(),
                    });
                }
            }
            None => {
                edits.push(SurgeryEdit {
                    start: drawing_node.range().start,
                    end: drawing_node.range().end,
                    replacement: String::new(),
                });
            }
        }
    }

    for node in doc.root_element().children() {
        if CLONE_STRIP_TOP_LEVEL_TAGS.contains(&node.tag_name().name()) {
            edits.push(SurgeryEdit {
                start: node.range().start,
                end: node.range().end,
                replacement: String::new(),
            });
        }
    }

    apply_surgery(vn_sheet_xml, edits)
}

/// Tìm số N nhỏ nhất khả dụng cho part `xl/worksheets/sheetN.xml` mới — dựa trên Target của
/// MỌI relationship trong `xl/_rels/workbook.xml.rels` hiện có (tránh trùng part đã tồn tại).
fn next_free_worksheet_index(rels_xml: &str) -> usize {
    let mut max_idx = 0usize;
    if let Ok(doc) = roxmltree::Document::parse(rels_xml) {
        for node in doc.descendants() {
            if node.tag_name().name() != "Relationship" {
                continue;
            }
            let Some(target) = node.attribute("Target") else {
                continue;
            };
            if let Some(rest) = target.rsplit('/').next() {
                if let Some(num_str) = rest.strip_prefix("sheet").and_then(|s| s.strip_suffix(".xml")) {
                    if let Ok(n) = num_str.parse::<usize>() {
                        max_idx = max_idx.max(n);
                    }
                }
            }
        }
    }
    max_idx + 1
}

/// Tìm `rId` nhỏ nhất khả dụng trong `xl/_rels/workbook.xml.rels` hiện có.
fn next_free_rid_index(rels_xml: &str) -> usize {
    let mut max_idx = 0usize;
    if let Ok(doc) = roxmltree::Document::parse(rels_xml) {
        for node in doc.descendants() {
            if node.tag_name().name() != "Relationship" {
                continue;
            }
            if let Some(id) = node
                .attribute("Id")
                .and_then(|s| s.strip_prefix("rId"))
                .and_then(|s| s.parse::<usize>().ok())
            {
                max_idx = max_idx.max(id);
            }
        }
    }
    max_idx + 1
}

/// Tìm `sheetId` nhỏ nhất khả dụng trong `xl/workbook.xml` hiện có.
fn next_free_sheet_id(workbook_xml: &str) -> usize {
    let mut max_id = 0usize;
    if let Ok(doc) = roxmltree::Document::parse(workbook_xml) {
        for node in doc.descendants() {
            if node.tag_name().name() != "sheet" {
                continue;
            }
            if let Some(id) = node.attribute("sheetId").and_then(|s| s.parse::<usize>().ok()) {
                max_id = max_id.max(id);
            }
        }
    }
    max_id + 1
}

/// Kết quả tạo sheet mới cho các sheet chỉ có ở VN.
struct ClonedSheetsOutcome {
    /// (tên sheet mới, đường dẫn part XML mới) — bên gọi thêm vào `sheet_path_map` TRƯỚC khi
    /// chạy vòng lặp chính của `apply_changes`, nhưng KHÔNG để vòng lặp đó dọn dẹp/ghi ô đỏ vào
    /// sheet này nữa (nội dung đã đúng 100% từ VN ngay khi clone) — xem ghi chú đầu mục.
    new_entries: Vec<(String, String)>,
}

/// Tạo sheet mới trong JP cho từng tên trong `sheets_to_clone`, bằng cách CLONE TRỰC TIẾP sheet
/// cùng tên từ file VN: đọc sheet XML + styles.xml + sharedStrings.xml + drawing (nếu có) của VN,
/// merge style vào JP (tích lũy qua từng sheet — xem `merge_vn_styles_into_jp`), remap `s=`/
/// chuyển shared string → inline (xem `build_cloned_sheet_xml`), clone luôn textbox/shape + ảnh
/// nhúng (xem `clone_vn_drawing`), rồi đăng ký part mới vào workbook.xml/rels/
/// [Content_Types].xml của JP. Mọi thay đổi được ghi vào `replaced` (chưa ghi ra đĩa).
fn clone_missing_sheets_from_vn(
    archive: &mut zip::ZipArchive<File>,
    vn_path: &str,
    sheets_to_clone: &[String],
    replaced: &mut HashMap<String, Vec<u8>>,
) -> AppResult<ClonedSheetsOutcome> {
    if sheets_to_clone.is_empty() {
        return Ok(ClonedSheetsOutcome {
            new_entries: Vec::new(),
        });
    }

    let vn_file =
        File::open(vn_path).map_err(|e| AppError::new(format!("Không mở được file VN: {e}")))?;
    let mut vn_archive = zip::ZipArchive::new(vn_file)
        .map_err(|e| AppError::new(format!("File VN không phải ZIP hợp lệ: {e}")))?;

    let vn_styles_xml = read_zip_entry(&mut vn_archive, "xl/styles.xml").unwrap_or_default();
    let vn_sst_xml = read_zip_entry(&mut vn_archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (vn_plain_ssi, vn_rich_ssi_raw) = if vn_sst_xml.is_empty() {
        (HashMap::new(), HashMap::new())
    } else {
        extract_all_shared_strings(&vn_sst_xml)
    };
    let vn_workbook_xml = read_zip_entry(&mut vn_archive, "xl/workbook.xml")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/workbook.xml trong file VN."))?;
    let vn_rels_xml = read_zip_entry(&mut vn_archive, "xl/_rels/workbook.xml.rels")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/_rels/workbook.xml.rels trong file VN."))?;
    let vn_sheet_path_map: HashMap<String, String> =
        resolve_sheet_xml_paths(&vn_workbook_xml, &vn_rels_xml)
            .into_iter()
            .collect();

    let mut jp_styles_xml = read_zip_entry(archive, "xl/styles.xml")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/styles.xml trong file JP."))?;
    let mut jp_workbook_xml = read_zip_entry(archive, "xl/workbook.xml")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/workbook.xml trong file JP."))?;
    let mut jp_rels_xml = read_zip_entry(archive, "xl/_rels/workbook.xml.rels")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/_rels/workbook.xml.rels trong file JP."))?;
    let mut jp_content_types_xml = read_zip_entry(archive, "[Content_Types].xml")
        .ok_or_else(|| AppError::new("Không tìm thấy [Content_Types].xml trong file JP."))?;

    let jp_existing_names: HashSet<String> = archive.file_names().map(|s| s.to_string()).collect();

    let mut next_sheet_part_idx = next_free_worksheet_index(&jp_rels_xml);
    let mut next_rid_idx = next_free_rid_index(&jp_rels_xml);
    let mut next_sheet_id = next_free_sheet_id(&jp_workbook_xml);
    let mut content_types_additions: Vec<String> = Vec::new();

    let mut new_entries: Vec<(String, String)> = Vec::new();

    for sheet_name in sheets_to_clone {
        let Some(vn_xml_path) = vn_sheet_path_map.get(sheet_name) else {
            continue;
        };
        let Some(vn_sheet_xml) = read_zip_entry(&mut vn_archive, vn_xml_path) else {
            continue;
        };

        let merge = merge_vn_styles_into_jp(&jp_styles_xml, &vn_styles_xml);
        jp_styles_xml = merge.new_styles_xml;

        let part_idx = next_sheet_part_idx;
        let rid = format!("rId{next_rid_idx}");
        let sheet_id = next_sheet_id;
        next_sheet_part_idx += 1;
        next_rid_idx += 1;
        next_sheet_id += 1;
        let part_path = format!("xl/worksheets/sheet{part_idx}.xml");

        // Textbox/shape nổi (+ ảnh nhúng nếu có) — clone luôn nếu sheet VN có drawing.
        let vn_drawing_path = resolve_drawing_path(&mut vn_archive, &vn_sheet_xml, vn_xml_path);
        let cloned_drawing = vn_drawing_path.as_ref().and_then(|dp| {
            clone_vn_drawing(
                &mut vn_archive,
                &jp_existing_names,
                dp,
                replaced,
                &mut content_types_additions,
            )
        });

        let new_xml = build_cloned_sheet_xml(
            &vn_sheet_xml,
            &vn_plain_ssi,
            &vn_rich_ssi_raw,
            &merge.xf_remap,
            cloned_drawing.as_ref().map(|d| d.sheet_rid),
        );
        replaced.insert(part_path.clone(), new_xml.into_bytes());

        if let Some(cloned) = &cloned_drawing {
            let sheet_rels_path = rels_path_for(&part_path);
            let sheet_rels_xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing" Target="../drawings/{}"/></Relationships>"#,
                cloned.sheet_rid,
                cloned.part_path.rsplit('/').next().unwrap_or(&cloned.part_path)
            );
            replaced.insert(sheet_rels_path, sheet_rels_xml.into_bytes());
        }

        if let Some(pos) = jp_workbook_xml.rfind("</sheets>") {
            // Khai báo `xmlns:r` ngay trên phần tử này — file thật (Excel/WPS) luôn khai báo ở
            // cấp <sheet>, không phải ở <workbook> gốc; thiếu khai báo làm prefix "r:" trong
            // r:id không xác định được namespace, khiến parser XML nghiêm ngặt (vd roxmltree)
            // coi tài liệu không hợp lệ dù calamine vẫn đọc được (quá dễ dãi).
            let entry = format!(
                r#"<sheet xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" name="{}" sheetId="{sheet_id}" r:id="{rid}"/>"#,
                xml_escape_attr(sheet_name)
            );
            jp_workbook_xml.insert_str(pos, &entry);
        }

        if let Some(pos) = jp_rels_xml.rfind("</Relationships>") {
            let entry = format!(
                r#"<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{part_idx}.xml"/>"#
            );
            jp_rels_xml.insert_str(pos, &entry);
        }

        content_types_additions.push(format!(
            r#"<Override PartName="/{part_path}" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#
        ));

        new_entries.push((sheet_name.clone(), part_path));
    }

    // Chèn [Content_Types].xml additions — khử trùng: bỏ qua Default đã có Extension trùng, hoặc
    // Override đã có PartName trùng (tránh khai báo lặp, không hợp lệ theo schema).
    for entry in &content_types_additions {
        let already_present = if let Some(ext) = entry
            .split("Extension=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
        {
            jp_content_types_xml.contains(&format!("Extension=\"{ext}\""))
        } else if let Some(part) = entry
            .split("PartName=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
        {
            jp_content_types_xml.contains(&format!("PartName=\"{part}\""))
        } else {
            false
        };
        if already_present {
            continue;
        }
        if let Some(pos) = jp_content_types_xml.rfind("</Types>") {
            jp_content_types_xml.insert_str(pos, entry);
        }
    }

    if !new_entries.is_empty() {
        replaced.insert("xl/styles.xml".to_string(), jp_styles_xml.into_bytes());
        replaced.insert("xl/workbook.xml".to_string(), jp_workbook_xml.into_bytes());
        replaced.insert(
            "xl/_rels/workbook.xml.rels".to_string(),
            jp_rels_xml.into_bytes(),
        );
        replaced.insert(
            "[Content_Types].xml".to_string(),
            jp_content_types_xml.into_bytes(),
        );
    }

    Ok(ClonedSheetsOutcome { new_entries })
}

/// Sắp xếp lại thứ tự `<sheet>` trong `<sheets>` của workbook.xml JP cho khớp thứ tự sheet VN
/// (`vn_sheet_order`): sheet có tên trùng VN được xếp theo ĐÚNG thứ tự xuất hiện trong VN; sheet
/// chỉ có ở JP (không có trong VN — vd sheet mẫu "(DEL)" dùng làm khung clone) giữ nguyên thứ tự
/// tương đối với nhau, dồn về cuối danh sách.
fn reorder_sheets_to_match_vn(workbook_xml: &str, vn_sheet_order: &[String]) -> String {
    let doc = match roxmltree::Document::parse(workbook_xml) {
        Ok(d) => d,
        Err(_) => return workbook_xml.to_string(),
    };
    let Some(sheets_node) = doc.descendants().find(|n| n.tag_name().name() == "sheets") else {
        return workbook_xml.to_string();
    };

    let entries: Vec<(String, String)> = sheets_node
        .children()
        .filter(|c| c.tag_name().name() == "sheet")
        .filter_map(|n| {
            n.attribute("name")
                .map(|name| (name.to_string(), workbook_xml[n.range()].to_string()))
        })
        .collect();

    let vn_pos: HashMap<&str, usize> = vn_sheet_order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    // Bản đồ phụ: base name của các sheet DEL trong VN → vị trí, dùng để khớp sheet DEL trong JP
    // có khoảng trắng khác (vd JP có "XXX (DEL)" còn VN có "XXX(DEL)") — tránh bị xếp cuối.
    let vn_del_base_pos: HashMap<String, usize> = vn_sheet_order
        .iter()
        .enumerate()
        .filter(|(_, n)| is_del_sheet_name(n))
        .map(|(i, n)| (strip_del_suffix(n), i))
        .collect();

    let mut indexed: Vec<(usize, &(String, String))> = entries.iter().enumerate().collect();
    indexed.sort_by_key(|(orig_idx, (name, _))| {
        if let Some(&p) = vn_pos.get(name.as_str()) {
            return (0usize, p, *orig_idx);
        }
        if is_del_sheet_name(name) {
            let base = strip_del_suffix(name);
            if let Some(&p) = vn_del_base_pos.get(&base) {
                return (0usize, p, *orig_idx);
            }
        }
        (1usize, 0usize, *orig_idx)
    });

    // Không đổi thứ tự thực tế → tránh ghi lại nếu không cần thiết.
    if indexed.iter().map(|(i, _)| *i).eq(0..entries.len()) {
        return workbook_xml.to_string();
    }

    let new_inner: String = indexed.iter().map(|(_, (_, raw))| raw.clone()).collect();

    let sheets_range = sheets_node.range();
    let open_end = find_tag_open_end(workbook_xml, sheets_range.start);
    let Some(close_offset) = workbook_xml[open_end..sheets_range.end].find("</sheets>") else {
        return workbook_xml.to_string();
    };
    let close_start = open_end + close_offset;

    apply_surgery(
        workbook_xml,
        vec![SurgeryEdit {
            start: open_end,
            end: close_start,
            replacement: new_inner,
        }],
    )
}

/// Đường dẫn file JP kết quả của pipeline "Áp dụng" — `Temp/{tên JP gốc}_merged.{ext}`, thư mục
/// `Temp` nằm cùng cấp với nơi cài đặt application (xem `app_config::temp_dir`). File này được
/// GHI ĐÈ nhiều lần trong 1 lượt "Áp dụng" (`sync_structure` rồi `insert_rows` rồi `merge_content`
/// — xem `apply_changes`), không dùng hộp thoại chọn nơi lưu như trước.
pub(crate) fn merged_output_path(jp_path: &str) -> AppResult<PathBuf> {
    let path = Path::new(jp_path);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::new(format!("Tên file JP không hợp lệ: {jp_path}")))?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("xlsx");
    Ok(app_config::temp_dir().join(format!("{stem}_merged.{ext}")))
}

/// Kết quả bước 1-2 của pipeline "Áp dụng" (xem `sync_structure`). Public trong crate vì mỗi
/// `c23{2,3,4,5,6,8}_sync_service::apply_changes` tự lắp ráp pipeline, cần đọc trực tiếp các field.
pub(crate) struct StructureSyncResult {
    pub(crate) strike_removed_count: usize,
    pub(crate) red_blackened_count: usize,
    pub(crate) cleanup_skipped_count: usize,
    /// Tên các sheet vừa clone trực tiếp từ VN — `merge_content` chạy sau phải BỎ QUA các sheet
    /// này khi ghi nội dung ô đỏ (nội dung đã đúng 100% từ VN ngay khi clone).
    pub(crate) cloned_names: HashSet<String>,
    pub(crate) del_renamed_count: usize,
    pub(crate) sheets_modified: Vec<String>,
}

/// Bước 1-2 của pipeline "Áp dụng" (xem `apply_changes`): dọn dẹp MỌI sheet JP (như
/// `cleanup_jp`), clone sang JP mọi sheet chỉ tồn tại ở VN (trừ sheet VN tự đánh dấu đã xóa —
/// xem `compute_del_renames`), đổi tên sheet JP còn sót lại thành "(DEL)" nếu VN đã đánh dấu xóa,
/// rồi sắp xếp lại thứ tự sheet cho khớp VN. Ghi kết quả ra `output_path`. KHÔNG ghi nội dung ô
/// đỏ VN — xem `merge_content`, chạy sau trên chính file này.
pub(crate) fn sync_structure(vn_path: &str, jp_path: &str, output_path: &str) -> AppResult<StructureSyncResult> {
    let analysis = analyze(vn_path, jp_path)?;
    let doc_type = detect_doc_type(vn_path, jp_path);

    let vn_sheet_names: Vec<String> = analysis
        .sheet_compare
        .iter()
        .filter(|c| c.in_vn)
        .map(|c| c.name.clone())
        .collect();
    let jp_sheet_names: HashSet<String> = analysis
        .sheet_compare
        .iter()
        .filter(|c| c.in_jp)
        .map(|c| c.name.clone())
        .collect();
    // Bất kỳ sheet nào chỉ có ở VN (trừ sheet tự đánh dấu đã xóa, xử lý riêng bên dưới) → clone.
    let sheets_to_clone: Vec<String> = analysis
        .sheet_compare
        .iter()
        .filter(|c| c.in_vn && !c.in_jp && !is_del_sheet_name(&c.name))
        .map(|c| c.name.clone())
        .collect();
    let del_renames = compute_del_renames(&vn_sheet_names, &jp_sheet_names);
    let del_renamed_count = del_renames.len();

    let jp_file = File::open(jp_path)
        .map_err(|e| AppError::new(format!("Không mở được file JP: {e}")))?;
    let mut archive = zip::ZipArchive::new(jp_file)
        .map_err(|e| AppError::new(format!("File JP không phải ZIP hợp lệ: {e}")))?;

    // Bước 1: dọn dẹp — xóa hẳn strikethrough cũ + tô đen chữ đỏ cũ tồn đọng từ bản tablet cũ,
    // trên MỌI sheet (xem TrinhTuDichThietKeChiTiet_HuongDan.xlsx, mục 3.1).
    let ctx = build_cleanup_context(&mut archive);
    let mut replaced: HashMap<String, Vec<u8>> = HashMap::new();
    let (rich_ssi, plain_text, mut strike_removed_count, mut red_blackened_count) =
        match cleanup_shared_strings(&mut archive) {
            Some((new_xml, removed, blackened, rich_ssi, plain_text)) => {
                if removed > 0 || blackened > 0 {
                    replaced.insert("xl/sharedStrings.xml".to_string(), new_xml.into_bytes());
                }
                (rich_ssi, plain_text, removed, blackened)
            }
            None => (HashSet::new(), HashMap::new(), 0, 0),
        };

    let mut workbook_xml = read_zip_entry(&mut archive, "xl/workbook.xml")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/workbook.xml trong file JP."))?;
    let rels_xml = read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/_rels/workbook.xml.rels."))?;

    // Bước 2 (đổi tên): sheet JP còn sót lại tên gốc mà VN đã đánh dấu xóa → thêm hậu tố "(DEL)".
    // Đổi TRƯỚC khi resolve `sheet_path_map` để vòng lặp dọn dẹp bên dưới thấy đúng tên mới ngay,
    // tự động chạy nhánh `is_del_sheet_name` (bỏ màu chữ về đen) mà không cần thêm branch riêng.
    for (old_name, new_name) in &del_renames {
        workbook_xml = rename_sheet_in_workbook_xml(&workbook_xml, old_name, new_name);
    }
    if !del_renames.is_empty() {
        replaced.insert("xl/workbook.xml".to_string(), workbook_xml.clone().into_bytes());
    }

    let sheet_path_list = resolve_sheet_xml_paths(&workbook_xml, &rels_xml);
    let mut sheet_path_map: HashMap<String, String> = sheet_path_list.iter().cloned().collect();

    // Bước 2 (clone): sheet chỉ có ở VN → clone TRỰC TIẾP từ VN — xem ghi chú đầu mục "Sheet chỉ
    // có ở VN sang JP". Thêm ngay vào `sheet_path_map` TRƯỚC vòng lặp dọn dẹp, nhưng đánh dấu để
    // vòng lặp đó BỎ QUA sheet này (nội dung đã đúng 100% từ VN ngay khi clone).
    let cloned = clone_missing_sheets_from_vn(&mut archive, vn_path, &sheets_to_clone, &mut replaced)?;
    let cloned_names: HashSet<String> =
        cloned.new_entries.iter().map(|(name, _)| name.clone()).collect();
    for (name, path) in &cloned.new_entries {
        sheet_path_map.insert(name.clone(), path.clone());
    }

    let mut cleanup_skipped_count = 0usize;
    let mut sheets_modified: Vec<String> = Vec::new();

    for (sheet_name, xml_path) in &sheet_path_map {
        let Some(original_xml) = read_current(&mut archive, &replaced, xml_path) else {
            continue;
        };
        let bounds = content_bounds_for(doc_type, sheet_name);

        if cloned_names.contains(sheet_name) {
            if !sheets_modified.contains(sheet_name) {
                sheets_modified.push(sheet_name.clone());
            }
            continue;
        }

        // Sheet "(DEL)" (gốc sẵn có ở JP, hoặc vừa đổi tên ở trên): chỉ bỏ màu chữ về đen, KHÔNG
        // dọn strikethrough, KHÔNG đụng shape — xem ghi chú đầu mục "Sheet JP có hậu tố (DEL)".
        if is_del_sheet_name(sheet_name) {
            let (new_xml, blackened, skip) = blacken_all_cells_sheet_xml(
                &original_xml,
                &ctx.colored_xf,
                &plain_text,
                &rich_ssi,
                bounds,
            );
            cleanup_skipped_count += skip;
            if blackened > 0 {
                red_blackened_count += blackened;
                replaced.insert(xml_path.clone(), new_xml.into_bytes());
                sheets_modified.push(sheet_name.clone());
            }
            continue;
        }

        let drawing_path = resolve_drawing_path(&mut archive, &original_xml, xml_path);

        let (cleaned_xml, s_removed, r_blackened, c_skip) =
            cleanup_sheet_xml(&original_xml, &ctx, &rich_ssi, &plain_text, bounds);
        cleanup_skipped_count += c_skip;
        if s_removed > 0 || r_blackened > 0 {
            strike_removed_count += s_removed;
            red_blackened_count += r_blackened;
            replaced.insert(xml_path.clone(), cleaned_xml.into_bytes());
            sheets_modified.push(sheet_name.clone());
        }

        let Some(drawing_path) = drawing_path else {
            continue;
        };
        let Some(drawing_xml) = read_zip_entry(&mut archive, &drawing_path) else {
            continue;
        };
        let (cleaned_drawing, d_removed, d_blackened) = cleanup_drawing_xml(&drawing_xml);
        if d_removed > 0 || d_blackened > 0 {
            strike_removed_count += d_removed;
            red_blackened_count += d_blackened;
            replaced.insert(drawing_path, cleaned_drawing.into_bytes());
            if !sheets_modified.contains(sheet_name) {
                sheets_modified.push(sheet_name.clone());
            }
        }
    }

    // Sắp xếp lại thứ tự sheet JP cho khớp thứ tự sheet VN (xem `reorder_sheets_to_match_vn`) —
    // đảm bảo sheet mới tạo (thường bị chèn cuối danh sách) nằm đúng vị trí như bên VN.
    let vn_sheet_order: Vec<String> = {
        let vn_file_for_order = File::open(vn_path)
            .map_err(|e| AppError::new(format!("Không mở được file VN: {e}")))?;
        let mut vn_archive_for_order = zip::ZipArchive::new(vn_file_for_order)
            .map_err(|e| AppError::new(format!("File VN không phải ZIP hợp lệ: {e}")))?;
        let vn_wb = read_zip_entry(&mut vn_archive_for_order, "xl/workbook.xml").unwrap_or_default();
        let vn_rels =
            read_zip_entry(&mut vn_archive_for_order, "xl/_rels/workbook.xml.rels").unwrap_or_default();
        resolve_sheet_xml_paths(&vn_wb, &vn_rels)
            .into_iter()
            .map(|(name, _)| name)
            .collect()
    };
    if let Some(current_workbook_xml) = read_current(&mut archive, &replaced, "xl/workbook.xml") {
        let reordered = reorder_sheets_to_match_vn(&current_workbook_xml, &vn_sheet_order);
        if reordered != current_workbook_xml {
            replaced.insert("xl/workbook.xml".to_string(), reordered.into_bytes());
        }
    }

    write_output_zip(&mut archive, &replaced, output_path)?;

    Ok(StructureSyncResult {
        strike_removed_count,
        red_blackened_count,
        cleanup_skipped_count,
        cloned_names,
        del_renamed_count,
        sheets_modified,
    })
}

/// Convert a 0-based column index to an Excel column letter (0→"A", 26→"AA", …).
pub(crate) fn col_index_to_letter(col_0: usize) -> String {
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
pub(crate) fn xml_escape(s: &str) -> String {
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
///
/// `output_path` có thể TRÙNG với file đang mở trong `archive` (ghi đè in-place — xem pipeline
/// "Áp dụng" ở `c23X_sync_service::apply_changes`, chạy `sync_structure` → `insert_rows` →
/// `merge_content` liên tiếp trên CÙNG 1 file Temp). Vì vậy phải ĐỌC HẾT nội dung ZIP mới vào
/// buffer trong bộ nhớ trước, rồi mới `File::create(output_path)` để ghi đè — nếu tạo/truncate
/// file output ngay từ đầu (như trước đây) trong khi `archive` vẫn còn đang đọc từ CHÍNH file đó,
/// dữ liệu trên đĩa bị ghi đè giữa chừng, khiến các entry đọc sau (vd `[Content_Types].xml`) lấy
/// phải byte đã bị thay, gây lỗi "Invalid checksum".
const CALC_CHAIN_PART: &str = "xl/calcChain.xml";

/// Xóa `<Override PartName="/xl/calcChain.xml" .../>` khỏi `[Content_Types].xml`.
fn strip_calc_chain_content_type(xml: &str) -> String {
    let re = Regex::new(r#"<Override PartName="/xl/calcChain\.xml"[^>]*/>"#).unwrap();
    re.replace(xml, "").to_string()
}

/// Xóa `<Relationship .../>` trỏ tới `calcChain.xml` khỏi `xl/_rels/workbook.xml.rels`.
fn strip_calc_chain_relationship(xml: &str) -> String {
    let re = Regex::new(r#"<Relationship[^>]*Target="calcChain\.xml"[^>]*/>"#).unwrap();
    re.replace(xml, "").to_string()
}

pub(crate) fn write_output_zip(
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

    // `xl/calcChain.xml` (cache tăng tốc tính công thức) không được cập nhật khi pipeline chèn/xóa
    // dòng hoặc thay đổi vị trí công thức cột A — để nguyên sẽ tham chiếu sai vị trí ô, khiến Excel
    // báo "found a problem with some content" và tự ý repair (xóa sheet/dòng). Bỏ hẳn part này khỏi
    // output — hoàn toàn hợp lệ theo OOXML (tùy chọn), Excel tự tính lại khi mở file — kèm dọn luôn
    // tham chiếu của nó trong `[Content_Types].xml` và `xl/_rels/workbook.xml.rels` để không còn
    // part/relationship mồ côi.
    let has_calc_chain = entry_names.iter().any(|n| n == CALC_CHAIN_PART);
    entry_names.retain(|n| n != CALC_CHAIN_PART);

    let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::<u8>::new()));

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

        let mut bytes = if let Some(bytes) = replaced.get(name.as_str()) {
            bytes.clone()
        } else {
            let mut entry = archive
                .by_name(name)
                .map_err(|e| AppError::new(format!("Lỗi đọc entry {name}: {e}")))?;
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buf)
                .map_err(|e| AppError::new(format!("Lỗi đọc entry {name}: {e}")))?;
            buf
        };

        // Dọn tham chiếu tới `calcChain.xml` (đã bị bỏ khỏi output ở trên) trong CẢ 2 trường hợp —
        // dù nội dung lấy từ `archive` gốc hay đã được `replaced` (vd `sync_structure` thêm sheet
        // mới vào Content_Types/rels nhưng không biết gì về calcChain) — nếu không sẽ để lại
        // Override/Relationship trỏ tới part không tồn tại, Excel vẫn coi là lỗi.
        if has_calc_chain && (name == "[Content_Types].xml" || name == "xl/_rels/workbook.xml.rels")
        {
            let text = String::from_utf8_lossy(&bytes).to_string();
            let cleaned = if name == "[Content_Types].xml" {
                strip_calc_chain_content_type(&text)
            } else {
                strip_calc_chain_relationship(&text)
            };
            bytes = cleaned.into_bytes();
        }

        writer
            .write_all(&bytes)
            .map_err(|e| AppError::new(format!("Lỗi ghi nội dung {name}: {e}")))?;
    }

    // Ghi thêm các part HOÀN TOÀN MỚI chưa tồn tại trong ZIP gốc (vd sheet vừa clone từ VN —
    // xem `clone_missing_sheets`) — vòng lặp trên chỉ xử lý entry đã có sẵn trong `archive`.
    let existing: HashSet<&str> = entry_names.iter().map(|s| s.as_str()).collect();
    for (name, bytes) in replaced {
        if name == CALC_CHAIN_PART || existing.contains(name.as_str()) {
            continue;
        }
        let options =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        writer
            .start_file(name.as_str(), options)
            .map_err(|e| AppError::new(format!("Lỗi bắt đầu entry {name}: {e}")))?;
        writer
            .write_all(bytes)
            .map_err(|e| AppError::new(format!("Lỗi ghi nội dung {name}: {e}")))?;
    }

    let cursor = writer
        .finish()
        .map_err(|e| AppError::new(format!("Lỗi hoàn tất ZIP: {e}")))?;

    // Chỉ ghi ra đĩa SAU KHI đã đọc xong toàn bộ entry cần copy từ `archive` ở trên — an toàn cả
    // khi `output_path` trùng với file `archive` đang mở (xem ghi chú đầu hàm).
    std::fs::write(output_path, cursor.into_inner())
        .map_err(|e| AppError::new(format!("Không ghi được file output: {e}")))?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Chèn dòng vào file JP (sau khi TL xác nhận từng vị trí đề xuất canh dòng ở trên).
// Chèn dòng thật nghĩa là: đánh số lại (renumber) MỌI dòng/ô nằm sau vị trí chèn,
// dịch chuyển vùng merge cell/dimension liên quan, rồi thêm các dòng trống mới.
// ─────────────────────────────────────────────────────────────────────────────

/// Chèn các dòng trống vào một sheet XML tại nhiều vị trí đã xác nhận.
/// `inserts`: danh sách `(jp_insert_after_row 1-based, insert_count)` — dòng mới được
/// đánh số lại dựa trên vị trí gốc (trước khi chèn) của mọi dòng/ô, nên áp dụng đồng thời
/// nhiều vị trí chèn trong 1 lượt xử lý mà không cần lặp lại nhiều lần.
/// Một lượt chèn dòng vào sheet: chèn `count` dòng NGAY SAU dòng gốc `pos` (1-based). `build(base)`
/// dựng XML các dòng mới, với `base` = số dòng đích ngay TRƯỚC dòng đầu tiên được chèn (các dòng
/// mới đánh số `base+1..=base+count`). Chèn dòng trống hay clone nội dung tùy `build`.
struct RowInsert<'a> {
    pos: usize,
    count: usize,
    build: Box<dyn Fn(usize) -> String + 'a>,
}

fn insert_rows_into_sheet_xml(sheet_xml: &str, inserts: &[RowInsert]) -> String {
    let doc = match roxmltree::Document::parse(sheet_xml) {
        Ok(d) => d,
        Err(_) => return sheet_xml.to_string(),
    };

    // Tổng số dòng đã được chèn PHÍA TRƯỚC dòng gốc `original_row` — dùng để tính số dòng mới.
    let shift_for = |original_row: usize| -> usize {
        inserts
            .iter()
            .filter(|ins| ins.pos < original_row)
            .map(|ins| ins.count)
            .sum()
    };

    // Đánh số lại một cell ref (vd "C5") theo shift_for; None nếu không đổi (shift = 0).
    let shift_ref = |cell_ref: &str| -> Option<String> {
        let (row_0, col_0) = parse_cell_ref(cell_ref)?;
        let k = row_0 + 1;
        let shift = shift_for(k);
        if shift == 0 {
            return None;
        }
        Some(format!("{}{}", col_index_to_letter(col_0), k + shift))
    };

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    let mut row_keys: Vec<usize> = Vec::new();
    let mut sheet_data_range: Option<std::ops::Range<usize>> = None;

    for node in doc.descendants() {
        match node.tag_name().name() {
            "sheetData" => {
                sheet_data_range = Some(node.range());
            }
            "row" => {
                if let Some(r) = node.attribute("r").and_then(|s| s.parse::<usize>().ok()) {
                    row_keys.push(r);
                    let shift = shift_for(r);
                    if shift > 0 {
                        if let Some(attr) = node.attribute_node("r") {
                            edits.push(SurgeryEdit {
                                start: attr.range_value().start,
                                end: attr.range_value().end,
                                replacement: (r + shift).to_string(),
                            });
                        }
                    }
                }
            }
            "c" => {
                if let Some(attr) = node.attribute_node("r") {
                    if let Some(new_ref) = shift_ref(attr.value()) {
                        edits.push(SurgeryEdit {
                            start: attr.range_value().start,
                            end: attr.range_value().end,
                            replacement: new_ref,
                        });
                    }
                }
            }
            "mergeCell" | "dimension" => {
                if let Some(attr) = node.attribute_node("ref") {
                    if let Some((s_ref, e_ref)) = attr.value().split_once(':') {
                        let new_s = shift_ref(s_ref);
                        let new_e = shift_ref(e_ref);
                        if new_s.is_some() || new_e.is_some() {
                            let final_s = new_s.unwrap_or_else(|| s_ref.to_string());
                            let final_e = new_e.unwrap_or_else(|| e_ref.to_string());
                            edits.push(SurgeryEdit {
                                start: attr.range_value().start,
                                end: attr.range_value().end,
                                replacement: format!("{final_s}:{final_e}"),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(sd_range) = &sheet_data_range {
        for ins in inserts {
            let base = ins.pos + shift_for(ins.pos);
            let rows_xml = (ins.build)(base);
            let insert_byte_pos = find_row_insert_pos(sheet_xml, sd_range, &row_keys, ins.pos + 1);
            edits.push(SurgeryEdit {
                start: insert_byte_pos,
                end: insert_byte_pos,
                replacement: rows_xml,
            });
        }
    }

    apply_surgery(sheet_xml, edits)
}

/// Dựng XML các dòng trống `<row r=".."/>` cho 1 lượt chèn (strategy neo cũ, không có nội dung VN).
fn blank_rows_builder(count: usize) -> Box<dyn Fn(usize) -> String> {
    Box::new(move |base| {
        (1..=count)
            .map(|i| format!(r#"<row r="{}"/>"#, base + i))
            .collect()
    })
}

/// Clone các dòng VN trong khoảng [vn_start0, vn_end0] (0-based) sang XML dòng JP, đánh số lại về
/// `base+1..` và remap style (`s`) qua `xf_remap`, inline shared string (độc lập sst của JP).
fn clone_vn_rows_xml(
    vn_sheet_xml: &str,
    vn_start0: usize,
    vn_end0: usize,
    base: usize,
    xf_remap: &[usize],
    plain_ssi: &HashMap<usize, String>,
    rich_ssi_raw: &HashMap<usize, String>,
) -> String {
    let Ok(doc) = roxmltree::Document::parse(vn_sheet_xml) else {
        return String::new();
    };
    let mut rows: Vec<roxmltree::Node> = doc
        .descendants()
        .filter(|n| n.tag_name().name() == "row")
        .filter(|n| {
            n.attribute("r")
                .and_then(|s| s.parse::<usize>().ok())
                .map(|r| r >= vn_start0 + 1 && r <= vn_end0 + 1)
                .unwrap_or(false)
        })
        .collect();
    rows.sort_by_key(|n| {
        n.attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
    });

    let mut out = String::new();
    for (k, row) in rows.iter().enumerate() {
        let target = base + 1 + k;
        out.push_str(&clone_vn_row_xml(
            *row,
            vn_sheet_xml,
            target,
            xf_remap,
            plain_ssi,
            rich_ssi_raw,
        ));
    }
    out
}

pub(crate) fn clone_vn_row_xml(
    row: roxmltree::Node,
    vn_sheet_xml: &str,
    target_row1: usize,
    xf_remap: &[usize],
    plain_ssi: &HashMap<usize, String>,
    rich_ssi_raw: &HashMap<usize, String>,
) -> String {
    let mut attrs = String::new();
    for a in row.attributes() {
        match a.name() {
            "r" => {}
            "s" => {
                let remapped = a
                    .value()
                    .parse::<usize>()
                    .ok()
                    .and_then(|o| xf_remap.get(o).copied());
                match remapped {
                    Some(v) => attrs.push_str(&format!(" s=\"{v}\"")),
                    None => attrs.push_str(&format!(" s=\"{}\"", a.value())),
                }
            }
            name => attrs.push_str(&format!(" {}=\"{}\"", name, xml_escape_attr(a.value()))),
        }
    }
    let cells: String = row
        .children()
        .filter(|c| c.tag_name().name() == "c")
        .map(|c| clone_vn_cell_xml(c, vn_sheet_xml, target_row1, xf_remap, plain_ssi, rich_ssi_raw))
        .collect();
    format!(r#"<row r="{target_row1}"{attrs}>{cells}</row>"#)
}

pub(crate) fn clone_vn_cell_xml(
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
    let new_ref = format!("{}{}", col_index_to_letter(col0), target_row1);
    let new_s = cell
        .attribute("s")
        .and_then(|s| s.parse::<usize>().ok())
        .and_then(|o| xf_remap.get(o).copied());
    let s_part = new_s.map(|v| format!(" s=\"{v}\"")).unwrap_or_default();

    let has_formula = cell.children().any(|c| c.tag_name().name() == "f");

    // Shared string (t="s") → inline hoá để độc lập sst của JP (giống build_cloned_sheet_xml).
    if !has_formula && cell.attribute("t") == Some("s") {
        let ssi = cell
            .children()
            .find(|c| c.tag_name().name() == "v")
            .and_then(|v| v.text())
            .and_then(|t| t.parse::<usize>().ok());
        if let Some(ssi) = ssi {
            if let Some(raw) = rich_ssi_raw.get(&ssi) {
                return format!(r#"<c r="{new_ref}"{s_part} t="inlineStr"><is>{raw}</is></c>"#);
            }
            let text = plain_ssi.get(&ssi).cloned().unwrap_or_default();
            return format!(
                r#"<c r="{new_ref}"{s_part} t="inlineStr"><is><t xml:space="preserve">{}</t></is></c>"#,
                xml_escape(&text)
            );
        }
    }

    // Cell khác (số / boolean / inlineStr sẵn / công thức): giữ nguyên `t` + nội dung con, chỉ đổi r/s.
    let t_part = cell
        .attribute("t")
        .map(|t| format!(" t=\"{t}\""))
        .unwrap_or_default();
    let inner: String = cell
        .children()
        .map(|ch| vn_sheet_xml[ch.range()].to_string())
        .collect();
    if inner.trim().is_empty() {
        format!(r#"<c r="{new_ref}"{s_part}{t_part}/>"#)
    } else {
        format!(r#"<c r="{new_ref}"{s_part}{t_part}>{inner}</c>"#)
    }
}

/// Xóa override màu chữ cấp run (`<color .../>` bên trong `<rPr>`) khỏi 1 raw cell XML.
///
/// Ô "giữ nội dung JP" đi qua đây khi `cleanup_sheet_xml` (xem `make_black_inline_cell`) từng tô
/// đen chữ đỏ tồn đọng bằng cách chèn `<rPr><color rgb="FF000000"/></rPr>` NGAY TRONG run, đồng
/// thời giữ nguyên `s=` cũ (màu đỏ) — cố ý, vì lúc đó ô này chưa được restyle. Khi ô đó sau này
/// được restyle theo style VN (strike/đỏ, xem `restyle_raw_cell_xml`), override `<color>` cấp run
/// nói trên VẪN CÒN và đè lên màu của `s=` mới, khiến chữ hiển thị đen dù `s=` đã là đỏ/strike —
/// đúng bug đã gặp thực tế (dòng "giữ nội dung JP" hiển thị chữ đen thay vì đỏ/gạch). Ô kept-JP
/// phải để `s=` (style VN) quyết định toàn bộ màu/strike, không được override cấp run.
fn strip_run_color_overrides(cell_xml: &str) -> String {
    let re = Regex::new(r#"<color[^>]*/>"#).unwrap();
    re.replace_all(cell_xml, "").to_string()
}

/// Thay `s="X"` trong raw cell XML bằng giá trị mới; thêm nếu chưa có.
fn restyle_raw_cell_xml(cell_xml: &str, new_s: usize) -> String {
    let cell_xml = &strip_run_color_overrides(cell_xml);
    let s_val = new_s.to_string();
    // Tìm và thay s="..."
    if let Some(s_start) = cell_xml.find(" s=\"") {
        let val_start = s_start + 4; // sau ' s="'
        if let Some(val_len) = cell_xml[val_start..].find('"') {
            let mut r = String::with_capacity(cell_xml.len());
            r.push_str(&cell_xml[..val_start]);
            r.push_str(&s_val);
            r.push_str(&cell_xml[val_start + val_len..]);
            return r;
        }
    }
    // Chưa có s= → thêm sau <c
    if let Some(pos) = cell_xml.find("<c ") {
        let insert = pos + 2; // sau "<c"
        let mut r = String::with_capacity(cell_xml.len() + 8);
        r.push_str(&cell_xml[..insert]);
        r.push_str(&format!(" s=\"{s_val}\""));
        r.push_str(&cell_xml[insert..]);
        return r;
    }
    cell_xml.to_string()
}

/// Rút các mergeCell VN NẰM GỌN trong [vn_start0, vn_end0] (0-based), đổi số dòng về vùng đích theo
/// `row_delta` (target_row1 - vn_row1). Cột giữ nguyên.
fn cloned_merge_refs(
    vn_sheet_xml: &str,
    vn_start0: usize,
    vn_end0: usize,
    row_delta: isize,
) -> Vec<String> {
    let Ok(doc) = roxmltree::Document::parse(vn_sheet_xml) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for node in doc.descendants() {
        if node.tag_name().name() != "mergeCell" {
            continue;
        }
        let Some(reference) = node.attribute("ref") else {
            continue;
        };
        let Some((s_ref, e_ref)) = reference.split_once(':') else {
            continue;
        };
        let (Some((sr0, sc0)), Some((er0, ec0))) = (parse_cell_ref(s_ref), parse_cell_ref(e_ref))
        else {
            continue;
        };
        if sr0 < vn_start0 || er0 > vn_end0 {
            continue; // chỉ clone merge nằm gọn trong group
        }
        let ns = (sr0 as isize + row_delta) as usize + 1;
        let ne = (er0 as isize + row_delta) as usize + 1;
        out.push(format!(
            "{}{}:{}{}",
            col_index_to_letter(sc0),
            ns,
            col_index_to_letter(ec0),
            ne
        ));
    }
    out
}

/// Chèn các `<mergeCell ref=".."/>` mới vào `<mergeCells>` của sheet JP (tạo mới nếu chưa có).
fn inject_merge_cells(sheet_xml: &str, new_refs: &[String]) -> String {
    if new_refs.is_empty() {
        return sheet_xml.to_string();
    }
    let entries: String = new_refs
        .iter()
        .map(|r| format!(r#"<mergeCell ref="{r}"/>"#))
        .collect();

    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return sheet_xml.to_string();
    };
    if let Some(mc) = doc.descendants().find(|n| n.tag_name().name() == "mergeCells") {
        // Cập nhật count += new và append entry trước </mergeCells>.
        let old_count = mc
            .attribute("count")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let mut edits: Vec<SurgeryEdit> = Vec::new();
        if let Some(attr) = mc.attribute_node("count") {
            edits.push(SurgeryEdit {
                start: attr.range_value().start,
                end: attr.range_value().end,
                replacement: (old_count + new_refs.len()).to_string(),
            });
        }
        let insert_pos = mc.range().end - "</mergeCells>".len();
        edits.push(SurgeryEdit {
            start: insert_pos,
            end: insert_pos,
            replacement: entries,
        });
        return apply_surgery(sheet_xml, edits);
    }

    // Chưa có <mergeCells>: chèn ngay sau </sheetData> (đúng thứ tự schema OOXML).
    if let Some(sd) = doc.descendants().find(|n| n.tag_name().name() == "sheetData") {
        let insert_pos = sd.range().end;
        let block = format!(
            r#"<mergeCells count="{}">{entries}</mergeCells>"#,
            new_refs.len()
        );
        return apply_surgery(
            sheet_xml,
            vec![SurgeryEdit {
                start: insert_pos,
                end: insert_pos,
                replacement: block,
            }],
        );
    }
    sheet_xml.to_string()
}

/// Chèn dòng vào file JP tại các vị trí TL đã xác nhận. Nếu `ConfirmedInsert` có `vn_row_start/end`
/// thì CLONE nguyên group từ VN (nội dung + định dạng + merge); nếu không thì chèn dòng trống.
/// Lưu ra `output_path`.
pub fn insert_rows(
    jp_path: &str,
    vn_path: &str,
    output_path: &str,
    inserts: &[ConfirmedInsert],
) -> AppResult<RowInsertResult> {
    if inserts.is_empty() {
        return Err(AppError::new(
            "Không có vị trí chèn dòng nào được xác nhận.",
        ));
    }
    let ext = Path::new(jp_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext != "xlsx" && ext != "xlsm" {
        return Err(AppError::new(format!(
            "Chỉ hỗ trợ file .xlsx / .xlsm. File không hợp lệ: {jp_path}"
        )));
    }

    let jp_file =
        File::open(jp_path).map_err(|e| AppError::new(format!("Không mở được file JP: {e}")))?;
    let mut archive = zip::ZipArchive::new(jp_file)
        .map_err(|e| AppError::new(format!("File JP không phải ZIP hợp lệ: {e}")))?;

    let workbook_xml = read_zip_entry(&mut archive, "xl/workbook.xml")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/workbook.xml trong file JP."))?;
    let rels_xml = read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/_rels/workbook.xml.rels."))?;
    let sheet_path_map: HashMap<String, String> =
        resolve_sheet_xml_paths(&workbook_xml, &rels_xml)
            .into_iter()
            .collect();

    let mut by_sheet: HashMap<String, Vec<&ConfirmedInsert>> = HashMap::new();
    for ins in inserts {
        by_sheet.entry(ins.sheet.clone()).or_default().push(ins);
    }

    // Chuẩn bị dữ liệu VN nếu có bất kỳ lượt chèn dạng CLONE (vn_row_start/end).
    let any_clone = inserts
        .iter()
        .any(|i| i.vn_row_start.is_some() && i.vn_row_end.is_some());

    let mut replaced: HashMap<String, Vec<u8>> = HashMap::new();
    let mut xf_remap: Vec<usize> = Vec::new();
    let mut vn_plain: HashMap<usize, String> = HashMap::new();
    let mut vn_rich: HashMap<usize, String> = HashMap::new();
    let mut vn_sheet_xml_by_name: HashMap<String, String> = HashMap::new();

    if any_clone {
        let vn_file = File::open(vn_path)
            .map_err(|e| AppError::new(format!("Không mở được file VN: {e}")))?;
        let mut vn_archive = zip::ZipArchive::new(vn_file)
            .map_err(|e| AppError::new(format!("File VN không phải ZIP hợp lệ: {e}")))?;

        let vn_styles = read_zip_entry(&mut vn_archive, "xl/styles.xml").unwrap_or_default();
        let vn_sst = read_zip_entry(&mut vn_archive, "xl/sharedStrings.xml").unwrap_or_default();
        if !vn_sst.is_empty() {
            let (plain, rich) = extract_all_shared_strings(&vn_sst);
            vn_plain = plain;
            vn_rich = rich;
        }
        let vn_wb = read_zip_entry(&mut vn_archive, "xl/workbook.xml")
            .ok_or_else(|| AppError::new("Không tìm thấy xl/workbook.xml trong file VN."))?;
        let vn_rels = read_zip_entry(&mut vn_archive, "xl/_rels/workbook.xml.rels")
            .ok_or_else(|| AppError::new("Không tìm thấy xl/_rels/workbook.xml.rels trong file VN."))?;
        let vn_paths: HashMap<String, String> =
            resolve_sheet_xml_paths(&vn_wb, &vn_rels).into_iter().collect();
        for ins in inserts {
            if ins.vn_row_start.is_none() || vn_sheet_xml_by_name.contains_key(&ins.sheet) {
                continue;
            }
            if let Some(p) = vn_paths.get(&ins.sheet) {
                if let Some(x) = read_zip_entry(&mut vn_archive, p) {
                    vn_sheet_xml_by_name.insert(ins.sheet.clone(), x);
                }
            }
        }

        // Merge styles.xml của VN vào JP để có xf_remap (dùng khi clone cell/row).
        let jp_styles = read_zip_entry(&mut archive, "xl/styles.xml")
            .ok_or_else(|| AppError::new("Không tìm thấy xl/styles.xml trong file JP."))?;
        let merge = merge_vn_styles_into_jp(&jp_styles, &vn_styles);
        xf_remap = merge.xf_remap;
        replaced.insert("xl/styles.xml".to_string(), merge.new_styles_xml.into_bytes());
    }

    let mut sheets_modified: Vec<String> = Vec::new();
    let mut rows_inserted = 0usize;

    for (sheet_name, mut sheet_inserts) in by_sheet {
        let Some(xml_path) = sheet_path_map.get(&sheet_name) else {
            continue;
        };
        let Some(sheet_xml) = read_zip_entry(&mut archive, xml_path) else {
            continue;
        };
        sheet_inserts.sort_by_key(|ins| ins.jp_insert_after_row);
        let total_count: usize = sheet_inserts.iter().map(|ins| ins.insert_count).sum();

        // Base (dòng đích ngay trước dòng đầu chèn) cho từng lượt = pos + tổng count chèn trước đó.
        let vn_xml = vn_sheet_xml_by_name.get(&sheet_name);
        let mut row_jobs: Vec<RowInsert> = Vec::new();
        let mut merge_refs: Vec<String> = Vec::new();
        let mut preceding: usize = 0;
        for ins in &sheet_inserts {
            let base = ins.jp_insert_after_row + preceding;
            match (ins.vn_row_start, ins.vn_row_end, vn_xml) {
                (Some(vs), Some(ve), Some(vx)) if ve >= vs => {
                    let (vn_start0, vn_end0) = (vs - 1, ve - 1);
                    // row_delta: dòng VN đầu (vs) → dòng đích đầu (base+1).
                    let row_delta = (base as isize + 1) - vs as isize;
                    merge_refs.extend(cloned_merge_refs(vx, vn_start0, vn_end0, row_delta));
                    let vx_owned = vx.clone();
                    let remap = xf_remap.clone();
                    let plain = vn_plain.clone();
                    let rich = vn_rich.clone();
                    row_jobs.push(RowInsert {
                        pos: ins.jp_insert_after_row,
                        count: ins.insert_count,
                        build: Box::new(move |b| {
                            clone_vn_rows_xml(&vx_owned, vn_start0, vn_end0, b, &remap, &plain, &rich)
                        }),
                    });
                }
                _ => {
                    row_jobs.push(RowInsert {
                        pos: ins.jp_insert_after_row,
                        count: ins.insert_count,
                        build: blank_rows_builder(ins.insert_count),
                    });
                }
            }
            preceding += ins.insert_count;
        }

        let mut new_xml = insert_rows_into_sheet_xml(&sheet_xml, &row_jobs);
        new_xml = inject_merge_cells(&new_xml, &merge_refs);
        replaced.insert(xml_path.clone(), new_xml.into_bytes());
        sheets_modified.push(sheet_name);
        rows_inserted += total_count;
    }

    write_output_zip(&mut archive, &replaced, output_path)?;

    Ok(RowInsertResult {
        output_path: output_path.to_string(),
        sheets_modified,
        rows_inserted,
    })
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
// Kiểm tra AI cho ô đỏ: dịch VN→JP CHỈ để so sánh (không ghi vào tài liệu), rồi so độ
// tương đồng với nội dung JP hiện có — giúp phát hiện ô có thể không thật sự thay đổi,
// hoặc dòng bị lệch (nội dung dịch giống 1 vị trí JP khác hơn là vị trí đang xét).
// ─────────────────────────────────────────────────────────────────────────────

/// Levenshtein distance (số ký tự thêm/xóa/sửa tối thiểu để biến chuỗi a thành b).
fn levenshtein_distance(a: &[char], b: &[char]) -> usize {
    let (n, m) = (a.len(), b.len());
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Độ tương đồng 2 chuỗi (0-100%), dựa trên Levenshtein distance chuẩn hóa theo độ dài.
fn text_similarity(a: &str, b: &str) -> f32 {
    let a = a.trim();
    let b = b.trim();
    if a.is_empty() && b.is_empty() {
        return 100.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let dist = levenshtein_distance(&a_chars, &b_chars);
    let max_len = a_chars.len().max(b_chars.len()) as f32;
    ((1.0 - (dist as f32 / max_len)) * 100.0).max(0.0)
}

/// Dịch VN→JP hàng loạt (chỉ để so sánh, KHÔNG ghi vào tài liệu) cho danh sách ô đỏ đã phân
/// tích, rồi so độ tương đồng với nội dung JP hiện có tại cùng vị trí và các vị trí lân cận
/// (cùng cột, ±5 dòng) trong cùng sheet để gợi ý khả năng lệch dòng. Gọi AI song song theo
/// từng lô nhỏ để không chặn quá lâu; lỗi ở 1 ô (mất mạng, hết quota...) chỉ bỏ qua ô đó,
/// không làm hỏng cả lượt kiểm tra.
pub async fn verify_red_cells_with_ai(
    jp_path: &str,
    red_cells: &[RedCell],
    provider: &str,
    model: &str,
) -> AppResult<RedCellVerificationReport> {
    if red_cells.is_empty() {
        return Ok(RedCellVerificationReport { items: Vec::new() });
    }

    let provider_lc = provider.trim().to_lowercase();
    let api_key = resolve_api_key(&provider_lc, None)?;
    let client = build_http_client()?;
    let jp_grid = read_workbook_grid(jp_path)?;

    const CHUNK_SIZE: usize = 5;
    const NEARBY_WINDOW: usize = 5;
    const BETTER_MATCH_MARGIN: f32 = 15.0;

    let mut items: Vec<RedCellVerification> = Vec::with_capacity(red_cells.len());

    for chunk in red_cells.chunks(CHUNK_SIZE) {
        let mut handles = Vec::with_capacity(chunk.len());
        for rc in chunk {
            let client = client.clone();
            let api_key = api_key.clone();
            let model = model.to_string();
            let provider_lc = provider_lc.clone();
            let vn_text = rc.vn_text.clone();
            handles.push(tokio::spawn(async move {
                let prompt = format!(
                    "あなたはプロの技術文書翻訳者です。以下のベトナム語テキストを日本語に翻訳してください。\
これは技術設計仕様書のテキストです。翻訳文のみ出力し、説明は不要です。\n\nベトナム語テキスト: {}",
                    vn_text
                );
                match provider_lc.as_str() {
                    "gemini" => call_gemini(&client, &api_key, &model, &prompt).await,
                    "groq" => call_groq(&client, &api_key, &model, &prompt).await,
                    other => Err(AppError::new(format!(
                        "Nhà cung cấp không được hỗ trợ: '{other}'."
                    ))),
                }
            }));
        }

        for (rc, handle) in chunk.iter().zip(handles) {
            let translation = match handle.await {
                Ok(Ok(t)) => t,
                _ => continue,
            };

            let same_pos_sim = text_similarity(&translation, &rc.jp_text);

            let mut best: Option<BetterMatch> = None;
            if let Some(sheet_grid) = jp_grid.get(&rc.sheet) {
                let row_0 = rc.row - 1;
                let col_0 = rc.col - 1;
                let lo = row_0.saturating_sub(NEARBY_WINDOW);
                let hi = (row_0 + NEARBY_WINDOW).min(sheet_grid.len().saturating_sub(1));
                for r in lo..=hi {
                    if r == row_0 {
                        continue;
                    }
                    let Some(cell) = sheet_grid.get(r).and_then(|row| row.get(col_0)) else {
                        continue;
                    };
                    if cell.trim().is_empty() {
                        continue;
                    }
                    let sim = text_similarity(&translation, cell);
                    if sim > same_pos_sim + BETTER_MATCH_MARGIN
                        && best.as_ref().map_or(true, |b| sim > b.similarity)
                    {
                        best = Some(BetterMatch {
                            row: r + 1,
                            col: col_0 + 1,
                            similarity: sim,
                        });
                    }
                }
            }

            items.push(RedCellVerification {
                sheet: rc.sheet.clone(),
                row: rc.row,
                col: rc.col,
                ai_translation: translation,
                similarity_same_pos: same_pos_sim,
                better_match: best,
            });
        }
    }

    Ok(RedCellVerificationReport { items })
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

// ─────────────────────────────────────────────────────────────────────────────
// Per-sheet pipeline helpers — text-similarity row alignment (dự phòng cho doc type khác)
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả canh dòng theo nội dung cột B: cặp VN↔JP đã khớp và dòng VN không có JP đối ứng.
pub struct ContentAlignment {
    /// (vn_row1, jp_row1) — khớp theo thứ tự tăng dần vn_row1
    pub matched: Vec<(usize, usize)>,
    /// Dòng VN không tìm được JP tương ứng (cần chèn vào JP), tăng dần
    pub vn_only: Vec<usize>,
}

/// Trích (row1, plain_text) cho cột `col0` (0-based) từ `sheet_xml`, bắt đầu từ `start_row1`.
/// Bỏ qua ô rỗng / shared string không tồn tại trong `sst_plain`.
pub(crate) fn extract_col_texts(
    sheet_xml: &str,
    sst_plain: &HashMap<usize, String>,
    col0: usize,
    start_row1: usize,
) -> Vec<(usize, String)> {
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return Vec::new();
    };
    let mut out: Vec<(usize, String)> = Vec::new();
    for cell in doc.descendants().filter(|n| n.tag_name().name() == "c") {
        let Some(r_attr) = cell.attribute("r") else { continue };
        let Some((row0, c)) = parse_cell_ref(r_attr) else { continue };
        if c != col0 { continue }
        let row1 = row0 + 1;
        if row1 < start_row1 { continue }

        let text = match cell.attribute("t") {
            Some("s") => {
                let idx = cell
                    .children()
                    .find(|n| n.tag_name().name() == "v")
                    .and_then(|v| v.text())
                    .and_then(|t| t.parse::<usize>().ok());
                idx.and_then(|i| sst_plain.get(&i)).cloned().unwrap_or_default()
            }
            Some("inlineStr") => cell
                .descendants()
                .filter(|n| n.tag_name().name() == "t")
                .filter_map(|n| n.text())
                .collect::<Vec<_>>()
                .join(""),
            _ => cell
                .children()
                .find(|n| n.tag_name().name() == "v")
                .and_then(|v| v.text())
                .unwrap_or("")
                .to_string(),
        };

        let text = text.trim().to_string();
        if !text.is_empty() {
            out.push((row1, text));
        }
    }
    out.sort_by_key(|(r, _)| *r);
    out
}

/// Điểm tương đồng text VN↔JP (0.0–1.0), dành riêng cho canh dòng theo nội dung cột.
/// VN B text thường có dạng "JP text\r\nVN translation" — lấy dòng đầu làm cơ sở so khớp.
fn col_text_similarity(vn_raw: &str, jp_text: &str) -> f64 {
    let vn_first = vn_raw
        .split(['\n', '\r'])
        .find(|s| !s.trim().is_empty())
        .unwrap_or(vn_raw)
        .trim();
    let jp = jp_text.trim();
    if vn_first.is_empty() || jp.is_empty() {
        return 0.0;
    }
    if vn_first == jp || vn_first.starts_with(jp) || jp.starts_with(vn_first) {
        return 1.0;
    }
    if vn_first.contains(jp) || jp.contains(vn_first) {
        return 0.9;
    }
    let vn_chars: Vec<char> = vn_first.chars().collect();
    let jp_chars: Vec<char> = jp.chars().collect();
    let common = vn_chars.iter().zip(jp_chars.iter()).take_while(|(a, b)| a == b).count();
    let max_len = vn_chars.len().max(jp_chars.len());
    let ratio = if max_len == 0 { 0.0 } else { common as f64 / max_len as f64 };
    let jp_len = jp_chars.len();
    if common > 0 && jp_len > 0 && common * 2 >= jp_len {
        (ratio + 0.1).min(1.0)
    } else {
        ratio
    }
}

/// Canh dòng VN↔JP theo nội dung cột B thay vì row number.
/// Greedy matching theo thứ tự — mỗi dòng VN tìm JP khớp nhất chưa được dùng, nằm sau JP đã match.
#[allow(dead_code)]
pub(crate) fn align_rows_by_col_text(
    vn_col: &[(usize, String)],
    jp_col: &[(usize, String)],
) -> ContentAlignment {
    const THRESHOLD: f64 = 0.45;

    let mut matched: Vec<(usize, usize)> = Vec::new();
    let mut jp_used = vec![false; jp_col.len()];
    let mut jp_start_idx = 0usize;

    for (vn_r, vn_text) in vn_col {
        let mut best_score = THRESHOLD;
        let mut best_ji: Option<usize> = None;

        for (ji, (_, jp_text)) in jp_col.iter().enumerate().skip(jp_start_idx) {
            if jp_used[ji] {
                continue;
            }
            let score = col_text_similarity(vn_text, jp_text);
            if score > best_score {
                best_score = score;
                best_ji = Some(ji);
                if score >= 0.95 {
                    break;
                }
            }
        }

        if let Some(ji) = best_ji {
            matched.push((*vn_r, jp_col[ji].0));
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

#[allow(dead_code)]
/// Trích plain text của MỌI ô trong sheet, trả về map `(row1, col0) → text`.
pub(crate) fn extract_cell_texts_map(
    sheet_xml: &str,
    sst_plain: &HashMap<usize, String>,
) -> HashMap<(usize, usize), String> {
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return HashMap::new();
    };
    let mut map: HashMap<(usize, usize), String> = HashMap::new();
    for cell in doc.descendants().filter(|n| n.tag_name().name() == "c") {
        let Some(r_attr) = cell.attribute("r") else {
            continue;
        };
        let Some((row0, col0)) = parse_cell_ref(r_attr) else {
            continue;
        };
        let text = match cell.attribute("t") {
            Some("s") => cell
                .children()
                .find(|n| n.tag_name().name() == "v")
                .and_then(|v| v.text())
                .and_then(|t| t.parse::<usize>().ok())
                .and_then(|i| sst_plain.get(&i))
                .cloned()
                .unwrap_or_default(),
            Some("inlineStr") => cell
                .descendants()
                .filter(|n| n.tag_name().name() == "t")
                .filter_map(|n| n.text())
                .collect::<Vec<_>>()
                .join(""),
            _ => cell
                .children()
                .find(|n| n.tag_name().name() == "v")
                .and_then(|v| v.text())
                .unwrap_or("")
                .to_string(),
        };
        let text = text.trim().to_string();
        if !text.is_empty() {
            map.insert((row0 + 1, col0), text);
        }
    }
    map
}

#[allow(dead_code)]
/// Tìm ô có giá trị VN ≠ JP tại các dòng aligned, trả về `HashSet<(row0, col0)>` (vn_row0).
/// Dùng để bổ sung vào `vn_changed_positions` khi VN thay đổi giá trị mà không đổi màu chữ.
pub(crate) fn find_value_diff_cells(
    vn_texts: &HashMap<(usize, usize), String>,
    jp_texts: &HashMap<(usize, usize), String>,
    matched: &[(usize, usize)],
) -> HashSet<(usize, usize)> {
    let mut result: HashSet<(usize, usize)> = HashSet::new();
    for &(vn_r1, jp_r1) in matched {
        let mut cols: HashSet<usize> = HashSet::new();
        for &(r, c) in vn_texts.keys() {
            if r == vn_r1 {
                cols.insert(c);
            }
        }
        for &(r, c) in jp_texts.keys() {
            if r == jp_r1 {
                cols.insert(c);
            }
        }
        for col0 in cols {
            let vn_val = vn_texts.get(&(vn_r1, col0)).map(String::as_str).unwrap_or("");
            let jp_val = jp_texts.get(&(jp_r1, col0)).map(String::as_str).unwrap_or("");
            if vn_val != jp_val {
                result.insert((vn_r1 - 1, col0));
            }
        }
    }
    result
}

#[allow(dead_code)]
/// Với dòng VN-only `vn_only_row1`, trả về JP row1 sẽ chèn SAU (0 = chèn trước dòng đầu tiên).
///
/// LƯU Ý: đã thử wire vào `c234_sync_service::apply_changes` (chèn thật dòng VN-only vào JP
/// trước khi merge) rồi REVERT — với sheet thiếu neo đáng tin cậy (xem `align_vn_jp_row_map`),
/// `alignment.matched` do `align_vn_jp_row_map` cung cấp có thể chứa cặp SAI (gap-fill không có
/// neo thật), khiến chèn dòng SAI VỊ TRÍ rồi làm lệch toàn bộ cấu trúc JP phía sau — hậu quả NẶNG
/// HƠN so với chỉ tra cứu ảo lúc clone (1 ô sai thay vì cả khối dòng sau đó sai). Chỉ nên dùng lại
/// hàm này khi có nguồn `matched` đáng tin cậy hơn (vd canh dòng theo nội dung thay vì theo neo).
pub(crate) fn vn_only_insert_after_jp(vn_only_row1: usize, matched: &[(usize, usize)]) -> usize {
    matched
        .iter()
        .filter(|(vn_r, _)| *vn_r < vn_only_row1)
        .last()
        .map(|(_, jp_r)| *jp_r)
        .unwrap_or(0)
}

#[allow(dead_code)]
/// Số dòng VN-only chèn TRƯỚC jp_row1 (làm shift cho JP row đó).
pub(crate) fn count_inserts_before_jp(alignment: &ContentAlignment, jp_row1: usize) -> usize {
    alignment
        .vn_only
        .iter()
        .filter(|&&vn_r| vn_only_insert_after_jp(vn_r, &alignment.matched) < jp_row1)
        .count()
}

/// Clone 1 dòng VN (raw XML) sang JP với số dòng = `target_row1`, bỏ qua các cột `skip_col0s`.
fn build_vn_row_skipping_cols(
    row_xml: &str,
    target_row1: usize,
    xf_remap: &[usize],
    plain_ssi: &HashMap<usize, String>,
    rich_ssi_raw: &HashMap<usize, String>,
    skip_col0s: &[usize],
) -> String {
    let wrapped = format!("<root>{}</root>", row_xml);
    let Ok(doc) = roxmltree::Document::parse(&wrapped) else {
        return format!(r#"<row r="{}"/>"#, target_row1);
    };
    let Some(row_node) = doc.descendants().find(|n| n.tag_name().name() == "row") else {
        return format!(r#"<row r="{}"/>"#, target_row1);
    };

    let mut attrs = String::new();
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
                    Some(v) => attrs.push_str(&format!(" s=\"{v}\"")),
                    None => attrs.push_str(&format!(" s=\"{}\"", a.value())),
                }
            }
            name => attrs.push_str(&format!(" {}=\"{}\"", name, xml_escape_attr(a.value()))),
        }
    }

    let cells: String = row_node
        .children()
        .filter(|c| c.tag_name().name() == "c")
        .filter(|c| {
            let col0 = c
                .attribute("r")
                .and_then(parse_cell_ref)
                .map(|(_, col)| col)
                .unwrap_or(usize::MAX);
            !skip_col0s.contains(&col0)
        })
        .map(|c| clone_vn_cell_xml(c, &wrapped, target_row1, xf_remap, plain_ssi, rich_ssi_raw))
        .collect();

    format!(r#"<row r="{target_row1}"{attrs}>{cells}</row>"#)
}

#[allow(dead_code)]
/// Chèn các dòng VN-only vào JP sheet XML, bỏ qua cột trong `skip_col0s` (vd [0] = cột A).
pub(crate) fn insert_vn_only_rows_into_sheet_xml(
    jp_sheet_xml: &str,
    vn_sheet_xml: &str,
    alignment: &ContentAlignment,
    skip_col0s: &[usize],
    xf_remap: &[usize],
    plain_ssi: &HashMap<usize, String>,
    rich_ssi_raw: &HashMap<usize, String>,
) -> String {
    if alignment.vn_only.is_empty() {
        return jp_sheet_xml.to_string();
    }

    let Ok(vn_doc) = roxmltree::Document::parse(vn_sheet_xml) else {
        return jp_sheet_xml.to_string();
    };
    let vn_row_xmls: HashMap<usize, String> = vn_doc
        .descendants()
        .filter(|n| n.tag_name().name() == "row")
        .filter_map(|n| {
            n.attribute("r")
                .and_then(|s| s.parse::<usize>().ok())
                .map(|r1| (r1, vn_sheet_xml[n.range()].to_string()))
        })
        .collect();

    // Gom theo vị trí chèn (BTreeMap để xử lý theo thứ tự tăng dần after_jp)
    let mut groups: std::collections::BTreeMap<usize, Vec<usize>> =
        std::collections::BTreeMap::new();
    for &vn_r in &alignment.vn_only {
        let after = vn_only_insert_after_jp(vn_r, &alignment.matched);
        groups.entry(after).or_default().push(vn_r);
    }

    let mut row_inserts: Vec<RowInsert> = Vec::new();
    for (after_jp, vn_rows) in groups {
        let row_xmls: Vec<String> = vn_rows
            .iter()
            .filter_map(|&r1| vn_row_xmls.get(&r1).cloned())
            .collect();
        let count = row_xmls.len();
        let xf = xf_remap.to_vec();
        let plain = plain_ssi.clone();
        let rich = rich_ssi_raw.clone();
        let skip = skip_col0s.to_vec();

        row_inserts.push(RowInsert {
            pos: after_jp,
            count,
            build: Box::new(move |base| {
                row_xmls
                    .iter()
                    .enumerate()
                    .map(|(i, row_xml)| {
                        build_vn_row_skipping_cols(row_xml, base + i + 1, &xf, &plain, &rich, &skip)
                    })
                    .collect()
            }),
        });
    }

    insert_rows_into_sheet_xml(jp_sheet_xml, &row_inserts)
}

// ─────────────────────────────────────────────────────────────────────────────
// Clone toàn bộ sheet VN sang JP output — dùng cho c234_sync_service
// ─────────────────────────────────────────────────────────────────────────────

/// Trích bảng text thô (index 0-based = row1 - 1) từ 1 sheet XML — chỉ lấy nội dung text hiển
/// thị, dùng làm input tính neo alignment (`align_vn_jp_row_map`), không cần style.
fn extract_sheet_text_grid(sheet_xml: &str, plain_ssi: &HashMap<usize, String>) -> Vec<Vec<String>> {
    let mut grid: Vec<Vec<String>> = Vec::new();
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return grid;
    };
    let Some(sd) = doc.descendants().find(|n| n.tag_name().name() == "sheetData") else {
        return grid;
    };
    for row in sd.children().filter(|n| n.tag_name().name() == "row") {
        let row1 = row
            .attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        if row1 == 0 {
            continue;
        }
        while grid.len() < row1 {
            grid.push(Vec::new());
        }
        let row_vec = &mut grid[row1 - 1];
        for cell in row.children().filter(|n| n.tag_name().name() == "c") {
            let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) else {
                continue;
            };
            while row_vec.len() <= col0 {
                row_vec.push(String::new());
            }
            row_vec[col0] = extract_cell_plain_text(cell, plain_ssi).unwrap_or_default();
        }
    }
    grid
}

/// Nguồn nội dung cho 1 dòng được chèn vật lý vào JP — xem `compute_row_insertions`.
#[derive(Clone, Copy)]
pub(crate) enum RowInsertSource {
    /// Clone nguyên dòng VN (row1) — dòng hoàn toàn mới ở VN (rule 1).
    Vn(usize),
    /// Dòng trống — VN không có gì tại vị trí này nhưng JP lại có dữ liệu (rule 3): chèn 1 dòng
    /// trống để "đẩy" dữ liệu JP đó xuống, giữ đúng vị trí so sánh cho dòng VN kế tiếp.
    Blank,
}

/// Tính danh sách các lượt CHÈN DÒNG VẬT LÝ vào JP trước khi merge nội dung, theo rule do người
/// dùng chỉ định (không dựa vào neo/anchor — anchor như STT tự đánh số lại không cho tín hiệu gì
/// khi có dòng chèn, xem doc `align_vn_jp_row_map`). Đi qua VN và JP bằng 2 con trỏ độc lập
/// (`vn_row1`, `jp_row1`), trong phạm vi cột `bounds` (trừ các cột ở `skip_col0s`, ví dụ cột A/STT
/// tự đánh số lại — luôn có dữ liệu và luôn đen nên phải loại khỏi việc xét "đã đổi"):
///
/// 1. Nếu dòng VN có ≥1 ô dữ liệu và MỌI ô dữ liệu đó đều "TOÀN Ô đã đổi" (`vn_fully_changed_positions`
///    — MỌI run trong ô đều màu khác đen hoặc gạch bỏ, không còn run đen/thường nào, xem
///    `find_fully_changed_cells_xlsx`) → dòng hoàn toàn mới ở VN → chèn 1 dòng clone từ VN vào JP
///    ngay tại vị trí `jp_row1` hiện tại (đẩy JP xuống); chỉ `vn_row1` tăng — `jp_row1` GIỮ NGUYÊN
///    vì dữ liệu JP tại đó chưa được dùng, sẽ so tiếp với dòng VN kế.
///    Lưu ý: dùng tiêu chí NGHIÊM (toàn ô, không phải ANY-run như `find_changed_style_cells_xlsx`)
///    — 1 ô "tên cũ gạch bỏ đen thường + tên mới tô đỏ" (dòng ĐỔI TÊN, không phải dòng MỚI) vẫn có
///    run đen/thường (phần tên cũ, nếu tự nó không gạch/đổi màu) nên KHÔNG được tính là "đã đổi
///    toàn ô" — nếu dùng tiêu chí ANY-based, các dòng đổi tên kiểu này sẽ bị nhận lầm thành "dòng
///    mới" (đã xảy ra thực tế: dòng có 1 ghi chú tham khảo tô đỏ chèn thêm vào ô D bị nhận lầm).
/// 2. Nếu dòng VN hoàn toàn trống (không ô nào có dữ liệu trong phạm vi):
///    a. Dòng JP tại CÙNG `jp_row1` cũng trống toàn bộ → không chèn gì, 2 con trỏ cùng tăng.
///    b. Dòng JP tại CÙNG `jp_row1` CÓ dữ liệu → chèn 1 dòng TRỐNG vào JP (đẩy dữ liệu JP đó
///       xuống 1 dòng) để dòng VN kế tiếp so đúng với dữ liệu JP đó; `vn_row1` tăng, `jp_row1`
///       GIỮ NGUYÊN (dữ liệu JP chưa được "dùng" bởi dòng VN nào).
/// 3. Ngoài 2 trường hợp trên (dòng "thường" — có ô đã đổi lẫn ô chưa đổi, hoặc mọi ô chưa đổi)
///    → KHÔNG chèn, coi dòng VN này khớp 1:1 với JP tại `jp_row1` hiện tại; 2 con trỏ cùng tăng.
///
/// Vòng lặp LUÔN tăng `vn_row1` mỗi bước nên chắc chắn kết thúc (không cần safety-net riêng).
///
/// Trả về `Vec<(jp_row1_gốc, source)>` theo thứ tự tăng dần — `jp_row1_gốc` là row1 GỐC (chưa
/// shift bởi lượt chèn nào khác) của JP mà lượt chèn này diễn ra NGAY TRƯỚC, dùng trực tiếp làm
/// `RowInsert::pos` (`pos` = "chèn sau row gốc pos", tức "chèn trước row gốc pos+1").
pub(crate) fn compute_row_insertions(
    vn_sheet_xml: &str,
    jp_sheet_xml: &str,
    vn_plain_ssi: &HashMap<usize, String>,
    jp_plain_ssi: &HashMap<usize, String>,
    vn_fully_changed_positions: &HashSet<(usize, usize)>,
    bounds: ContentBounds,
    skip_col0s: &[usize],
) -> Vec<(usize, RowInsertSource)> {
    let vn_grid = extract_sheet_text_grid(vn_sheet_xml, vn_plain_ssi);
    let jp_grid = extract_sheet_text_grid(jp_sheet_xml, jp_plain_ssi);

    let data_cols = |grid: &[Vec<String>], row1: usize| -> Vec<usize> {
        let row0 = row1 - 1;
        grid.get(row0)
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter(|&(c, text)| {
                        c <= bounds.last_col0 && !skip_col0s.contains(&c) && !text.trim().is_empty()
                    })
                    .map(|(c, _)| c)
                    .collect()
            })
            .unwrap_or_default()
    };

    let mut result: Vec<(usize, RowInsertSource)> = Vec::new();
    let mut vn_row1 = bounds.start_row0 + 1;
    let mut jp_row1 = bounds.start_row0 + 1;
    let vn_max_row1 = vn_grid.len();

    while vn_row1 <= vn_max_row1 {
        let vn_cols = data_cols(&vn_grid, vn_row1);

        if vn_cols.is_empty() {
            let jp_cols = data_cols(&jp_grid, jp_row1);
            if jp_cols.is_empty() {
                vn_row1 += 1;
                jp_row1 += 1;
            } else {
                result.push((jp_row1 - 1, RowInsertSource::Blank));
                vn_row1 += 1;
            }
            continue;
        }

        let all_changed = vn_cols
            .iter()
            .all(|&c| vn_fully_changed_positions.contains(&(vn_row1 - 1, c)));
        if all_changed {
            result.push((jp_row1 - 1, RowInsertSource::Vn(vn_row1)));
            vn_row1 += 1;
        } else {
            vn_row1 += 1;
            jp_row1 += 1;
        }
    }

    result
}

/// Áp danh sách lượt chèn từ `compute_row_insertions` vào `jp_sheet_xml` — chèn vật lý (dịch
/// row/cell-ref/mergeCell/dimension phía sau như `insert_rows_into_sheet_xml`), không phải ánh xạ
/// ảo. Gom các lượt chèn CÙNG vị trí (`pos`) thành 1 nhóm liên tiếp để giữ đúng thứ tự.
pub(crate) fn apply_row_insertions(
    jp_sheet_xml: &str,
    vn_sheet_xml: &str,
    insertions: &[(usize, RowInsertSource)],
    skip_col0s: &[usize],
    xf_remap: &[usize],
    plain_ssi: &HashMap<usize, String>,
    rich_ssi_raw: &HashMap<usize, String>,
) -> String {
    if insertions.is_empty() {
        return jp_sheet_xml.to_string();
    }

    let Ok(vn_doc) = roxmltree::Document::parse(vn_sheet_xml) else {
        return jp_sheet_xml.to_string();
    };
    let vn_row_xmls: HashMap<usize, String> = vn_doc
        .descendants()
        .filter(|n| n.tag_name().name() == "row")
        .filter_map(|n| {
            n.attribute("r")
                .and_then(|s| s.parse::<usize>().ok())
                .map(|r1| (r1, vn_sheet_xml[n.range()].to_string()))
        })
        .collect();

    // Gom theo vị trí chèn (BTreeMap để xử lý theo thứ tự tăng dần pos), giữ thứ tự trong nhóm.
    let mut groups: std::collections::BTreeMap<usize, Vec<RowInsertSource>> =
        std::collections::BTreeMap::new();
    for &(pos, source) in insertions {
        groups.entry(pos).or_default().push(source);
    }

    let mut row_inserts: Vec<RowInsert> = Vec::new();
    for (pos, sources) in groups {
        let count = sources.len();
        let xf = xf_remap.to_vec();
        let plain = plain_ssi.clone();
        let rich = rich_ssi_raw.clone();
        let skip = skip_col0s.to_vec();
        let vn_rows = vn_row_xmls.clone();

        row_inserts.push(RowInsert {
            pos,
            count,
            build: Box::new(move |base| {
                sources
                    .iter()
                    .enumerate()
                    .map(|(i, src)| {
                        let target_row1 = base + i + 1;
                        match src {
                            RowInsertSource::Blank => format!(r#"<row r="{target_row1}"/>"#),
                            RowInsertSource::Vn(vn_r1) => vn_rows
                                .get(vn_r1)
                                .map(|row_xml| {
                                    build_vn_row_skipping_cols(
                                        row_xml,
                                        target_row1,
                                        &xf,
                                        &plain,
                                        &rich,
                                        &skip,
                                    )
                                })
                                .unwrap_or_else(|| format!(r#"<row r="{target_row1}"/>"#)),
                        }
                    })
                    .collect()
            }),
        });
    }

    insert_rows_into_sheet_xml(jp_sheet_xml, &row_inserts)
}

/// Tính ánh xạ vn_row1 → jp_row1 ĐẦY ĐỦ cho mọi dòng có nội dung — không chỉ riêng các dòng "neo"
/// (số/mã kỹ thuật, xem `is_anchor_cell`) khớp được bằng LCS, mà cả các dòng NẰM GIỮA 2 neo liên
/// tiếp, được ghép tuần tự theo đúng thứ tự xuất hiện trong từng khoảng hở đó — tương đương hiệu
/// ứng của việc "chèn dòng" thật trong Excel (đẩy mọi dòng sau vị trí chèn xuống đúng 1 dòng) mà
/// không cần mutate vật lý cấu trúc `sheetData`/`mergeCells` của JP. Nếu chỉ map riêng dòng neo
/// (bỏ qua dòng ở giữa), MỌI ô không phải neo sau điểm chèn (ví dụ nội dung text thường) vẫn bị
/// tra theo ánh xạ 1:1 sai — CÙNG cơ chế với `c233_align_rows`/`build_complete_row_mapping` của
/// `c233_sync_service`, tái dùng cho `clone_vn_sheet_for_jp` mọi loại tài liệu khác.
///
/// `vn_changed_positions` — ô VN "coi như đã thay đổi" (đỏ/gạch bỏ/style khác) của SHEET này; dòng
/// chứa bất kỳ ô nào trong tập này bị loại khỏi vai trò "neo" vì nội dung không ổn định (có thể là
/// dòng mới) — chỉ dòng VN "sạch" mới đủ tin cậy để khớp 1:1 với JP. Dòng đã đổi VẪN được lấp vào
/// khoảng hở tuần tự (có thể còn ô khác trong dòng không đổi, cần tra đúng JP).
///
/// An toàn khi không tìm được neo (sheet không có ô neo, hoặc quá lớn): trả về map rỗng, khi đó
/// `clone_vn_sheet_for_jp` tự rơi về ánh xạ 1:1 như hành vi cũ — không có rủi ro map sai lệch.
pub(crate) fn align_vn_jp_row_map(
    vn_sheet_xml: &str,
    jp_sheet_xml: &str,
    vn_plain_ssi: &HashMap<usize, String>,
    jp_plain_ssi: &HashMap<usize, String>,
    vn_changed_positions: &HashSet<(usize, usize)>,
    bounds: ContentBounds,
) -> HashMap<usize, usize> {
    let vn_grid = extract_sheet_text_grid(vn_sheet_xml, vn_plain_ssi);
    let jp_grid = extract_sheet_text_grid(jp_sheet_xml, jp_plain_ssi);
    let changed_rows: HashSet<usize> = vn_changed_positions.iter().map(|&(r, _)| r).collect();

    // Bỏ CỘT A khỏi vùng quét ô neo: cột A ở vùng nội dung các loại tài liệu này luôn là STT tự
    // đánh số lại tuần tự (1,2,3...) ở CẢ VN và JP — khi VN chèn N dòng, STT phía sau tự renumber
    // liền mạch, không hề để lại dấu hiệu gì (không trùng, không nhảy số) để LCS phát hiện chèn
    // dòng: "STT=6" ở VN luôn "khớp" ngay với "STT=6" ở JP dù 2 dòng đó là 2 bản ghi hoàn toàn
    // khác nhau (JP đã bị đẩy lệch). Coi cột A như không tồn tại khi tìm neo — chỉ dùng các cột
    // nội dung khác (mã kỹ thuật, số liệu KHÔNG tự đánh số lại) làm neo đáng tin cậy.
    let vn_anchors: Vec<(usize, String)> = vn_grid
        .iter()
        .enumerate()
        .filter(|(r, _)| !changed_rows.contains(r))
        .filter(|(r, _)| *r >= bounds.start_row0)
        .filter_map(|(r, row)| row_anchor_key(anchor_slice_excl_col_a(row, bounds)).map(|k| (r, k)))
        .collect();
    let jp_anchors: Vec<(usize, String)> = jp_grid
        .iter()
        .enumerate()
        .filter(|(r, _)| *r >= bounds.start_row0)
        .filter_map(|(r, row)| row_anchor_key(anchor_slice_excl_col_a(row, bounds)).map(|k| (r, k)))
        .collect();

    // Không có neo nào đủ tin cậy ở 1 trong 2 phía (rất có thể với sheet nội dung chủ yếu là câu
    // văn tự do, gần như mọi dòng đều "đã đổi" theo quy ước tô đỏ tên hạng mục — không còn tín
    // hiệu gì phân biệt "dòng chèn mới" khỏi "dòng đã có nhưng đổi tên") — AN TOÀN nhất là bỏ
    // cuộc, trả map rỗng để nơi gọi rơi về ánh xạ 1:1 như cũ. ĐÃ THỬ lấp khoảng hở "ưu tiên dòng
    // chưa đổi" khi không có neo — với sheet mà "đã đổi" là PHỔ BIẾN (không phải ngoại lệ), heuristic
    // đó suy luận ngược, xáo trộn nội dung sang vị trí hoàn toàn không liên quan còn tệ hơn ánh xạ
    // 1:1 — nên bỏ, chấp nhận không tự động bù được lệch dòng khi thiếu neo thay vì đoán sai.
    if vn_anchors.is_empty() || jp_anchors.is_empty() {
        return HashMap::new();
    }
    if vn_anchors.len().saturating_mul(jp_anchors.len()) > MAX_ANCHOR_LCS_CELLS {
        return HashMap::new();
    }

    // (vn_row0, jp_row0) — chỉ các cặp neo khớp được, KHÔNG phải mapping đầy đủ.
    let matched: Vec<(usize, usize)> = lcs_match(&vn_anchors, &jp_anchors);
    let matched_vn: HashSet<usize> = matched.iter().map(|&(v, _)| v).collect();
    let matched_jp: HashSet<usize> = matched.iter().map(|&(_, j)| j).collect();

    let mut result: HashMap<usize, usize> = matched
        .iter()
        .map(|&(vn_r0, jp_r0)| (vn_r0 + 1, jp_r0 + 1))
        .collect();

    // Mọi dòng CÓ NỘI DUNG (bất kể đã đổi hay chưa) trong vùng nội dung — dùng để lấp khoảng hở
    // giữa 2 neo liên tiếp bằng ghép tuần tự.
    let has_content = |row: &[String]| row.iter().any(|c| !c.trim().is_empty());
    let vn_content_rows: Vec<usize> = vn_grid
        .iter()
        .enumerate()
        .filter(|(r, row)| *r >= bounds.start_row0 && has_content(row))
        .map(|(r, _)| r)
        .collect();
    let jp_content_rows: Vec<usize> = jp_grid
        .iter()
        .enumerate()
        .filter(|(r, row)| *r >= bounds.start_row0 && has_content(row))
        .map(|(r, _)| r)
        .collect();

    let vn_min = vn_content_rows.first().copied().unwrap_or(0);
    let vn_max = vn_content_rows.last().copied().unwrap_or(0);
    let jp_min = jp_content_rows.first().copied().unwrap_or(0);
    let jp_max = jp_content_rows.last().copied().unwrap_or(0);

    let mut anchors_sorted = matched;
    anchors_sorted.sort();

    // (vn_lo_incl, vn_hi_excl, jp_lo_incl, jp_hi_excl) — khoảng hở TRƯỚC neo đầu, GIỮA mỗi cặp
    // neo liên tiếp, và SAU neo cuối.
    let mut segments: Vec<(usize, usize, usize, usize)> = Vec::new();
    if anchors_sorted.is_empty() {
        segments.push((vn_min, vn_max + 1, jp_min, jp_max + 1));
    } else {
        let (fv, fj) = anchors_sorted[0];
        segments.push((vn_min, fv, jp_min, fj));
        for w in anchors_sorted.windows(2) {
            segments.push((w[0].0 + 1, w[1].0, w[0].1 + 1, w[1].1));
        }
        let (lv, lj) = *anchors_sorted.last().unwrap();
        segments.push((lv + 1, vn_max + 1, lj + 1, jp_max + 1));
    }

    for (vs, ve, js, je) in segments {
        let gap_vn: Vec<usize> = vn_content_rows
            .iter()
            .copied()
            .filter(|r| *r >= vs && *r < ve && !matched_vn.contains(r))
            .collect();
        let gap_jp: Vec<usize> = jp_content_rows
            .iter()
            .copied()
            .filter(|r| *r >= js && *r < je && !matched_jp.contains(r))
            .collect();
        // Ghép tuần tự — dòng dư ở phía VN (chèn mới) hoặc phía JP (đã xóa) tự nhiên không được
        // ghép, giữ đúng ý nghĩa "dòng mới dùng nội dung VN" / "dòng JP bị xóa không dùng tới".
        for (&vn_r0, &jp_r0) in gap_vn.iter().zip(gap_jp.iter()) {
            result.insert(vn_r0 + 1, jp_r0 + 1);
        }
    }

    result
}

/// Trích công thức và style của ô cột A đầu tiên có formula trong vùng nội dung từ sheet JP.
/// Trả về (formula_text, xf_style_index, first_formula_row1).
/// Nếu không tìm thấy: formula mặc định, style 0, content_start_row1 làm fallback.
pub(crate) fn extract_jp_col_a_info(
    jp_sheet_xml: &str,
    content_start_row1: usize,
) -> (String, usize, usize) {
    let default_formula =
        "MAX($A$2:OFFSET(INDIRECT(ADDRESS(ROW(),COLUMN())),-1,0))+1".to_string();
    let Ok(doc) = roxmltree::Document::parse(jp_sheet_xml) else {
        return (default_formula, 0, content_start_row1);
    };
    for row in doc.descendants().filter(|n| n.tag_name().name() == "row") {
        let row1 = row
            .attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        if row1 < content_start_row1 {
            continue;
        }
        for cell in row.children().filter(|n| n.tag_name().name() == "c") {
            let (r0, c0) = match cell.attribute("r").and_then(parse_cell_ref) {
                Some(v) => v,
                None => continue,
            };
            if r0 + 1 != row1 || c0 != 0 {
                continue;
            }
            // Chỉ lấy ô CÓ công thức (<f>) — bỏ qua ô text "STT" hay số được tính sẵn
            if !cell.children().any(|n| n.tag_name().name() == "f") {
                continue;
            }
            let style = cell
                .attribute("s")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let formula = cell
                .children()
                .find(|n| n.tag_name().name() == "f")
                .and_then(|f| f.text())
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .unwrap_or(default_formula.clone());
            return (formula, style, row1);
        }
    }
    (default_formula, 0, content_start_row1)
}

/// Clone toàn bộ nội dung sheet VN vào sheet JP:
/// - JP được giữ làm khung (drawing, page setup, cols, sheetView, relationships) → shapes không mất
/// - Chỉ thay thế `<sheetData>` và `<mergeCells>` của JP bằng nội dung từ VN
/// - Hàng < `formula_start_row1` (header + cột tiêu đề): clone VN nguyên vẹn kể cả cột A
/// - Hàng >= `formula_start_row1` (dòng dữ liệu): remap style, inline string; nếu
///   `use_col_a_formula` = true thì thay cột A bằng công thức JP (`MAX($A$2:OFFSET(...))+1`) —
///   đặt `false` cho sheet không có cột STT tự đánh số (vd "変更履歴"), khi đó cột A được xử lý
///   như mọi cột khác bên dưới. Riêng từng ô (trừ cột A khi `use_col_a_formula` = true): nếu ô đó
///   KHÔNG có trong `vn_changed_positions` (nghĩa là chữ đen VÀ không strikethrough — xem
///   `find_changed_style_cells_xlsx`) VÀ JP đã có ô tại đúng vị trí đó, GIỮ NGUYÊN ô JP thay vì
///   ghi đè bằng VN (xem yêu cầu tại `c234_sync_service`).
///
/// Kết quả: số dòng / nội dung / style khớp VN, cột A là công thức JP, shapes header được giữ,
/// ô không đổi ở VN giữ nguyên bản JP.
///
/// `vn_to_jp_row` — ánh xạ (vn_row1 → jp_row1) cho trường hợp VN chèn/xóa dòng khiến vị trí lệch
/// nhau (xem `align_vn_jp_row_map`). Khi tra JP để giữ nguyên ô "không đổi", dòng VN tại `vn_row1`
/// sẽ tra JP tại `vn_to_jp_row[vn_row1]` (nếu có) thay vì trực tiếp `vn_row1` — nhờ đó ô JP gốc
/// được giữ đúng dòng logic dù số dòng vật lý đã lệch. VN row không map được (dòng hoàn toàn mới,
/// không khớp neo với JP) tự nhiên rơi về lookup miss → dùng nội dung VN, đúng ý nghĩa "dòng mới".
/// Truyền `None` để dùng ánh xạ 1:1 (vn_row = jp_row) như trước (không có chèn/xóa dòng).
///
/// `border_ext` — tích lũy các `<border>`/`<xf>` union JP+VN cần thêm vào styles.xml khi giữ nội
/// dung JP cho ô "không đổi" mà style VN áp lên không có đủ cạnh border JP vốn có (xem
/// `BorderUnionExtender`). Cột A (STT) cũng được sửa tương tự: dùng ĐÚNG style JP tại dòng đó
/// (qua `jp_cell_lookup`) thay vì 1 style canonical áp chung cho mọi dòng như trước — JP gốc có
/// thể dùng nhiều style/border khác nhau theo từng nhóm dòng ở cột A.
///
/// `jp_plain_ssi` — dùng để so sánh NỘI DUNG (không chỉ style) giữa ô VN và ô JP tại vị trí đã
/// align: nếu ô VN không được đánh dấu "đã thay đổi" (không đỏ/gạch bỏ) NHƯNG nội dung text thực
/// tế khác JP, vẫn coi là đã thay đổi (dùng VN) — bắt các trường hợp TL gõ nội dung mới mà quên tô
/// đỏ theo quy ước. `vn_fully_struck_positions` — ô VN bị gạch bỏ toàn bộ + màu không đen (coi
/// như "đã xóa nội dung", xem `find_fully_struck_colored_cells_xlsx`) LOẠI KHỎI so sánh nội dung
/// này — nếu không, phần chữ trước khi bị gạch (khác JP) sẽ bị hiểu nhầm thành "thay đổi" và ghi
/// đè lại đúng nội dung đáng lẽ phải xóa.
pub(crate) fn clone_vn_sheet_for_jp(
    vn_sheet_xml: &str,
    jp_sheet_xml: &str,
    jp_col_a_formula: &str,
    jp_col_a_style: usize,
    formula_start_row1: usize,
    xf_remap: &[usize],
    vn_plain_ssi: &HashMap<usize, String>,
    vn_rich_ssi_raw: &HashMap<usize, String>,
    jp_plain_ssi: &HashMap<usize, String>,
    vn_changed_positions: Option<&HashSet<(usize, usize)>>,
    vn_fully_struck_positions: Option<&HashSet<(usize, usize)>>,
    use_col_a_formula: bool,
    jp_preserved_header_cells: Option<&HashSet<(usize, usize)>>,
    vn_to_jp_row: Option<&HashMap<usize, usize>>,
    border_ext: &mut BorderUnionExtender,
) -> String {
    // ── 1. Xây sheetData mới từ VN ────────────────────────────────────────────
    let Ok(vn_doc) = roxmltree::Document::parse(vn_sheet_xml) else {
        return jp_sheet_xml.to_string();
    };
    let Some(vn_sd) = vn_doc.descendants().find(|n| n.tag_name().name() == "sheetData") else {
        return jp_sheet_xml.to_string();
    };

    // Lookup ô JP theo (row1, col0) — dùng để giữ nguyên ô JP tại các vị trí VN không đánh
    // dấu chỉnh sửa (chữ đen). `jp_text_lookup` — nội dung text thuần song song, dùng để so sánh
    // với VN (xem doc phía trên) khi quyết định có thực sự giữ JP hay không.
    let mut jp_cell_lookup: HashMap<(usize, usize), &str> = HashMap::new();
    let mut jp_text_lookup: HashMap<(usize, usize), String> = HashMap::new();
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
                for cell in row.children().filter(|n| n.tag_name().name() == "c") {
                    if let Some((_, col0)) = cell.attribute("r").and_then(parse_cell_ref) {
                        jp_cell_lookup.insert((jp_row1, col0), &jp_sheet_xml[cell.range()]);
                        if let Some(text) = extract_cell_plain_text(cell, jp_plain_ssi) {
                            jp_text_lookup.insert((jp_row1, col0), text);
                        }
                    }
                }
            }
        }
    }

    // Phát hiện "dòng header nhóm": dòng trong vùng nội dung có merge bắt đầu từ cột A.
    // Ví dụ: <mergeCell ref="A16:M16"/> → row 16 là dòng tiêu đề nhóm, không đánh số thứ tự.
    let group_header_rows: HashSet<usize> = {
        let mut set = HashSet::new();
        if let Some(mc) = vn_doc.descendants().find(|n| n.tag_name().name() == "mergeCells") {
            for merge in mc.children().filter(|n| n.tag_name().name() == "mergeCell") {
                if let Some(ref_str) = merge.attribute("ref") {
                    if let Some((start, _)) = ref_str.split_once(':') {
                        if let Some((r0, c0)) = parse_cell_ref(start) {
                            if c0 == 0 {
                                set.insert(r0 + 1); // row1
                            }
                        }
                    }
                }
            }
        }
        set
    };

    let mut new_sd = String::from("<sheetData>");
    for row_node in vn_sd.children().filter(|n| n.tag_name().name() == "row") {
        let row1 = row_node
            .attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        if row1 == 0 {
            continue;
        }

        if row1 < formula_start_row1 || group_header_rows.contains(&row1) {
            // Header row (kể cả "STT") hoặc dòng tiêu đề nhóm (merge từ cột A) — clone kể cả cột A.
            // Nếu có `jp_preserved_header_cells`, ô nào nằm trong set đó sẽ giữ nguyên bản JP.
            let has_preserved = jp_preserved_header_cells
                .map(|s| row_node.children().filter(|c| c.tag_name().name() == "c").any(|c| {
                    c.attribute("r")
                        .and_then(parse_cell_ref)
                        .map(|(_, col0)| s.contains(&(row1, col0)))
                        .unwrap_or(false)
                }))
                .unwrap_or(false);
            if has_preserved {
                let preserved = jp_preserved_header_cells.unwrap();
                let mut row_attrs = String::new();
                for a in row_node.attributes() {
                    match a.name() {
                        "r" => {}
                        "s" => {
                            let remapped = a.value().parse::<usize>().ok()
                                .and_then(|o| xf_remap.get(o).copied());
                            match remapped {
                                Some(v) => row_attrs.push_str(&format!(" s=\"{v}\"")),
                                None => row_attrs.push_str(&format!(" s=\"{}\"", a.value())),
                            }
                        }
                        name => row_attrs.push_str(&format!(" {}=\"{}\"", name, xml_escape_attr(a.value()))),
                    }
                }
                let cells: String = row_node.children()
                    .filter(|c| c.tag_name().name() == "c")
                    .map(|c| {
                        let col0 = c.attribute("r").and_then(parse_cell_ref).map(|(_, col)| col);
                        if let Some(col0) = col0 {
                            if preserved.contains(&(row1, col0)) {
                                if let Some(jp_raw) = jp_cell_lookup.get(&(row1, col0)) {
                                    return (*jp_raw).to_string();
                                }
                            }
                        }
                        clone_vn_cell_xml(c, vn_sheet_xml, row1, xf_remap, vn_plain_ssi, vn_rich_ssi_raw)
                    })
                    .collect();
                new_sd.push_str(&format!(r#"<row r="{row1}"{row_attrs}>{cells}</row>"#));
            } else {
                new_sd.push_str(&clone_vn_row_xml(
                    row_node,
                    vn_sheet_xml,
                    row1,
                    xf_remap,
                    vn_plain_ssi,
                    vn_rich_ssi_raw,
                ));
            }
        } else {
            // Dòng JP tương ứng để tra "giữ nguyên nội dung" — `None` nghĩa là dòng VN này ĐÃ ĐƯỢC
            // XÁC NHẬN là dòng mới chèn (không khớp neo/gap-fill nào với JP), nên KHÔNG được tra
            // JP theo số dòng vật lý (ánh xạ 1:1 tình cờ trùng số sẽ lấy nhầm nội dung của 1 bản
            // ghi JP khác) — coi như ô JP ở dòng này hoàn toàn RỖNG, mọi cell dùng nội dung VN
            // (đúng yêu cầu: dòng chèn phải "trống trước khi merge", nội dung merge = VN).
            // Chỉ fallback về ánh xạ 1:1 (row1) khi `vn_to_jp_row` HOÀN TOÀN RỖNG — nghĩa là sheet
            // này không tìm được neo nào cả (không có thông tin gì để biết dòng nào mới chèn),
            // giữ hành vi cũ để an toàn.
            let jp_lookup_row1: Option<usize> = match vn_to_jp_row {
                Some(m) if !m.is_empty() => m.get(&row1).copied(),
                _ => Some(row1),
            };

            // Dòng dữ liệu — bỏ cột A VN, thêm công thức JP (trừ khi `use_col_a_formula = false`,
            // vd sheet "変更履歴" không có cột STT tự đánh số — cột A khi đó xử lý như cột thường).
            // Style dùng ĐÚNG style JP tại dòng aligned (qua `jp_cell_lookup`) — JP gốc có thể
            // dùng nhiều style/border khác nhau theo từng nhóm dòng ở cột A, không phải 1 style
            // canonical duy nhất; chỉ fallback về `jp_col_a_style` khi JP không có ô tại dòng đó
            // (dòng VN hoàn toàn mới, không khớp neo với JP).
            let col_a_cell = if use_col_a_formula {
                let col_a_style = jp_lookup_row1
                    .and_then(|r| jp_cell_lookup.get(&(r, 0)))
                    .and_then(|raw| extract_raw_cell_style(raw))
                    .unwrap_or(jp_col_a_style);
                format!(
                    r#"<c r="A{row1}" s="{col_a_style}"><f ca="1">{}</f></c>"#,
                    xml_escape(jp_col_a_formula)
                )
            } else {
                String::new()
            };

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
            let other_cells: String = row_node
                .children()
                .filter(|c| c.tag_name().name() == "c")
                .filter_map(|c| {
                    let col0 = c.attribute("r").and_then(parse_cell_ref).map(|(_, col)| col)?;
                    if col0 == 0 && use_col_a_formula {
                        return None; // cột A đã xử lý riêng ở trên (công thức JP)
                    }
                    // VN không coi là đã thay đổi (chữ đen, không strikethrough) tại ô này →
                    // giữ nguyên NỘI DUNG ô JP cùng vị trí nếu có, nhưng dùng STYLE từ VN (đã
                    // remap) để bảo toàn border/fill mà VN có thể đã thêm — HỢP NHẤT với border
                    // JP gốc (không làm mất cạnh JP vốn có nếu VN không ghi đè, xem
                    // `BorderUnionExtender`) thay vì ghi đè toàn bộ.
                    let is_edited_by_style = vn_changed_positions
                        .map(|m| m.contains(&(vn_row0, col0)))
                        .unwrap_or(true);
                    // Dự phòng cho trường hợp TL gõ nội dung mới nhưng quên tô đỏ theo quy ước:
                    // nếu nội dung text thực tế của VN khác JP tại vị trí đã align, vẫn coi là đã
                    // thay đổi (dùng VN) — trừ ô đã bị gạch bỏ toàn bộ + màu không đen (coi như
                    // "xóa nội dung", xem `find_fully_struck_colored_cells_xlsx`): phần chữ trước
                    // khi gạch chắc chắn khác JP nhưng KHÔNG được hiểu thành "thay đổi" ở đây, nếu
                    // không nội dung đáng lẽ phải xóa sẽ bị ghi đè lại.
                    let is_fully_struck = vn_fully_struck_positions
                        .map(|m| m.contains(&(vn_row0, col0)))
                        .unwrap_or(false);
                    let content_differs = !is_fully_struck
                        && extract_cell_plain_text(c, vn_plain_ssi)
                            .map(|vn_text| {
                                let vn_text = vn_text.trim();
                                if vn_text.is_empty() {
                                    return false;
                                }
                                let jp_text = jp_lookup_row1
                                    .and_then(|r| jp_text_lookup.get(&(r, col0)))
                                    .map(|s| s.trim())
                                    .unwrap_or("");
                                vn_text != jp_text
                            })
                            .unwrap_or(false);
                    let is_edited = is_edited_by_style || content_differs;
                    // `jp_lookup_row1` là `None` ⇒ dòng VN này đã được xác nhận là dòng MỚI CHÈN
                    // (không khớp neo/gap-fill với JP) — bỏ qua hoàn toàn việc tra JP, coi như ô
                    // JP ở đây RỖNG, luôn dùng nội dung VN (rơi thẳng xuống `clone_vn_cell_xml`
                    // bên dưới) — đúng yêu cầu "dòng chèn phải trống trước khi merge".
                    if !is_edited {
                        if let Some(jp_raw) =
                            jp_lookup_row1.and_then(|r| jp_cell_lookup.get(&(r, col0)))
                        {
                            let vn_s = c.attribute("s")
                                .and_then(|s| s.parse::<usize>().ok())
                                .and_then(|o| xf_remap.get(o).copied());
                            // `jp_raw` mang `r=` gốc theo DÒNG JP (jp_lookup_row1) — khi
                            // `vn_to_jp_row` bù lệch dòng, dòng đó có thể KHÁC `row1` hiện tại;
                            // phải retarget lại `r=` khớp `row1` trước khi ghép vào, nếu không
                            // `<c r="D19">` lọt vào `<row r="20">` là cấu trúc sai OOXML — Excel
                            // coi file hỏng, tự "sửa" bằng cách xóa dòng/ô vi phạm.
                            let retargeted = if jp_lookup_row1 != Some(row1) {
                                retarget_raw_cell_row(jp_raw, row1)
                            } else {
                                (*jp_raw).to_string()
                            };
                            return Some(match vn_s {
                                Some(new_s) => {
                                    let jp_xf = extract_raw_cell_style(jp_raw).unwrap_or(0);
                                    let resolved_s = border_ext.resolve_style_for_kept_cell(jp_xf, new_s);
                                    restyle_raw_cell_xml(&retargeted, resolved_s)
                                }
                                None => retargeted,
                            });
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

            new_sd.push_str(&format!(
                r#"<row r="{row1}"{row_attrs}>{col_a_cell}{other_cells}</row>"#
            ));
        }
    }
    new_sd.push_str("</sheetData>");

    // ── 2. Trích mergeCells của VN ────────────────────────────────────────────
    let vn_merge_xml = vn_doc
        .descendants()
        .find(|n| n.tag_name().name() == "mergeCells")
        .map(|n| vn_sheet_xml[n.range()].to_string())
        .unwrap_or_default();

    // ── 3. Surgery trên JP sheet XML (giữ drawing, cols, sheetView, ...) ──────
    let Ok(jp_doc) = roxmltree::Document::parse(jp_sheet_xml) else {
        return jp_sheet_xml.to_string();
    };
    let Some(jp_sd) = jp_doc.descendants().find(|n| n.tag_name().name() == "sheetData") else {
        return jp_sheet_xml.to_string();
    };

    let mut edits: Vec<SurgeryEdit> = Vec::new();

    // Thay thế sheetData của JP bằng nội dung VN
    let jp_sd_end = jp_sd.range().end;
    edits.push(SurgeryEdit {
        start: jp_sd.range().start,
        end: jp_sd.range().end,
        replacement: new_sd,
    });

    // Xử lý mergeCells
    let jp_merge = jp_doc
        .descendants()
        .find(|n| n.tag_name().name() == "mergeCells");
    match (jp_merge, vn_merge_xml.is_empty()) {
        (Some(jm), false) => {
            // Thay mergeCells JP bằng VN
            edits.push(SurgeryEdit {
                start: jm.range().start,
                end: jm.range().end,
                replacement: vn_merge_xml,
            });
        }
        (Some(jm), true) => {
            // VN không có merge — xóa của JP
            edits.push(SurgeryEdit {
                start: jm.range().start,
                end: jm.range().end,
                replacement: String::new(),
            });
        }
        (None, false) => {
            // JP không có merge nhưng VN có — chèn sau sheetData
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
