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

    // Tìm fontId có font màu đỏ trong styles.xml
    let red_font_ids: HashSet<usize> = read_zip_entry(&mut archive, "xl/styles.xml")
        .map(|xml| parse_red_font_ids(&xml))
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
        parse_shared_strings_rich_info(&sst_xml)
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
            let cells =
                find_red_cells_in_sheet(&sheet_xml, &red_xf_indices, &red_ssi, &all_rich_ssi);
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

/// Tìm tất cả ô có font đỏ trong một sheet XML. Rich-text run (nếu có) LUÔN được xét TRƯỚC
/// style cấp-cell — đúng theo cách Excel hiển thị (run đè lên font mặc định của cell), tránh
/// hiểu nhầm 1 cell có base style đỏ nhưng nội dung rich-text thật lại không đỏ (hoặc ngược lại).
fn find_red_cells_in_sheet(
    sheet_xml: &str,
    red_xf: &HashSet<usize>,
    red_ssi: &HashSet<usize>,
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
                    if runs.iter().any(|r| has_red_font_run(r)) {
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

// ── Màu "chỉnh sửa" (edit) = ĐỎ hoặc XANH ─────────────────────────────────────
// Tài liệu này dùng CẢ đỏ (FFFF0000) LẪN xanh (FF0070C0) để đánh dấu nội dung mới/đã sửa cần phản
// ánh sang JP. Vì vậy khâu PHẢN ÁNH (find_red_cells_with_style_xlsx → apply_changes) coi cả hai màu
// là "edit". Ngược lại, khâu DỌN DẸP (transform_rich_runs) CHỈ tô đen màu ĐỎ (hoàn tất bản cũ),
// GIỮ NGUYÊN màu xanh — xem ghi chú tại 2 hàm đó.

fn is_edit_color(argb: &str) -> bool {
    is_argb_red(argb) || is_argb_blue(argb)
}

/// Run `<r>` có font màu edit (đỏ hoặc xanh) trong `<rPr><color rgb=".."/>`.
fn has_edit_font_run(run_node: &roxmltree::Node) -> bool {
    run_node.descendants().any(|child| {
        child.tag_name().name() == "color"
            && child.attribute("rgb").map(is_edit_color).unwrap_or(false)
    })
}

/// Tập fontId có màu edit (đỏ ∪ xanh).
fn parse_edit_font_ids(styles_xml: &str) -> HashSet<usize> {
    let mut ids = parse_red_font_ids(styles_xml);
    ids.extend(parse_blue_font_ids(styles_xml));
    ids
}

/// Parse styles.xml → tập fontId có màu xanh (blue). Dùng chung bởi `parse_edit_font_ids` (đỏ ∪
/// xanh) ở trên VÀ bởi `super::c238_sync_service` (nhận diện header nhóm mới màu xanh).
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

/// Giống `parse_shared_strings_rich_info` nhưng bắt cả run ĐỎ lẫn XANH: trả về
/// `(map ssi→raw runs của si có ÍT NHẤT 1 run màu edit, tập MỌI ssi rich-text)`.
fn parse_shared_strings_edit_info(sst_xml: &str) -> (HashMap<usize, String>, HashSet<usize>) {
    let mut edit_rich = HashMap::new();
    let mut all_rich = HashSet::new();
    let Ok(doc) = roxmltree::Document::parse(sst_xml) else {
        return (edit_rich, all_rich);
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
                if runs.iter().any(|r| has_edit_font_run(r)) {
                    if let (Some(first), Some(last)) = (runs.first(), runs.last()) {
                        edit_rich.insert(
                            si_idx,
                            sst_xml[first.range().start..last.range().end].to_string(),
                        );
                    }
                }
            }
            si_idx += 1;
        }
    }
    (edit_rich, all_rich)
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
        parse_shared_strings_rich_info(&sst_xml).1
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
const CHANGE_HISTORY_SHEET_NAME: &str = "変更履歴";
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

/// Vùng nội dung hợp lệ của 1 sheet theo loại tài liệu đã nhận diện — dispatch tới đúng method xử
/// lý riêng theo loại tài liệu ở trên. `None` nếu không nhận diện được loại tài liệu
/// (`DocType::Unknown`) ⇒ không giới hạn, giữ hành vi cũ (quét toàn sheet).
fn content_bounds_for(doc_type: DocType, sheet_name: &str) -> Option<ContentBounds> {
    match doc_type {
        DocType::C232 => Some(super::c232_sync_service::content_bounds(sheet_name)),
        DocType::C233 => Some(super::c233_sync_service::content_bounds(sheet_name)),
        DocType::C234 => Some(super::c234_sync_service::content_bounds(sheet_name)),
        DocType::C235 => Some(super::c235_sync_service::content_bounds(sheet_name)),
        DocType::C236 => Some(super::c236_sync_service::content_bounds(sheet_name)),
        DocType::C238 => Some(super::c238_sync_service::content_bounds(sheet_name)),
        DocType::Unknown => None,
    }
}

/// Bỏ vùng cột ngoài `bounds` khỏi 1 dòng (dùng khi quét cả dòng tìm ô neo canh dòng) — không đổi
/// gì nếu không có `bounds` (loại tài liệu không nhận diện được).
fn bounded_row<'a>(row: &'a [String], bounds: Option<ContentBounds>) -> &'a [String] {
    match bounds {
        Some(b) => &row[..row.len().min(b.last_col0 + 1)],
        None => row,
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

/// Như `filter_cells_by_bounds` nhưng cho kết quả có kèm style gốc (`find_red_cells_with_style_xlsx`).
fn filter_styled_cells_by_bounds(
    mut cells: HashMap<String, HashMap<(usize, usize), CellStyleSource>>,
    doc_type: DocType,
) -> HashMap<String, HashMap<(usize, usize), CellStyleSource>> {
    cells.retain(|sheet_name, map| {
        if let Some(bounds) = content_bounds_for(doc_type, sheet_name) {
            map.retain(|&(r, c), _| bounds.contains(r, c));
        }
        !map.is_empty()
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
    if super::c238_sync_service::is_screen_interface_doc(vn_path)
        || super::c238_sync_service::is_screen_interface_doc(jp_path)
    {
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
    /// XML gốc nguyên văn của `<a:p>...</a:p>` — dùng để copy y nguyên định dạng khi ghi sang JP.
    raw_xml: String,
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
                    raw_xml: drawing_xml[p.range()].to_string(),
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
    let red_font_ids = parse_red_font_ids(&styles_xml);
    let red_xf = parse_red_xf_indices(&styles_xml, &red_font_ids);
    let strike_xf = parse_strike_xf_indices(&styles_xml);
    let font_infos = parse_font_infos(&styles_xml);
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
fn xml_escape_attr(s: &str) -> String {
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

/// Chèn các đoạn văn shape mới (nguyên xi XML gốc từ VN) vào đúng shape (khớp theo TÊN) trong 1
/// file drawing JP, ngay trước `</xdr:txBody>` của shape đó. `paragraphs`: danh sách
/// `(tên shape, XML gốc của <a:p>)`. Trả về `(new_xml, applied_count, skipped_count)` — skipped
/// khi không tìm thấy shape cùng tên trong JP (vd shape chỉ có trong VN, chưa từng có ở JP).
fn inject_shape_paragraphs(
    drawing_xml: &str,
    paragraphs: &[(String, String)],
) -> (String, usize, usize) {
    if paragraphs.is_empty() {
        return (drawing_xml.to_string(), 0, 0);
    }

    let doc = match roxmltree::Document::parse(drawing_xml) {
        Ok(d) => d,
        Err(_) => return (drawing_xml.to_string(), 0, paragraphs.len()),
    };

    // Tên shape → vị trí byte ngay trước `</xdr:txBody>` (tìm bằng cách quét ngược '<' gần nhất
    // trước điểm kết thúc range của node txBody — tránh phải biết trước độ dài chuỗi đóng thẻ).
    let mut close_pos_by_name: HashMap<String, usize> = HashMap::new();
    for sp in doc.descendants().filter(|n| n.tag_name().name() == "sp") {
        let Some(name) = sp
            .descendants()
            .find(|n| n.tag_name().name() == "cNvPr")
            .and_then(|n| n.attribute("name"))
        else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let Some(tx_body) = sp.children().find(|c| c.tag_name().name() == "txBody") else {
            continue;
        };
        if let Some(open_lt) = drawing_xml[..tx_body.range().end].rfind('<') {
            close_pos_by_name.entry(name.to_string()).or_insert(open_lt);
        }
    }

    // Gom các đoạn văn cùng đích chèn lại rồi nối 1 lần (giữ đúng thứ tự), tránh việc chèn nhiều
    // lần tại CÙNG 1 vị trí làm đảo ngược thứ tự — xem cách xử lý tương tự ở `pos_inserts`
    // trong `inject_cells_into_sheet_xml`.
    let mut pos_texts: HashMap<usize, Vec<String>> = HashMap::new();
    let mut applied = 0usize;
    let mut skipped = 0usize;
    for (name, raw_p) in paragraphs {
        if let Some(&pos) = close_pos_by_name.get(name) {
            pos_texts.entry(pos).or_default().push(raw_p.clone());
            applied += 1;
        } else {
            skipped += 1;
        }
    }

    let edits: Vec<SurgeryEdit> = pos_texts
        .into_iter()
        .map(|(pos, texts)| SurgeryEdit {
            start: pos,
            end: pos,
            replacement: texts.join(""),
        })
        .collect();

    (apply_surgery(drawing_xml, edits), applied, skipped)
}

// ─────────────────────────────────────────────────────────────────────────────
// Sheet JP có hậu tố "(DEL)" trong tên: sheet này sắp bị xóa/thay thế, KHÔNG cần dọn dẹp
// strikethrough hay phản ánh nội dung VN mới như luồng thông thường — chỉ cần bỏ màu TOÀN BỘ
// chữ trong sheet về đen (giữ nguyên mọi định dạng khác: bold, gạch bỏ, border, fill...).
// ─────────────────────────────────────────────────────────────────────────────

/// Kiểm tra tên sheet có hậu tố "(DEL)" hay không (cho phép khoảng trắng trước dấu ngoặc,
/// không phân biệt hoa/thường).
fn is_del_sheet_name(name: &str) -> bool {
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

/// Nguồn style của 1 ô đỏ VN — dùng để tái tạo lại đúng định dạng khi ghi sang JP.
#[derive(Clone)]
enum CellStyleSource {
    /// Ô có rich-text run (nhiều `<r>`) — copy y nguyên XML gốc của các run này sang JP.
    Rich(String),
    /// Ô dùng font cấp-cell (không có rich-text run) — style lấy từ `<font>` trong styles.xml.
    Uniform {
        color: String,
        bold: bool,
        italic: bool,
        strike: bool,
    },
}

/// Parse styles.xml → FontInfo cho từng font, theo đúng thứ tự trong `<fonts>`.
fn parse_font_infos(styles_xml: &str) -> Vec<FontInfo> {
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
                        info.color = child.attribute("rgb").map(|s| s.to_string());
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

/// Giống `parse_red_xf_indices` nhưng trả về font_id tương ứng của từng xf đỏ
/// (thay vì chỉ đánh dấu có/không), để tra cứu FontInfo đầy đủ (màu/bold/italic/strike).
fn parse_red_xf_font_ids(styles_xml: &str, red_font_ids: &HashSet<usize>) -> HashMap<usize, usize> {
    let mut result = HashMap::new();
    if red_font_ids.is_empty() {
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
            let apply_font = node.attribute("applyFont").map_or(false, |v| v == "1" || v == "true");
            let inherited_font = node
                .attribute("xfId")
                .and_then(|s| s.parse::<usize>().ok())
                .and_then(|xfid| style_xf_fonts.get(xfid).copied());

            if apply_font && red_font_ids.contains(&font_id) {
                result.insert(xf_idx, font_id);
            } else if let Some(f) = inherited_font {
                if red_font_ids.contains(&f) {
                    result.insert(xf_idx, f);
                }
            }
            xf_idx += 1;
        }
    }
    result
}

/// Giống `parse_red_shared_strings` nhưng lấy luôn XML thô của các run bên trong
/// (thay vì chỉ đánh dấu index), để copy y nguyên khi ghi sang JP.
/// Trả về (ssi có run đỏ → raw XML của các run, tập hợp MỌI ssi có rich-text run bất kể màu).
/// Tập thứ 2 dùng để biết 1 cell dùng shared string rich-text hay không — rich-text LUÔN ưu
/// tiên hơn style cấp-cell, kể cả khi bản thân rich-text đó không có run nào đỏ.
pub(crate) fn parse_shared_strings_rich_info(sst_xml: &str) -> (HashMap<usize, String>, HashSet<usize>) {
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
                if runs.iter().any(|r| has_red_font_run(r)) {
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

/// Chuẩn hóa 1 mã màu (6 hoặc 8 hex) về dạng ARGB 8-hex dùng cho `<color rgb="...">`.
fn normalize_argb(color: &str) -> String {
    let c = color.trim_start_matches('#');
    if c.len() == 6 {
        format!("FF{c}")
    } else {
        c.to_string()
    }
}

/// Quét toàn bộ file VN, với mỗi ô đỏ trả về `CellStyleSource` tương ứng — dùng riêng cho
/// bước ghi (apply), tách biệt với `find_red_cells_xlsx` (dùng cho phân tích/đếm) để không
/// ảnh hưởng logic phát hiện đã hoạt động ổn định.
fn find_red_cells_with_style_xlsx(path: &str) -> HashMap<String, HashMap<(usize, usize), CellStyleSource>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return HashMap::new(),
    };

    let styles_xml = read_zip_entry(&mut archive, "xl/styles.xml").unwrap_or_default();
    // Bắt cả ô ĐỎ lẫn XANH (edit color) để phản ánh sang JP — xem `is_edit_color`.
    let edit_font_ids = parse_edit_font_ids(&styles_xml);
    let font_infos = parse_font_infos(&styles_xml);
    let red_xf_font_ids = parse_red_xf_font_ids(&styles_xml, &edit_font_ids);

    let sst_xml = read_zip_entry(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (red_ssi_rich, all_rich_ssi) = if sst_xml.is_empty() {
        (HashMap::new(), HashSet::new())
    } else {
        parse_shared_strings_edit_info(&sst_xml)
    };

    // Lưu ý: KHÔNG early-return dù red_xf_font_ids/red_ssi_rich rỗng — ô inline string có thể
    // mang màu đỏ nhúng trực tiếp trong run (case 3 bên dưới), không qua styles.xml/sharedStrings.xml.

    let workbook_xml = match read_zip_entry(&mut archive, "xl/workbook.xml") {
        Some(s) => s,
        None => return HashMap::new(),
    };
    let rels_xml = match read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels") {
        Some(s) => s,
        None => return HashMap::new(),
    };
    let sheet_paths = resolve_sheet_xml_paths(&workbook_xml, &rels_xml);

    let mut result: HashMap<String, HashMap<(usize, usize), CellStyleSource>> = HashMap::new();
    for (name, xml_path) in sheet_paths {
        let Some(sheet_xml) = read_zip_entry(&mut archive, &xml_path) else {
            continue;
        };
        let Ok(doc) = roxmltree::Document::parse(&sheet_xml) else {
            continue;
        };
        let mut cells: HashMap<(usize, usize), CellStyleSource> = HashMap::new();

        for node in doc.descendants() {
            if node.tag_name().name() != "c" {
                continue;
            }
            let Some(pos) = node.attribute("r").and_then(parse_cell_ref) else {
                continue;
            };

            // Rich-text run (nếu có) LUÔN được ưu tiên hơn style cấp-cell — đúng theo cách Excel
            // hiển thị: định dạng trên từng run đè lên font mặc định của cell. Nếu không xử lý
            // trước, 1 cell có base style đỏ (rất phổ biến) nhưng nội dung thật là rich-text đan
            // xen (đỏ/đen/gạch bỏ) sẽ bị hiểu nhầm thành "toàn bộ ô 1 màu đỏ đồng nhất", làm mất
            // hẳn phần gạch bỏ khi ghi sang JP.
            let mut handled_by_rich = false;

            // 1. Shared string rich-text (t="s")
            if node.attribute("t") == Some("s") {
                if let Some(v) = node.children().find(|c| c.tag_name().name() == "v") {
                    if let Some(ssi) = v.text().and_then(|t| t.parse::<usize>().ok()) {
                        if all_rich_ssi.contains(&ssi) {
                            handled_by_rich = true;
                            if let Some(raw) = red_ssi_rich.get(&ssi) {
                                cells.insert(pos, CellStyleSource::Rich(raw.clone()));
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
                        if runs.iter().any(|r| has_edit_font_run(r)) {
                            if let (Some(first), Some(last)) = (runs.first(), runs.last()) {
                                let raw =
                                    sheet_xml[first.range().start..last.range().end].to_string();
                                cells.insert(pos, CellStyleSource::Rich(raw));
                            }
                        }
                    }
                }
            }

            if handled_by_rich {
                continue;
            }

            // 3. Style cấp-cell (chỉ áp dụng khi ô KHÔNG có rich-text run nào)
            if let Some(s_idx) = node.attribute("s").and_then(|s| s.parse::<usize>().ok()) {
                if let Some(font_id) = red_xf_font_ids.get(&s_idx) {
                    if let Some(info) = font_infos.get(*font_id) {
                        cells.insert(
                            pos,
                            CellStyleSource::Uniform {
                                color: info
                                    .color
                                    .clone()
                                    .unwrap_or_else(|| "FFFF0000".to_string()),
                                bold: info.bold,
                                italic: info.italic,
                                strike: info.strike,
                            },
                        );
                    }
                }
            }
        }

        if !cells.is_empty() {
            result.insert(name, cells);
        }
    }

    result
}

/// Tạo `<c>` mới cho ô cần ghi, giữ nguyên style gốc từ VN nếu có (`CellStyleSource`);
/// nếu không xác định được style (fallback), dùng lại kiểu đỏ đậm mặc định như cũ.
fn make_cell_with_style(
    cell_ref: &str,
    s_attr: &str,
    vn_text: &str,
    style: &Option<CellStyleSource>,
) -> String {
    match style {
        Some(CellStyleSource::Rich(raw)) => {
            let s_part = if !s_attr.is_empty() {
                format!(" s=\"{s_attr}\"")
            } else {
                String::new()
            };
            format!(r#"<c r="{cell_ref}"{s_part} t="inlineStr"><is>{raw}</is></c>"#)
        }
        Some(CellStyleSource::Uniform {
            color,
            bold,
            italic,
            strike,
        }) => {
            let s_part = if !s_attr.is_empty() {
                format!(" s=\"{s_attr}\"")
            } else {
                String::new()
            };
            let mut rpr = format!(r#"<color rgb="{}"/>"#, normalize_argb(color));
            if *bold {
                rpr.push_str("<b/>");
            }
            if *italic {
                rpr.push_str("<i/>");
            }
            if *strike {
                rpr.push_str("<strike/>");
            }
            format!(
                r#"<c r="{cell_ref}"{s_part} t="inlineStr"><is><r><rPr>{rpr}</rPr><t xml:space="preserve">{}</t></r></is></c>"#,
                xml_escape(vn_text)
            )
        }
        None => make_vn_inline_cell(cell_ref, s_attr, vn_text),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Áp dụng thay đổi: ghi VN text (đỏ) vào đúng vị trí ô trong file JP xlsx
// ─────────────────────────────────────────────────────────────────────────────

/// Độ dài tối thiểu (số ký tự) của nội dung JP để được xem là 1 điểm khớp hợp lệ khi tìm cột
/// đúng — tránh khớp nhầm với nội dung quá ngắn/chung (vd "）", số thứ tự...).
const MIN_COLUMN_MATCH_LEN: usize = 3;

/// Độ dài (số ký tự) chuỗi con CHUNG DÀI NHẤT giữa 2 chuỗi — dùng đo mức "liên quan" nội dung
/// JP↔VN mà không đòi hỏi chứa trọn (VN thường = JP + phần thêm, hoặc lệch dấu câu cuối).
fn common_substr_len(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    let mut prev = vec![0usize; b.len() + 1];
    let mut best = 0usize;
    for i in 1..=a.len() {
        let mut cur = vec![0usize; b.len() + 1];
        for j in 1..=b.len() {
            if a[i - 1] == b[j - 1] {
                cur[j] = prev[j - 1] + 1;
                best = best.max(cur[j]);
            }
        }
        prev = cur;
    }
    best
}

/// Xác định cột ĐÍCH trong JP để ghi ô VN. Mặc định giữ nguyên cột của VN; chỉ đổi sang cột khác
/// khi cột đó khớp nội dung vn_text DÀI HƠN HẲN cột gốc (xử lý lệch cột do JP thêm/thiếu cột).
/// Dùng chuỗi con chung dài nhất (không đòi hỏi chứa trọn) để cột gốc "dính" khi VN chỉ là JP +
/// phần thêm — tránh chuyển nhầm sang cột có mã ngắn (vd "MJDL020") tình cờ nằm trong vn_text.
/// Trả về `(cột đích 0-based, có bị điều chỉnh hay không)`.
fn resolve_target_col(
    jp_grid: &HashMap<String, Vec<Vec<String>>>,
    sheet: &str,
    row_0: usize,
    col_0: usize,
    vn_text: &str,
    last_col0: Option<usize>,
) -> (usize, bool) {
    let Some(row_vec) = jp_grid.get(sheet).and_then(|g| g.get(row_0)) else {
        return (col_0, false);
    };
    let vn = vn_text.trim();

    let same_score = row_vec
        .get(col_0)
        .map(|t| common_substr_len(t.trim(), vn))
        .unwrap_or(0);

    let search_len = last_col0.map_or(row_vec.len(), |c| row_vec.len().min(c + 1));
    let mut best_col = col_0;
    let mut best_score = same_score;
    for (c, text) in row_vec[..search_len].iter().enumerate() {
        if c == col_0 {
            continue;
        }
        let t = text.trim();
        if t.chars().count() < MIN_COLUMN_MATCH_LEN {
            continue;
        }
        let score = common_substr_len(t, vn);
        if score > best_score {
            best_score = score;
            best_col = c;
        }
    }

    if best_col != col_0 && best_score >= MIN_COLUMN_MATCH_LEN && best_score > same_score {
        (best_col, true)
    } else {
        (col_0, false)
    }
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
fn extract_all_shared_strings(sst_xml: &str) -> (HashMap<usize, String>, HashMap<usize, String>) {
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
struct StyleMergeResult {
    new_styles_xml: String,
    /// Chỉ số cellXfs GỐC của VN (0-based) → chỉ số MỚI trong styles.xml JP đã merge — dùng để
    /// remap thuộc tính `s=`/`style=` của mọi cell/row/col trong sheet VN được clone.
    xf_remap: Vec<usize>,
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

fn merge_vn_styles_into_jp(jp_styles_xml: &str, vn_styles_xml: &str) -> StyleMergeResult {
    let jp_doc = match roxmltree::Document::parse(jp_styles_xml) {
        Ok(d) => d,
        Err(_) => {
            return StyleMergeResult {
                new_styles_xml: jp_styles_xml.to_string(),
                xf_remap: Vec::new(),
            }
        }
    };
    let vn_doc = match roxmltree::Document::parse(vn_styles_xml) {
        Ok(d) => d,
        Err(_) => {
            return StyleMergeResult {
                new_styles_xml: jp_styles_xml.to_string(),
                xf_remap: Vec::new(),
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
    if let Some(vn_cxfs) = find_child(vn_root, "cellXfs") {
        for (i, n) in vn_cxfs.children().filter(|c| c.tag_name().name() == "xf").enumerate() {
            new_cell_xfs_raw.push_str(&remap_xf_element(
                n,
                vn_styles_xml,
                fonts_offset,
                fills_offset,
                borders_offset,
                cell_style_xfs_offset,
                true,
                &numfmt_remap,
            ));
            xf_remap.push(cell_xfs_offset + i);
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

    StyleMergeResult {
        new_styles_xml: apply_surgery(jp_styles_xml, edits),
        xf_remap,
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

    let mut indexed: Vec<(usize, &(String, String))> = entries.iter().enumerate().collect();
    indexed.sort_by_key(|(orig_idx, (name, _))| match vn_pos.get(name.as_str()) {
        Some(&p) => (0usize, p, *orig_idx),
        None => (1usize, 0usize, *orig_idx),
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

/// Bước 5-6 của pipeline "Áp dụng" (xem `apply_changes`): ghi VN text (in đỏ) vào đúng vị trí
/// tương ứng trong `jp_path` — vốn đã qua `sync_structure` (dọn dẹp/clone/đổi tên DEL/canh dòng)
/// — rồi lưu ra `output_path`. Nội dung VN được giữ nguyên (không dịch) và tô màu đỏ để reviewer
/// kiểm tra. `carried_cloned_names`: tên sheet đã được `sync_structure` clone trực tiếp từ VN
/// trước đó — BỎ QUA các sheet này (nội dung đã đúng 100% từ VN rồi, không ghi/dọn lại).
pub(crate) fn merge_content(
    vn_path: &str,
    jp_path: &str,
    output_path: &str,
    carried_cloned_names: &HashSet<String>,
) -> AppResult<ApplyResult> {
    let analysis = analyze(vn_path, jp_path)?;
    let doc_type = detect_doc_type(vn_path, jp_path);
    // Ô "edit" của VN = ĐỎ hoặc XANH (xem `is_edit_color`) kèm style gốc (màu/bold/italic/strike).
    let vn_styles = filter_styled_cells_by_bounds(find_red_cells_with_style_xlsx(vn_path), doc_type);
    let jp_grid_for_column_check = read_workbook_grid(jp_path)?;
    let vn_grid = read_workbook_grid(vn_path)?;

    // Group edit cells by sheet: sheet_name → Vec<(row_0, col_0, vn_text, style)>
    // `style` giữ lại màu chữ/bold/italic/strikethrough gốc của ô VN để tái tạo đúng khi ghi sang JP
    // (đỏ giữ đỏ, xanh giữ xanh). `col_0` là cột ĐÍCH trong JP — có thể khác cột VN nếu phát hiện
    // lệch cột cùng dòng (xem resolve_target_col): style vẫn tra theo cột GỐC của VN, chỉ vị trí đổi.
    let mut cells_by_sheet: HashMap<String, Vec<(usize, usize, String, Option<CellStyleSource>)>> =
        HashMap::new();
    let mut column_corrected_count = 0usize;
    for (sheet, cell_styles) in &vn_styles {
        for ((vn_row_0, vn_col_0), style) in cell_styles {
            let vn_text = vn_grid
                .get(sheet)
                .and_then(|g| g.get(*vn_row_0))
                .and_then(|row| row.get(*vn_col_0))
                .cloned()
                .unwrap_or_default();
            if vn_text.trim().is_empty() {
                continue;
            }
            let last_col0 = content_bounds_for(doc_type, sheet).map(|b| b.last_col0);
            let (target_col_0, corrected) = resolve_target_col(
                &jp_grid_for_column_check,
                sheet,
                *vn_row_0,
                *vn_col_0,
                &vn_text,
                last_col0,
            );
            if corrected {
                column_corrected_count += 1;
            }
            cells_by_sheet.entry(sheet.clone()).or_default().push((
                *vn_row_0,
                target_col_0,
                vn_text,
                Some(style.clone()),
            ));
        }
    }

    // Gom đoạn văn shape/textbox nổi có chữ đỏ từ VN theo sheet: sheet_name → Vec<(tên shape,
    // XML gốc của <a:p>)>. Đối ứng theo TÊN shape (không theo neo) — xem ghi chú đầu mục
    // "Xử lý textbox/shape nổi".
    let vn_shapes = filter_shapes_by_bounds(find_shapes_xlsx(vn_path), doc_type);
    let mut shape_paragraphs_by_sheet: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (sheet_name, shapes) in &vn_shapes {
        for shape in shapes {
            if shape.name.is_empty() {
                continue;
            }
            for p in &shape.paragraphs {
                if p.any_red {
                    shape_paragraphs_by_sheet
                        .entry(sheet_name.clone())
                        .or_default()
                        .push((shape.name.clone(), p.raw_xml.clone()));
                }
            }
        }
    }

    // Sheet chỉ có ở VN chưa có ở JP — bình thường đã = 0 ở bước này vì `sync_structure` đã clone
    // trước; chỉ khả thi nếu hàm này được gọi độc lập (jp_path không qua `sync_structure`).
    let sheets_to_clone: Vec<String> = analysis
        .sheet_compare
        .iter()
        .filter(|c| c.in_vn && !c.in_jp && !is_del_sheet_name(&c.name))
        .map(|c| c.name.clone())
        .collect();

    // Open JP file as ZIP
    let jp_file = File::open(jp_path)
        .map_err(|e| AppError::new(format!("Không mở được file JP: {e}")))?;
    let mut archive = zip::ZipArchive::new(jp_file)
        .map_err(|e| AppError::new(format!("File JP không phải ZIP hợp lệ: {e}")))?;

    // Dọn dẹp lại phòng khi hàm này được gọi độc lập trên 1 file JP chưa qua `sync_structure` —
    // đã dọn rồi thì đây là no-op (không còn strikethrough/chữ đỏ cũ để tìm thấy).
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

    // Resolve sheet name → XML path
    let workbook_xml = read_zip_entry(&mut archive, "xl/workbook.xml")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/workbook.xml trong file JP."))?;
    let rels_xml = read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels")
        .ok_or_else(|| AppError::new("Không tìm thấy xl/_rels/workbook.xml.rels."))?;
    let sheet_path_list = resolve_sheet_xml_paths(&workbook_xml, &rels_xml);
    let mut sheet_path_map: HashMap<String, String> = sheet_path_list.iter().cloned().collect();

    let cloned = clone_missing_sheets_from_vn(&mut archive, vn_path, &sheets_to_clone, &mut replaced)?;
    let mut newly_cloned_names: HashSet<String> =
        cloned.new_entries.iter().map(|(name, _)| name.clone()).collect();
    for (name, path) in &cloned.new_entries {
        sheet_path_map.insert(name.clone(), path.clone());
    }
    let cloned_sheet_count = cloned.new_entries.len();
    newly_cloned_names.extend(carried_cloned_names.iter().cloned());

    // Bước 2: dọn dẹp mọi sheet, sau đó phản ánh chữ đỏ VN lên trên (chỉ các sheet có ô đỏ).
    let mut applied_count = 0usize;
    let mut skipped_count = 0usize;
    let mut cleanup_skipped_count = 0usize;
    let mut shape_applied_count = 0usize;
    let mut shape_skipped_count = 0usize;
    let mut del_sheet_count = 0usize;
    let mut sheets_modified: Vec<String> = Vec::new();

    for (sheet_name, xml_path) in &sheet_path_map {
        let Some(original_xml) = read_current(&mut archive, &replaced, xml_path) else {
            if let Some(cells) = cells_by_sheet.get(sheet_name) {
                skipped_count += cells.len();
            }
            if let Some(paragraphs) = shape_paragraphs_by_sheet.get(sheet_name) {
                shape_skipped_count += paragraphs.len();
            }
            continue;
        };
        let bounds = content_bounds_for(doc_type, sheet_name);

        // Sheet vừa clone trực tiếp từ VN (xem `clone_missing_sheets_from_vn`): nội dung/định
        // dạng/shape đã đúng 100% từ VN ngay khi clone — KHÔNG chạy dọn dẹp hay ghi ô đỏ lại,
        // tránh vô tình tô đen nhầm ô đỏ mới hoặc ghi đè mất style vừa merge.
        if newly_cloned_names.contains(sheet_name) {
            if !sheets_modified.contains(sheet_name) {
                sheets_modified.push(sheet_name.clone());
            }
            continue;
        }

        // Sheet "(DEL)": chỉ bỏ màu chữ về đen, KHÔNG dọn strikethrough, KHÔNG ghi nội dung VN
        // mới, KHÔNG đụng shape — xem ghi chú đầu mục "Sheet JP có hậu tố (DEL)".
        if is_del_sheet_name(sheet_name) {
            del_sheet_count += 1;
            if let Some(cells) = cells_by_sheet.get(sheet_name) {
                skipped_count += cells.len();
            }
            if let Some(paragraphs) = shape_paragraphs_by_sheet.get(sheet_name) {
                shape_skipped_count += paragraphs.len();
            }
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

        // Xác định drawing (nếu có) TRƯỚC khi original_xml có thể bị move ở dưới.
        let drawing_path = resolve_drawing_path(&mut archive, &original_xml, xml_path);

        let (cleaned_xml, s_removed, r_blackened, c_skip) =
            cleanup_sheet_xml(&original_xml, &ctx, &rich_ssi, &plain_text, bounds);
        cleanup_skipped_count += c_skip;
        let cleaned = s_removed > 0 || r_blackened > 0;
        if cleaned {
            strike_removed_count += s_removed;
            red_blackened_count += r_blackened;
        }
        let mut current_xml = if cleaned { cleaned_xml } else { original_xml };
        let mut changed = cleaned;

        if let Some(cells) = cells_by_sheet.get(sheet_name) {
            let (new_xml, n_applied) = inject_cells_into_sheet_xml(&current_xml, cells);
            let n_skipped = cells.len() - n_applied;
            applied_count += n_applied;
            skipped_count += n_skipped;
            if n_applied > 0 {
                current_xml = new_xml;
                changed = true;
            }
        }

        if changed {
            replaced.insert(xml_path.clone(), current_xml.into_bytes());
            sheets_modified.push(sheet_name.clone());
        }

        // Dọn dẹp + phản ánh textbox/shape nổi (nếu sheet có drawing).
        let Some(drawing_path) = drawing_path else {
            continue;
        };
        let Some(drawing_xml) = read_zip_entry(&mut archive, &drawing_path) else {
            if let Some(paragraphs) = shape_paragraphs_by_sheet.get(sheet_name) {
                shape_skipped_count += paragraphs.len();
            }
            continue;
        };

        let (cleaned_drawing, d_removed, d_blackened) = cleanup_drawing_xml(&drawing_xml);
        let drawing_cleaned = d_removed > 0 || d_blackened > 0;
        if drawing_cleaned {
            strike_removed_count += d_removed;
            red_blackened_count += d_blackened;
        }
        let mut current_drawing = if drawing_cleaned {
            cleaned_drawing
        } else {
            drawing_xml
        };
        let mut drawing_changed = drawing_cleaned;

        if let Some(paragraphs) = shape_paragraphs_by_sheet.get(sheet_name) {
            let (new_drawing, n_applied, n_skipped) =
                inject_shape_paragraphs(&current_drawing, paragraphs);
            shape_applied_count += n_applied;
            shape_skipped_count += n_skipped;
            if n_applied > 0 {
                current_drawing = new_drawing;
                drawing_changed = true;
            }
        }

        if drawing_changed {
            replaced.insert(drawing_path, current_drawing.into_bytes());
            if !sheets_modified.contains(sheet_name) {
                sheets_modified.push(sheet_name.clone());
            }
        }
    }

    // Ô đỏ / đoạn shape VN trỏ tới sheet không tồn tại trong JP → bỏ qua, tính vào skipped_count.
    for (sheet_name, cells) in &cells_by_sheet {
        if !sheet_path_map.contains_key(sheet_name) {
            skipped_count += cells.len();
        }
    }
    for (sheet_name, paragraphs) in &shape_paragraphs_by_sheet {
        if !sheet_path_map.contains_key(sheet_name) {
            shape_skipped_count += paragraphs.len();
        }
    }

    // Sắp xếp lại thứ tự sheet JP cho khớp thứ tự sheet VN — nếu `sync_structure` đã sắp xếp
    // trước thì đây là no-op; vẫn giữ lại phòng khi hàm này được gọi độc lập.
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

    // Write output ZIP (copy JP + swap modified sheet/sharedStrings/drawing XMLs + sheet mới tạo)
    write_output_zip(&mut archive, &replaced, output_path)?;

    Ok(ApplyResult {
        output_path: output_path.to_string(),
        applied_count,
        skipped_count,
        sheets_modified,
        strike_removed_count,
        red_blackened_count,
        cleanup_skipped_count,
        column_corrected_count,
        shape_applied_count,
        shape_skipped_count,
        cloned_sheet_count,
        del_sheet_count,
        rows_inserted: 0,
    })
}

// Pipeline "Áp dụng" VN → JP KHÔNG còn nằm ở đây — mỗi `c23{2,3,4,5,6,8}_sync_service::apply_changes`
// (xem module doc từng file) tự chạy đủ 6 bước (chấp nhận lặp code giữa các loại tài liệu), chỉ
// gọi vào các hàm dùng chung bên dưới (`sync_structure`, `merge_content`, `analyze_row_alignment`,
// `insert_rows`, `merged_output_path`). Dispatch theo `detect_doc_type` nằm ở
// `services::vnjp_sync_service::apply_changes`.

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
    cells: &[(usize, usize, String, Option<CellStyleSource>)], // (row_0, col_0, vn_text, style)
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

    // Dedup theo (row_0, col_0) trước — nhiều ô đỏ VN có thể bị `resolve_target_col` tự sửa lệch
    // cột trỏ về CÙNG 1 vị trí đích JP (vd 2 cột VN khác nhau cùng khớp nội dung JP ở 1 cột). Nếu
    // không dedup, 2 cell trong `cells` cùng đích sẽ sinh 2 SurgeryEdit đè lên CÙNG 1 range, khiến
    // `apply_surgery` cắt ghép sai byte (ghi đè xong lại ghi đè tiếp lên offset đã lệch) — làm
    // hỏng cấu trúc XML, Excel báo "cần sửa chữa" khi mở file output dù `applied_count` vẫn đếm
    // đủ (không cell nào bị coi là lỗi). Giữ lại bản ghi SAU CÙNG (theo thứ tự trong `cells`).
    let mut dedup: HashMap<(usize, usize), (String, Option<CellStyleSource>)> = HashMap::new();
    for (row_0, col_0, vn_text, style) in cells {
        dedup.insert((*row_0, *col_0), (vn_text.clone(), style.clone()));
    }

    // Sort input cells so same-position inserts accumulate in col order
    let mut sorted_cells: Vec<(usize, usize, String, Option<CellStyleSource>)> = dedup
        .into_iter()
        .map(|((row_0, col_0), (vn_text, style))| (row_0, col_0, vn_text, style))
        .collect();
    sorted_cells.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut edits: Vec<SurgeryEdit> = Vec::new();
    // Accumulate same-position insertions to avoid collisions
    let mut pos_inserts: HashMap<usize, Vec<String>> = HashMap::new();
    // New rows: row_1 → [(col_0, vn_text, style)] — built separately to merge multi-cell rows
    let mut new_rows: HashMap<usize, Vec<(usize, String, Option<CellStyleSource>)>> = HashMap::new();

    let mut applied_count = 0usize;

    for (row_0, col_0, vn_text, style) in &sorted_cells {
        let row_1 = row_0 + 1;
        let cell_ref = format!("{}{}", col_index_to_letter(*col_0), row_1);

        if let Some((cell_range, s_attr)) = existing_cells.get(&(*row_0, *col_0)) {
            // Replace existing cell with VN text, giữ nguyên style gốc từ VN nếu có
            edits.push(SurgeryEdit {
                start: cell_range.start,
                end: cell_range.end,
                replacement: make_cell_with_style(&cell_ref, s_attr, vn_text, style),
            });
            applied_count += 1;
        } else if let Some((row_range, cells_in_row)) = row_data.get(&row_1) {
            // Insert into existing row
            let insert_pos =
                find_cell_insert_pos(sheet_xml, row_range, cells_in_row, *col_0);
            pos_inserts
                .entry(insert_pos)
                .or_default()
                .push(make_cell_with_style(&cell_ref, "", vn_text, style));
            applied_count += 1;
        } else {
            // Need new row
            new_rows
                .entry(row_1)
                .or_default()
                .push((*col_0, vn_text.clone(), style.clone()));
            applied_count += 1;
        }
    }

    // Build new-row insertions (one <row> per row_1, all cells sorted by col)
    if let Some(sd_range) = &sheet_data_range {
        let row_keys: Vec<usize> = row_data.keys().copied().collect();
        for (row_1, mut row_cells) in new_rows {
            row_cells.sort_by_key(|(col, _, _)| *col);
            let cells_xml: String = row_cells
                .iter()
                .map(|(col_0, vn_text, style)| {
                    let cref = format!("{}{}", col_index_to_letter(*col_0), row_1);
                    make_cell_with_style(&cref, "", vn_text, style)
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
///
/// `output_path` có thể TRÙNG với file đang mở trong `archive` (ghi đè in-place — xem pipeline
/// "Áp dụng" ở `c23X_sync_service::apply_changes`, chạy `sync_structure` → `insert_rows` →
/// `merge_content` liên tiếp trên CÙNG 1 file Temp). Vì vậy phải ĐỌC HẾT nội dung ZIP mới vào
/// buffer trong bộ nhớ trước, rồi mới `File::create(output_path)` để ghi đè — nếu tạo/truncate
/// file output ngay từ đầu (như trước đây) trong khi `archive` vẫn còn đang đọc từ CHÍNH file đó,
/// dữ liệu trên đĩa bị ghi đè giữa chừng, khiến các entry đọc sau (vd `[Content_Types].xml`) lấy
/// phải byte đã bị thay, gây lỗi "Invalid checksum".
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

    // Ghi thêm các part HOÀN TOÀN MỚI chưa tồn tại trong ZIP gốc (vd sheet vừa clone từ VN —
    // xem `clone_missing_sheets`) — vòng lặp trên chỉ xử lý entry đã có sẵn trong `archive`.
    let existing: HashSet<&str> = entry_names.iter().map(|s| s.as_str()).collect();
    for (name, bytes) in replaced {
        if existing.contains(name.as_str()) {
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

fn clone_vn_row_xml(
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

fn clone_vn_cell_xml(
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
