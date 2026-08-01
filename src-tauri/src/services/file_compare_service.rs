//! Service so sánh khác biệt giữa 2 file.
//!
//! - Text/Markdown: đọc `read_to_string` → diff theo dòng (`similar`).
//! - Word (`.docx`): zip mở `word/document.xml` → `roxmltree` gom các `<w:p>`
//!   (mỗi paragraph 1 dòng) → diff theo dòng.
//! - Excel (`.xlsx/.xls`): `calamine` đọc từng sheet → so sánh cell-by-cell.

use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};
use rust_xlsxwriter::{Color, Format, Workbook};
use similar::{capture_diff_slices, Algorithm, ChangeTag, DiffOp, TextDiff as SimilarTextDiff};

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::file_compare::{
    AxisMarker, CellDiff, CompareResult, DiffLine, ExcelDiff, SheetDiff, TextDiff,
};

/// Điểm vào: so sánh 2 file đường dẫn tuyệt đối. 2 file phải cùng loại.
pub fn compare(file_a: &str, file_b: &str) -> AppResult<CompareResult> {
    let kind_a = detect_kind(file_a)?;
    let kind_b = detect_kind(file_b)?;
    if kind_a != kind_b {
        return Err(AppError::new(
            "2 file phải cùng loại mới so sánh được.".to_string(),
        ));
    }

    for p in [file_a, file_b] {
        if !Path::new(p).exists() {
            return Err(AppError::new(format!("Không tìm thấy file: {p}")));
        }
    }

    match kind_a {
        Kind::Text | Kind::Markdown | Kind::Word => {
            let text_a = read_as_lines(file_a, kind_a)?;
            let text_b = read_as_lines(file_b, kind_a)?;
            let text_diff = diff_lines(&text_a, &text_b);
            Ok(CompareResult {
                kind: kind_a.as_str().to_string(),
                text_diff: Some(text_diff),
                excel_diff: None,
            })
        }
        Kind::Excel => {
            let excel_diff = diff_excel(file_a, file_b)?;
            Ok(CompareResult {
                kind: kind_a.as_str().to_string(),
                text_diff: None,
                excel_diff: Some(excel_diff),
            })
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Text,
    Markdown,
    Word,
    Excel,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Text => "text",
            Kind::Markdown => "markdown",
            Kind::Word => "word",
            Kind::Excel => "excel",
        }
    }
}

/// Xác định loại file theo phần mở rộng.
fn detect_kind(path: &str) -> AppResult<Kind> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "md" | "markdown" => Ok(Kind::Markdown),
        "txt" | "text" | "log" | "csv" => Ok(Kind::Text),
        "docx" => Ok(Kind::Word),
        "xlsx" | "xls" | "xlsm" => Ok(Kind::Excel),
        other => Err(AppError::new(format!(
            "Định dạng không hỗ trợ: .{other}. Chỉ hỗ trợ text, markdown, .docx, Excel."
        ))),
    }
}

/// Đọc file về mảng dòng theo loại.
fn read_as_lines(path: &str, kind: Kind) -> AppResult<Vec<String>> {
    match kind {
        Kind::Word => read_docx_paragraphs(path),
        _ => {
            let content = std::fs::read_to_string(path)
                .map_err(|e| AppError::new(format!("Không đọc được file {path}: {e}")))?;
            Ok(content.lines().map(|l| l.to_string()).collect())
        }
    }
}

/// Trích các đoạn văn từ `.docx`: mỗi `<w:p>` → 1 dòng (ghép text các `<w:t>`).
fn read_docx_paragraphs(path: &str) -> AppResult<Vec<String>> {
    let file = File::open(path)
        .map_err(|e| AppError::new(format!("Không mở được file Word {path}: {e}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::new(format!("File .docx không hợp lệ: {e}")))?;
    let mut xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|e| AppError::new(format!("Thiếu word/document.xml trong .docx: {e}")))?
        .read_to_string(&mut xml)
        .map_err(|e| AppError::new(format!("Không đọc được nội dung .docx: {e}")))?;

    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| AppError::new(format!("Lỗi phân tích XML .docx: {e}")))?;

    let mut paragraphs: Vec<String> = Vec::new();
    for node in doc.descendants() {
        if node.tag_name().name() == "p" {
            let mut text = String::new();
            for child in node.descendants() {
                match child.tag_name().name() {
                    "t" => text.push_str(child.text().unwrap_or("")),
                    "tab" => text.push('\t'),
                    "br" | "cr" => text.push('\n'),
                    _ => {}
                }
            }
            paragraphs.push(text);
        }
    }
    Ok(paragraphs)
}

/// Diff 2 mảng dòng bằng thuật toán Myers (`similar`).
fn diff_lines(a: &[String], b: &[String]) -> TextDiff {
    let text_a = a.join("\n");
    let text_b = b.join("\n");
    let diff = SimilarTextDiff::from_lines(&text_a, &text_b);

    let mut lines: Vec<DiffLine> = Vec::new();
    let mut added = 0usize;
    let mut removed = 0usize;

    for change in diff.iter_all_changes() {
        let tag = match change.tag() {
            ChangeTag::Equal => "equal",
            ChangeTag::Insert => {
                added += 1;
                "insert"
            }
            ChangeTag::Delete => {
                removed += 1;
                "delete"
            }
        };
        let content = change.value().trim_end_matches('\n').to_string();
        lines.push(DiffLine {
            tag: tag.to_string(),
            old_line: change.old_index().map(|i| i + 1),
            new_line: change.new_index().map(|i| i + 1),
            content,
        });
    }

    TextDiff {
        lines,
        added,
        removed,
    }
}

/// Đọc toàn bộ sheet của 1 workbook thành map: tên sheet → lưới giá trị ô.
fn read_workbook(path: &str) -> AppResult<Vec<(String, Vec<Vec<String>>)>> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| AppError::new(format!("Không mở được Excel {path}: {e}")))?;
    let names: Vec<String> = workbook.sheet_names().to_vec();
    let mut result: Vec<(String, Vec<Vec<String>>)> = Vec::new();
    for name in names {
        let Ok(range) = workbook.worksheet_range(&name) else {
            continue;
        };
        let mut grid: Vec<Vec<String>> = Vec::with_capacity(range.height());
        for row in range.rows() {
            grid.push(row.iter().map(cell_to_string).collect());
        }
        result.push((name, grid));
    }
    Ok(result)
}

/// Chuyển một cell calamine sang chuỗi hiển thị.
fn cell_to_string(cell: &Data) -> String {
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

/// Bỏ phần thập phân thừa cho số nguyên (vd 3.0 → "3").
fn trim_float(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        f.to_string()
    }
}

/// Nhãn cột kiểu bảng tính: 0→A, 1→B, …, 26→AA.
fn col_letter(mut index: usize) -> String {
    let mut label = String::new();
    loop {
        label.insert(0, (b'A' + (index % 26) as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    label
}

/// Multiset giá trị (bỏ ô rỗng) của từng cột — dùng để căn cột chịu được việc
/// thêm/xóa dòng.
fn column_multisets(grid: &[Vec<String>], cols: usize) -> Vec<HashMap<String, i64>> {
    let mut sets: Vec<HashMap<String, i64>> = vec![HashMap::new(); cols];
    for row in grid {
        for (c, set) in sets.iter_mut().enumerate() {
            if let Some(v) = row.get(c) {
                if !v.is_empty() {
                    *set.entry(v.clone()).or_insert(0) += 1;
                }
            }
        }
    }
    sets
}

/// Độ tương đồng 2 multiset (Sørensen–Dice): 2·|giao| / (|A|+|B|). 1.0 nếu cùng rỗng.
fn multiset_similarity(a: &HashMap<String, i64>, b: &HashMap<String, i64>) -> f64 {
    let total: i64 = a.values().sum::<i64>() + b.values().sum::<i64>();
    if total == 0 {
        return 1.0;
    }
    let mut inter = 0i64;
    for (k, va) in a {
        if let Some(vb) = b.get(k) {
            inter += (*va).min(*vb);
        }
    }
    (2 * inter) as f64 / total as f64
}

/// Một bước căn cột.
enum ColAlign {
    Match(usize, usize),
    DelA(usize),
    InsB(usize),
}

const COL_SIM_THRESHOLD: f64 = 0.5;

/// Căn cột bằng LCS "mờ": 2 cột coi là khớp khi độ tương đồng multiset ≥ ngưỡng.
/// Nhờ đó cột phía sau cột bị xóa không bị lệch, chịu được cả thêm/xóa dòng.
fn align_columns(
    sets_a: &[HashMap<String, i64>],
    sets_b: &[HashMap<String, i64>],
) -> Vec<ColAlign> {
    let n = sets_a.len();
    let m = sets_b.len();
    let eq = |i: usize, j: usize| multiset_similarity(&sets_a[i], &sets_b[j]) >= COL_SIM_THRESHOLD;

    // dp[i][j] = độ dài LCS xét từ cột i (của A) và j (của B) trở đi.
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if eq(i, j) {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut ops = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if eq(i, j) {
            ops.push(ColAlign::Match(i, j));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            ops.push(ColAlign::DelA(i));
            i += 1;
        } else {
            ops.push(ColAlign::InsB(j));
            j += 1;
        }
    }
    while i < n {
        ops.push(ColAlign::DelA(i));
        i += 1;
    }
    while j < m {
        ops.push(ColAlign::InsB(j));
        j += 1;
    }
    ops
}

/// So sánh cell-by-cell theo từng sheet (union tên sheet của 2 file).
fn diff_excel(file_a: &str, file_b: &str) -> AppResult<ExcelDiff> {
    let book_a = read_workbook(file_a)?;
    let book_b = read_workbook(file_b)?;

    // Giữ thứ tự xuất hiện: sheet của A trước, sheet chỉ có ở B nối sau.
    let mut sheet_order: Vec<String> = book_a.iter().map(|(n, _)| n.clone()).collect();
    let existing: BTreeSet<String> = sheet_order.iter().cloned().collect();
    for (n, _) in &book_b {
        if !existing.contains(n) {
            sheet_order.push(n.clone());
        }
    }

    let mut sheets: Vec<SheetDiff> = Vec::new();
    for name in sheet_order {
        let grid_a = book_a
            .iter()
            .find(|(n, _)| n == &name)
            .map(|(_, g)| g.clone())
            .unwrap_or_default();
        let grid_b = book_b
            .iter()
            .find(|(n, _)| n == &name)
            .map(|(_, g)| g.clone())
            .unwrap_or_default();
        sheets.push(diff_sheet(name, &grid_a, &grid_b));
    }

    Ok(ExcelDiff { sheets })
}

/// Ô tại giao (dòng, cột) đã căn — chứa chỉ số nguồn ở A/B (nếu có).
struct Slot {
    a: Option<usize>,
    b: Option<usize>,
    tag: &'static str, // equal | added | removed (cấp trục)
}

/// Ký tự phân tách khi ghép "khóa dòng" (khó xuất hiện trong dữ liệu).
const KEY_SEP: char = '\u{1}';

/// So sánh 2 lưới ô của cùng một sheet, có căn chỉnh cột + dòng bằng LCS.
fn diff_sheet(name: String, a: &[Vec<String>], b: &[Vec<String>]) -> SheetDiff {
    let cols_a = a.iter().map(|r| r.len()).max().unwrap_or(0);
    let cols_b = b.iter().map(|r| r.len()).max().unwrap_or(0);

    // ── 1. Căn cột (LCS mờ theo multiset giá trị) ──
    let sets_a = column_multisets(a, cols_a);
    let sets_b = column_multisets(b, cols_b);
    let mut col_slots: Vec<Slot> = Vec::new();
    for op in align_columns(&sets_a, &sets_b) {
        match op {
            ColAlign::Match(i, j) => col_slots.push(Slot { a: Some(i), b: Some(j), tag: "equal" }),
            ColAlign::DelA(i) => col_slots.push(Slot { a: Some(i), b: None, tag: "removed" }),
            ColAlign::InsB(j) => col_slots.push(Slot { a: None, b: Some(j), tag: "added" }),
        }
    }

    // Cột khớp cặp — dùng để tạo khóa dòng (bỏ qua cột đã thêm/xóa).
    let matched_cols: Vec<(usize, usize)> = col_slots
        .iter()
        .filter_map(|s| match (s.a, s.b) {
            (Some(i), Some(j)) => Some((i, j)),
            _ => None,
        })
        .collect();

    let row_key = |grid: &[Vec<String>], r: usize, pick: &dyn Fn(&(usize, usize)) -> usize| {
        matched_cols
            .iter()
            .map(|pair| grid[r].get(pick(pair)).cloned().unwrap_or_default())
            .collect::<Vec<_>>()
            .join(&KEY_SEP.to_string())
    };
    let keys_a: Vec<String> = (0..a.len()).map(|r| row_key(a, r, &|p| p.0)).collect();
    let keys_b: Vec<String> = (0..b.len()).map(|r| row_key(b, r, &|p| p.1)).collect();

    // ── 2. Căn dòng (Myers trên khóa dòng) ──
    let mut row_slots: Vec<Slot> = Vec::new();
    for op in capture_diff_slices(Algorithm::Myers, &keys_a, &keys_b) {
        match op {
            DiffOp::Equal { old_index, new_index, len } => {
                for k in 0..len {
                    row_slots.push(Slot { a: Some(old_index + k), b: Some(new_index + k), tag: "equal" });
                }
            }
            DiffOp::Delete { old_index, old_len, .. } => {
                for k in 0..old_len {
                    row_slots.push(Slot { a: Some(old_index + k), b: None, tag: "removed" });
                }
            }
            DiffOp::Insert { old_index: _, new_index, new_len } => {
                for k in 0..new_len {
                    row_slots.push(Slot { a: None, b: Some(new_index + k), tag: "added" });
                }
            }
            // Replace: ghép theo vị trí trong khối để lộ ô bị sửa (thay vì xóa+thêm cả dòng).
            DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                let n = old_len.max(new_len);
                for k in 0..n {
                    let a_row = (k < old_len).then_some(old_index + k);
                    let b_row = (k < new_len).then_some(new_index + k);
                    let tag = match (a_row, b_row) {
                        (Some(_), Some(_)) => "equal",
                        (Some(_), None) => "removed",
                        _ => "added",
                    };
                    row_slots.push(Slot { a: a_row, b: b_row, tag });
                }
            }
        }
    }

    // ── 3. Dựng lưới ô + đếm ──
    let mut cells: Vec<Vec<CellDiff>> = Vec::with_capacity(row_slots.len());
    let (mut changed, mut added, mut removed) = (0usize, 0usize, 0usize);
    for rs in &row_slots {
        let mut row_cells: Vec<CellDiff> = Vec::with_capacity(col_slots.len());
        for cs in &col_slots {
            let old_v = match (rs.a, cs.a) {
                (Some(r), Some(c)) => a[r].get(c).cloned().unwrap_or_default(),
                _ => String::new(),
            };
            let new_v = match (rs.b, cs.b) {
                (Some(r), Some(c)) => b[r].get(c).cloned().unwrap_or_default(),
                _ => String::new(),
            };
            let a_present = rs.a.is_some() && cs.a.is_some();
            let b_present = rs.b.is_some() && cs.b.is_some();

            let tag = if a_present && b_present {
                if old_v == new_v {
                    "equal"
                } else {
                    changed += 1;
                    "changed"
                }
            } else if a_present {
                if old_v.is_empty() {
                    "equal"
                } else {
                    removed += 1;
                    "removed"
                }
            } else if b_present {
                if new_v.is_empty() {
                    "equal"
                } else {
                    added += 1;
                    "added"
                }
            } else {
                "equal"
            };

            row_cells.push(CellDiff { tag: tag.to_string(), old: old_v, new: new_v });
        }
        cells.push(row_cells);
    }

    // ── 4. Marker trục + đếm cột/dòng ──
    let columns: Vec<AxisMarker> = col_slots
        .iter()
        .map(|s| AxisMarker {
            tag: s.tag.to_string(),
            label: col_letter(s.a.or(s.b).unwrap_or(0)),
        })
        .collect();
    let rows_meta: Vec<AxisMarker> = row_slots
        .iter()
        .enumerate()
        .map(|(idx, _)| AxisMarker { tag: row_slots[idx].tag.to_string(), label: (idx + 1).to_string() })
        .collect();

    let col_added = col_slots.iter().filter(|s| s.tag == "added").count();
    let col_removed = col_slots.iter().filter(|s| s.tag == "removed").count();
    let row_added = row_slots.iter().filter(|s| s.tag == "added").count();
    let row_removed = row_slots.iter().filter(|s| s.tag == "removed").count();

    SheetDiff {
        name,
        columns,
        rows_meta,
        cells,
        changed,
        added,
        removed,
        col_added,
        col_removed,
        row_added,
        row_removed,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Xuất kết quả so sánh ra file Excel (.xlsx)
// ─────────────────────────────────────────────────────────────────────────

/// Một dòng trong bảng báo cáo xuất Excel.
struct ReportRow {
    sheet: String,
    object: String,
    change: String,
    old: String,
    new: String,
}

const JOIN_SEP: &str = " | ";

/// So sánh lại 2 file rồi xuất báo cáo khác biệt ra `output_path` (.xlsx).
pub fn export_excel(file_a: &str, file_b: &str, output_path: &str) -> AppResult<String> {
    let result = compare(file_a, file_b)?;
    let rows = build_report(&result);
    write_report_xlsx(&rows, output_path)?;
    Ok(output_path.to_string())
}

/// Dựng danh sách dòng báo cáo từ kết quả so sánh.
fn build_report(result: &CompareResult) -> Vec<ReportRow> {
    let mut rows = Vec::new();
    if let Some(excel) = &result.excel_diff {
        for sheet in &excel.sheets {
            build_sheet_report(sheet, &mut rows);
        }
    } else if let Some(text) = &result.text_diff {
        build_text_report(text, &mut rows);
    }
    if rows.is_empty() {
        rows.push(ReportRow {
            sheet: "—".into(),
            object: "—".into(),
            change: "Không có khác biệt".into(),
            old: String::new(),
            new: String::new(),
        });
    }
    rows
}

/// Báo cáo cho một sheet Excel: cột/dòng thêm-xóa (gộp) + ô sửa (chi tiết).
fn build_sheet_report(sheet: &crate::models::file_compare::SheetDiff, rows: &mut Vec<ReportRow>) {
    let name = &sheet.name;

    // Cột thêm / xóa (gộp nội dung cả cột).
    for (c, col) in sheet.columns.iter().enumerate() {
        if col.tag == "removed" {
            let old = join_column(sheet, c, true);
            rows.push(ReportRow {
                sheet: name.clone(),
                object: format!("Cột {}", col.label),
                change: "Xóa cột".into(),
                old,
                new: String::new(),
            });
        } else if col.tag == "added" {
            let new = join_column(sheet, c, false);
            rows.push(ReportRow {
                sheet: name.clone(),
                object: format!("Cột {}", col.label),
                change: "Thêm cột".into(),
                old: String::new(),
                new,
            });
        }
    }

    // Dòng thêm / xóa (gộp nội dung cả dòng).
    for (r, row) in sheet.rows_meta.iter().enumerate() {
        if row.tag == "removed" {
            let old = join_row(sheet, r, true);
            rows.push(ReportRow {
                sheet: name.clone(),
                object: format!("Dòng {}", row.label),
                change: "Xóa dòng".into(),
                old,
                new: String::new(),
            });
        } else if row.tag == "added" {
            let new = join_row(sheet, r, false);
            rows.push(ReportRow {
                sheet: name.clone(),
                object: format!("Dòng {}", row.label),
                change: "Thêm dòng".into(),
                old: String::new(),
                new,
            });
        }
    }

    // Ô bị sửa (chỉ ở giao của cột & dòng đều "equal").
    for (r, row) in sheet.rows_meta.iter().enumerate() {
        if row.tag != "equal" {
            continue;
        }
        for (c, col) in sheet.columns.iter().enumerate() {
            if col.tag != "equal" {
                continue;
            }
            let cell = &sheet.cells[r][c];
            if cell.tag == "changed" {
                rows.push(ReportRow {
                    sheet: name.clone(),
                    object: format!("Ô {}{}", col.label, row.label),
                    change: "Sửa".into(),
                    old: cell.old.clone(),
                    new: cell.new.clone(),
                });
            }
        }
    }
}

/// Gộp nội dung 1 cột (bỏ ô rỗng). `old = true` lấy giá trị cũ, ngược lại giá trị mới.
fn join_column(sheet: &crate::models::file_compare::SheetDiff, c: usize, old: bool) -> String {
    sheet
        .cells
        .iter()
        .filter_map(|row| {
            let v = if old { &row[c].old } else { &row[c].new };
            (!v.is_empty()).then(|| v.clone())
        })
        .collect::<Vec<_>>()
        .join(JOIN_SEP)
}

/// Gộp nội dung 1 dòng (bỏ cột đã thêm/xóa và ô rỗng).
fn join_row(sheet: &crate::models::file_compare::SheetDiff, r: usize, old: bool) -> String {
    sheet.cells[r]
        .iter()
        .enumerate()
        .filter_map(|(c, cell)| {
            // Với dòng xóa bỏ cột "added"; với dòng thêm bỏ cột "removed".
            let skip = if old {
                sheet.columns[c].tag == "added"
            } else {
                sheet.columns[c].tag == "removed"
            };
            if skip {
                return None;
            }
            let v = if old { &cell.old } else { &cell.new };
            (!v.is_empty()).then(|| v.clone())
        })
        .collect::<Vec<_>>()
        .join(JOIN_SEP)
}

/// Báo cáo cho text/markdown/word: ghép delete+insert liền kề thành "Sửa".
fn build_text_report(text: &TextDiff, rows: &mut Vec<ReportRow>) {
    let lines = &text.lines;
    let mut i = 0;
    while i < lines.len() {
        match lines[i].tag.as_str() {
            "equal" => i += 1,
            "delete" => {
                let mut dels = Vec::new();
                while i < lines.len() && lines[i].tag == "delete" {
                    dels.push(&lines[i]);
                    i += 1;
                }
                let mut ins = Vec::new();
                while i < lines.len() && lines[i].tag == "insert" {
                    ins.push(&lines[i]);
                    i += 1;
                }
                let n = dels.len().max(ins.len());
                for k in 0..n {
                    match (dels.get(k), ins.get(k)) {
                        (Some(d), Some(a)) => rows.push(ReportRow {
                            sheet: "-".into(),
                            object: format!("Dòng {}", a.new_line.unwrap_or(0)),
                            change: "Sửa".into(),
                            old: d.content.clone(),
                            new: a.content.clone(),
                        }),
                        (Some(d), None) => rows.push(ReportRow {
                            sheet: "-".into(),
                            object: format!("Dòng {}", d.old_line.unwrap_or(0)),
                            change: "Xóa".into(),
                            old: d.content.clone(),
                            new: String::new(),
                        }),
                        (None, Some(a)) => rows.push(ReportRow {
                            sheet: "-".into(),
                            object: format!("Dòng {}", a.new_line.unwrap_or(0)),
                            change: "Thêm".into(),
                            old: String::new(),
                            new: a.content.clone(),
                        }),
                        (None, None) => {}
                    }
                }
            }
            "insert" => {
                while i < lines.len() && lines[i].tag == "insert" {
                    let a = &lines[i];
                    rows.push(ReportRow {
                        sheet: "-".into(),
                        object: format!("Dòng {}", a.new_line.unwrap_or(0)),
                        change: "Thêm".into(),
                        old: String::new(),
                        new: a.content.clone(),
                    });
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
}

/// Màu nền theo loại thay đổi.
fn change_format(change: &str) -> Format {
    let base = Format::new().set_border(rust_xlsxwriter::FormatBorder::Thin);
    if change.contains("Thêm") {
        base.set_background_color(Color::RGB(0xDCFCE7)).set_font_color(Color::RGB(0x166534))
    } else if change.contains("Xóa") {
        base.set_background_color(Color::RGB(0xFEE2E2)).set_font_color(Color::RGB(0x991B1B))
    } else if change.contains("Sửa") {
        base.set_background_color(Color::RGB(0xFEF3C7)).set_font_color(Color::RGB(0x92400E))
    } else {
        base.set_font_color(Color::RGB(0x6B7280))
    }
}

/// Ghi danh sách báo cáo ra file .xlsx.
fn write_report_xlsx(rows: &[ReportRow], output_path: &str) -> AppResult<()> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    let border = rust_xlsxwriter::FormatBorder::Thin;
    let header_fmt = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xE5E7EB))
        .set_font_color(Color::RGB(0x111827))
        .set_border(border)
        .set_text_wrap();
    let cell_fmt = Format::new().set_border(border).set_text_wrap();
    let center_fmt = Format::new()
        .set_border(border)
        .set_align(rust_xlsxwriter::FormatAlign::Center);

    let headers = ["STT", "Sheet", "Đối tượng", "Thay đổi", "Nội dung cũ", "Nội dung mới"];
    let widths = [6.0, 18.0, 16.0, 12.0, 48.0, 48.0];
    for (c, (h, w)) in headers.iter().zip(widths.iter()).enumerate() {
        let col = c as u16;
        sheet
            .set_column_width(col, *w)
            .map_err(|e| AppError::new(format!("Lỗi ghi Excel: {e}")))?;
        sheet
            .write_string_with_format(0, col, *h, &header_fmt)
            .map_err(|e| AppError::new(format!("Lỗi ghi Excel: {e}")))?;
    }

    for (i, row) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        let mut w = |col: u16, val: &str, fmt: &Format| -> AppResult<()> {
            sheet
                .write_string_with_format(r, col, val, fmt)
                .map_err(|e| AppError::new(format!("Lỗi ghi Excel: {e}")))?;
            Ok(())
        };
        w(0, &(i + 1).to_string(), &center_fmt)?;
        w(1, &row.sheet, &cell_fmt)?;
        w(2, &row.object, &cell_fmt)?;
        w(3, &row.change, &change_format(&row.change))?;
        w(4, &row.old, &cell_fmt)?;
        w(5, &row.new, &cell_fmt)?;
    }

    sheet
        .set_freeze_panes(1, 0)
        .map_err(|e| AppError::new(format!("Lỗi ghi Excel: {e}")))?;
    workbook
        .save(output_path)
        .map_err(|e| AppError::new(format!("Không lưu được file Excel: {e}")))?;
    Ok(())
}
