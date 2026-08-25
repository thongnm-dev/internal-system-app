//! Method xử lý riêng cho C2.3.4 画面仕様書（編集要領）.
//!
//! Khác với các loại tài liệu khác, pipeline "Áp dụng" ở đây clone TOÀN BỘ nội dung sheet VN
//! vào JP output thay vì chỉ ghi đè ô đỏ — vì C2.3.4 có thêm dòng mới, đổi style cả dòng,
//! đổi màu nền, border... đòi hỏi toàn bộ nội dung + style của VN phải được phản ánh chính xác
//! vào kết quả. Cột A được bảo toàn bằng công thức JP thay vì nội dung VN.
//!
//! Pipeline `apply_changes`:
//! 1. `sync_structure` — dọn dẹp JP, clone sheet chỉ có ở VN, đổi tên "(DEL)", sắp xếp sheet.
//! 2. Merge styles VN→JP.
//! 3. Quét toàn bộ ô VN "coi như đã thay đổi": strikethrough HOẶC màu chữ KHÔNG phải đen (bất kỳ
//!    màu nào, không riêng đỏ/xanh) — `find_changed_style_cells_xlsx`.
//!    Ngoại lệ: ô mà TẤT CẢ nội dung đều bị gạch bỏ VÀ có màu không đen (toàn bộ cell bị xóa)
//!    — `find_fully_struck_colored_cells_xlsx` — được loại khỏi tập "đã thay đổi": giữ nội dung
//!    JP nhưng format strike/color từ VN được áp qua cell style.
//! 4. Vòng loop qua từng sheet chung (VN ∩ JP − cloned − DEL):
//!    a. Trích công thức cột A từ JP (`extract_jp_col_a_info`).
//!    a2. CHÈN DÒNG VẬT LÝ vào JP theo rule cố định (không dựa anchor/neo) TRƯỚC khi clone —
//!        `compute_row_insertions` + `apply_row_insertions`: dòng VN mà mọi ô dữ liệu đều "toàn ô
//!        đã đổi" (`find_fully_changed_cells_xlsx`, trừ ô gạch+đổi màu toàn bộ vì đó là quy ước
//!        "không đổi nội dung") coi là dòng hoàn toàn mới ở VN, và dòng VN trống trong khi JP tại
//!        cùng vị trí còn dữ liệu đều được chèn thêm 1 dòng vào JP (nội dung VN hoặc trống) để
//!        giữ đúng vị trí so sánh cho các dòng còn lại — thay thế cơ chế ánh xạ ảo dựa anchor cho
//!        đúng trường hợp chèn/xóa dòng thật, xem doc `compute_row_insertions`.
//!    a3. `align_vn_jp_row_map` chạy TRÊN cấu trúc JP đã chèn ở bước a2, chỉ còn vai trò bù lệch
//!        còn lại (nếu có) — rỗng nếu không tìm được neo, khi đó rơi về ánh xạ 1:1 (đã đúng nhờ
//!        bước a2).
//!    b. Clone toàn bộ sheet VN vào JP output (`clone_vn_sheet_for_jp`):
//!       - Hàng header (row < 7): giữ nguyên (kể cả cột A), NGOẠI TRỪ các ô A3, C3, E3, F3
//!         luôn lấy từ JP (`jp_preserved_header_cells`).
//!       - Hàng nội dung (row ≥ 7): remap style, inline string, thay cột A bằng công thức JP;
//!         riêng từng ô (trừ cột A) — nếu ô đó KHÔNG nằm trong tập "đã thay đổi" ở bước 3 (chữ
//!         đen, không strikethrough) VÀ JP đã có ô tại đúng vị trí đó, GIỮ NGUYÊN ô JP thay vì
//!         ghi đè bằng VN.
//!       - Riêng sheet "変更履歴" (`CHANGE_HISTORY_SHEET_NAME`): KHÔNG có cột STT tự đánh số —
//!         `use_col_a_formula = false` nên cột A được xử lý như mọi cột thường (clone VN / giữ
//!         JP nếu không đổi), không bị ghi đè bằng công thức JP.
//! 4b. Cleanup: xóa dòng thừa ở cuối mỗi sheet — chỉ giữ đến dòng cuối cùng có nội dung thật
//!     trong VN (`find_last_content_row` + `strip_trailing_rows`), cập nhật `<dimension>`.
//! 4c. Chuẩn hóa rows 4-6: toàn bộ cells ở rows 4, 5, 6 được thay bằng JP reference
//!     (`normalize_rows_4_to_6`). Xóa mergeCell bao phủ A4:A6 — đảm bảo A4="項", A5=blank,
//!     A6="番" là cells riêng lẻ, không merge. Áp dụng cho cả sheet chung và sheet clone (trừ
//!     変更履歴).
//! 4d. Sheet clone trực tiếp từ VN: các ô header row 3 (A3, C3, E3, F3) được ghi đè bằng nội
//!     dung từ sheet JP bất kỳ (`patch_header_cells_in_sheet`), sau đó normalize rows 4-6.
//! 5. Ghi output.

use std::collections::{HashMap, HashSet};
use std::fs::File;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::vnjp_sync::ApplyResult;

use crate::models::vnjp_sync::CellDataMismatch;

use super::sync_service::{
    align_vn_jp_row_map, apply_dictionary_and_verify_data, apply_row_insertions, apply_surgery,
    build_replace_dictionary, clone_vn_sheet_for_jp, compute_row_insertions,
    extract_all_shared_strings, extract_jp_col_a_info, find_changed_style_cells_xlsx,
    find_fully_changed_cells_xlsx, find_fully_struck_colored_cells_xlsx, is_del_sheet_name,
    merge_vn_styles_into_jp, merged_output_path, parse_cell_ref, read_zip_entry,
    resolve_sheet_xml_paths, sync_structure, write_output_zip, BorderUnionExtender, ContentBounds,
    SurgeryEdit, CHANGE_HISTORY_SHEET_NAME,
};

/// Nội dung cột A ~ M (0-based 12).
pub fn content_bounds(sheet_name: &str) -> ContentBounds {
    super::sync_service::content_bounds_for_sheet(sheet_name, 12)
}

/// Các ô header ROW 3 luôn lấy từ JP: A3, C3, E3, F3 — (row1, col0).
/// Rows 4-6 được xử lý riêng bởi `normalize_rows_4_to_6`.
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

/// Pipeline "Áp dụng" VN → JP cho C2.3.4. Xem module doc.
pub fn apply_changes(vn_path: &str, jp_path: &str) -> AppResult<ApplyResult> {
    // ── 1. sync_structure ──────────────────────────────────────────────────────
    let output_path = merged_output_path(jp_path)?;
    let output_path_str = output_path.to_string_lossy().to_string();
    let structure = sync_structure(vn_path, jp_path, &output_path_str)?;

    // ── 2. Mở VN zip ──────────────────────────────────────────────────────────
    let vn_file =
        File::open(vn_path).map_err(|e| AppError::new(format!("Không mở được file VN: {e}")))?;
    let mut vn_archive = zip::ZipArchive::new(vn_file)
        .map_err(|e| AppError::new(format!("File VN không phải ZIP hợp lệ: {e}")))?;

    let vn_wb_xml = read_zip_entry(&mut vn_archive, "xl/workbook.xml").unwrap_or_default();
    let vn_rels_xml = read_zip_entry(&mut vn_archive, "xl/_rels/workbook.xml.rels").unwrap_or_default();
    let vn_sst_xml = read_zip_entry(&mut vn_archive, "xl/sharedStrings.xml").unwrap_or_default();
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
    let jp_rels_xml = read_zip_entry(&mut jp_archive, "xl/_rels/workbook.xml.rels").unwrap_or_default();
    let jp_styles_xml = read_zip_entry(&mut jp_archive, "xl/styles.xml").unwrap_or_default();
    let jp_sst_xml = read_zip_entry(&mut jp_archive, "xl/sharedStrings.xml").unwrap_or_default();
    let (jp_plain_ssi, _jp_rich_ssi) = extract_all_shared_strings(&jp_sst_xml);
    let jp_sheet_paths = resolve_sheet_xml_paths(&jp_wb_xml, &jp_rels_xml);
    let jp_sheet_map: HashMap<String, String> = jp_sheet_paths.into_iter().collect();

    // ── 4. Merge styles VN→JP ─────────────────────────────────────────────────
    let style_result = merge_vn_styles_into_jp(&jp_styles_xml, &vn_styles_xml);
    // Tích lũy border union JP+VN cho ô "không đổi" giữ nội dung JP (xem `BorderUnionExtender`) —
    // ghép vào styles.xml đã merge sau khi clone xong toàn bộ sheet (bước 6b/7).
    let mut border_ext = BorderUnionExtender::new(&style_result);

    // Ô VN "coi như đã thay đổi" (strikethrough hoặc màu chữ không phải đen) theo từng sheet — ô
    // KHÔNG nằm trong set này (chữ đen, không strikethrough) sẽ được giữ nguyên bản JP tại đúng
    // vị trí khi clone (xem `clone_vn_sheet_for_jp`).
    let vn_changed_cells = find_changed_style_cells_xlsx(vn_path);
    // Ô VN mà TẤT CẢ nội dung đều bị gạch bỏ + màu không đen → coi như KHÔNG thay đổi nội dung
    // (giữ JP) nhưng format strike/color được phản ánh qua cell style.
    let vn_fully_struck = find_fully_struck_colored_cells_xlsx(vn_path);
    // Ô VN mà TOÀN BỘ nội dung (mọi run) đã đổi — dùng riêng cho rule "dòng hoàn toàn mới ở VN"
    // (`compute_row_insertions`), nghiêm ngặt hơn `vn_changed_cells` (xem doc hàm đó).
    let vn_fully_changed_cells = find_fully_changed_cells_xlsx(vn_path);

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

    // Row bắt đầu vùng nội dung (1-based): row 7 = CONTENT_START_ROW0 (6) + 1
    const CONTENT_START_ROW1: usize = 7;

    let preserved_cells = jp_preserved_header_cells();

    // Trích JP reference rows 4-6 (cells) trước khi xử lý — dùng để chuẩn hóa header sau.
    let jp_ref_rows = extract_jp_ref_rows_4_to_6(&mut jp_archive, &jp_sheet_map, cloned_names);

    let empty_changed: HashSet<(usize, usize)> = HashSet::new();
    let mut replaced: HashMap<String, Vec<u8>> = HashMap::new();
    let mut applied_count = 0usize;
    let mut sheets_modified: Vec<String> = structure.sheets_modified.clone();

    // Styles đã merge — ghi vào output
    replaced.insert("xl/styles.xml".to_string(), style_result.new_styles_xml.into_bytes());

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
        let mut jp_sheet_xml = match read_zip_entry(&mut jp_archive, &jp_xml_path) {
            Some(x) => x,
            None => continue,
        };

        // Trích công thức + style + row bắt đầu thực sự của cột A JP
        // (row đầu tiên CÓ công thức — bỏ qua "STT" và các ô header khác). Công thức là mẫu
        // tương đối theo ROW()/COLUMN() (xem `extract_jp_col_a_info`) — không phụ thuộc số dòng
        // thật, nên trích TRƯỚC khi chèn dòng vật lý bên dưới vẫn đúng.
        let (jp_col_a_formula, jp_col_a_style, formula_start_row1) = extract_jp_col_a_info(&jp_sheet_xml, CONTENT_START_ROW1);
        // Riêng sheet "変更履歴": KHÔNG có cột STT đánh số tự động — cột A là dữ liệu thật (ngày/số
        // phiên bản...) nên phải giữ nguyên như mọi cột khác (clone VN / giữ JP nếu không đổi),
        // không áp công thức JP.
        let use_col_a_formula = sheet_name != CHANGE_HISTORY_SHEET_NAME;

        // Tập ô "đã thay đổi" hiệu dụng = changed − fully_struck_colored: ô mà TẤT CẢ nội dung
        // bị gạch bỏ + màu không đen được loại khỏi tập "đã thay đổi" → giữ nội dung JP, chỉ áp
        // style VN (strike + color) qua cell format.
        let effective_changed: HashSet<(usize, usize)> = match vn_changed_cells.get(sheet_name) {
            Some(changed) => match vn_fully_struck.get(sheet_name) {
                Some(struck) => changed.difference(struck).copied().collect(),
                None => changed.clone(),
            },
            None => empty_changed.clone(),
        };

        // Dùng `formula_start_row1` (không phải `content_bounds` chung — cố định row1 >= 4) làm
        // mốc bắt đầu: C2.3.4 có thêm rows 4-6 là header phụ (xử lý riêng bởi
        // `normalize_rows_4_to_6`), KHÔNG thuộc vùng "danh sách STT" cần canh dòng — nếu dùng mốc
        // chung sẽ lẫn các dòng header phụ vào neo/khoảng hở, gây lệch ngay từ đầu danh sách.
        let alignment_bounds = ContentBounds {
            start_row0: formula_start_row1.saturating_sub(1),
            last_col0: content_bounds(sheet_name).last_col0,
        };
        // Cột bỏ qua khi xét rule chèn dòng / khi clone dòng VN-only: cột A (STT) tự đánh số lại,
        // luôn có dữ liệu và luôn đen — không mang tín hiệu gì về việc chèn dòng (xem doc
        // `compute_row_insertions`), và khi có công thức JP thì cột A của dòng chèn cũng phải để
        // JP tự đánh số lại (không giữ số VN).
        let skip_col0s: Vec<usize> = if use_col_a_formula { vec![0] } else { vec![] };

        // Chèn dòng VẬT LÝ vào JP theo rule (không dựa anchor) TRƯỚC khi tính alignment/clone —
        // xem doc `compute_row_insertions`: dòng VN hoàn toàn mới (mọi ô đã đổi) hoặc dòng VN
        // trống trong khi JP còn dữ liệu đều được chèn thêm 1 dòng vào JP để giữ đúng vị trí so
        // sánh cho các dòng còn lại.
        // "Toàn ô đã đổi" NHƯNG loại trừ ô "gạch bỏ + đổi màu toàn bộ" (`vn_fully_struck`) — ô đó
        // theo quy ước của tài liệu là "KHÔNG đổi nội dung, giữ JP, chỉ áp style" (xem module doc
        // + `effective_changed`), không phải dấu hiệu "dòng mới". Không loại trừ sẽ nhận lầm các
        // dòng bị đánh dấu xoá/gạch toàn dòng (đỏ + strike đồng thời) thành "dòng mới ở VN" — đã
        // xảy ra thực tế với các dòng dùng style vừa đỏ vừa gạch cho CẢ NHÃN (không phải kiểu
        // "tên cũ gạch đen + tên mới tô đỏ" của dòng đổi tên thông thường).
        let vn_fully_changed_not_struck: HashSet<(usize, usize)> =
            match vn_fully_changed_cells.get(sheet_name) {
                Some(fully_changed) => match vn_fully_struck.get(sheet_name) {
                    Some(struck) => fully_changed.difference(struck).copied().collect(),
                    None => fully_changed.clone(),
                },
                None => HashSet::new(),
            };
        let row_insertions = compute_row_insertions(
            &vn_sheet_xml,
            &jp_sheet_xml,
            &vn_plain_ssi,
            &jp_plain_ssi,
            &vn_fully_changed_not_struck,
            alignment_bounds,
            &skip_col0s,
        );
        if !row_insertions.is_empty() {
            jp_sheet_xml = apply_row_insertions(
                &jp_sheet_xml,
                &vn_sheet_xml,
                &row_insertions,
                &skip_col0s,
                &style_result.xf_remap,
                &vn_plain_ssi,
                &vn_rich_ssi,
            );
        }

        // Ánh xạ vn_row1 → jp_row1 theo ô neo — bù lệch dòng còn lại (nếu có) sau khi đã chèn dòng
        // vật lý ở trên (xem `clone_vn_sheet_for_jp`). Rỗng nếu sheet không tìm được neo — khi đó
        // hành vi giữ nguyên ánh xạ 1:1 như trước.
        let vn_to_jp_row = align_vn_jp_row_map(
            &vn_sheet_xml,
            &jp_sheet_xml,
            &vn_plain_ssi,
            &jp_plain_ssi,
            &effective_changed,
            alignment_bounds,
        );
        // Clone toàn bộ VN sheet vào khung JP (shapes/drawing được giữ từ JP); ô VN chữ đen và
        // không strikethrough (không có trong effective changed set) sẽ giữ nguyên bản JP cùng vị trí.
        let cloned_xml = clone_vn_sheet_for_jp(
            &vn_sheet_xml,
            &jp_sheet_xml,
            &jp_col_a_formula,
            jp_col_a_style,
            formula_start_row1,
            &style_result.xf_remap,
            &vn_plain_ssi,
            &vn_rich_ssi,
            &jp_plain_ssi,
            Some(&effective_changed),
            vn_fully_struck.get(sheet_name),
            use_col_a_formula,
            Some(&preserved_cells),
            Some(&vn_to_jp_row),
            &mut border_ext,
        );

        // Cleanup + normalize — bỏ qua 変更履歴 (cấu trúc khác, rows 4-6 là dữ liệu).
        let new_sheet_xml = if sheet_name != CHANGE_HISTORY_SHEET_NAME {
            let vn_last_row = find_last_content_row(&vn_sheet_xml);
            let stripped_xml = strip_trailing_rows(&cloned_xml, vn_last_row);
            normalize_rows_4_to_6(&stripped_xml, &jp_ref_rows)
        } else {
            cloned_xml
        };

        applied_count += 1;
        replaced.insert(jp_xml_path, new_sheet_xml.into_bytes());
        if !sheets_modified.contains(sheet_name) {
            sheets_modified.push(sheet_name.clone());
        }
    }

    // ── 6b. Cloned sheets: ghi đè header cells JP + normalize rows 4-6 ────
    // Sheet clone trực tiếp từ VN không có bản JP tương ứng — lấy header row 3 cells từ JP bất kỳ,
    // sau đó chuẩn hóa rows 4-6 bằng JP reference.
    if !cloned_names.is_empty() {
        let jp_header_cells =
            extract_jp_header_cells(&mut jp_archive, &jp_sheet_map, &preserved_cells);
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
            // Patch row 3 header cells
            let patched = if !jp_header_cells.is_empty() {
                patch_header_cells_in_sheet(&cloned_xml, &jp_header_cells)
            } else {
                cloned_xml
            };
            // Chuẩn hóa rows 4-6
            let normalized = normalize_rows_4_to_6(&patched, &jp_ref_rows);
            replaced.insert(cloned_xml_path, normalized.into_bytes());
        }
    }

    // ── 7. Ghi output ─────────────────────────────────────────────────────────
    // Ghép border union (nếu có) vào styles.xml đã merge — phải làm SAU khi clone xong toàn bộ
    // sheet (border_ext đã tích lũy đủ mọi <border>/<xf> union cần thêm).
    if let Some(styles_bytes) = replaced.remove("xl/styles.xml") {
        let updated = border_ext.finish(&String::from_utf8_lossy(&styles_bytes));
        replaced.insert("xl/styles.xml".to_string(), updated.into_bytes());
    }

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
        data_mismatches: Vec::new(),
        dictionary_applied_count: 0,
    })
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

/// Tìm dòng cuối cùng có ít nhất 1 ô chứa nội dung thật (<v>, <is>, <f>) trong sheetData.
/// Dòng chỉ có ô styled rỗng (vd `<c r="D30" s="5"/>`) không tính là có nội dung.
fn find_last_content_row(sheet_xml: &str) -> usize {
    let Ok(doc) = roxmltree::Document::parse(sheet_xml) else {
        return usize::MAX;
    };
    let Some(sd) = doc
        .descendants()
        .find(|n| n.tag_name().name() == "sheetData")
    else {
        return usize::MAX;
    };
    let mut last = 0usize;
    for row in sd.children().filter(|n| n.tag_name().name() == "row") {
        let row1 = row
            .attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let has_content = row
            .children()
            .filter(|c| c.tag_name().name() == "c")
            .any(|c| {
                c.children()
                    .any(|ch| matches!(ch.tag_name().name(), "v" | "is" | "f"))
            });
        if has_content {
            last = last.max(row1);
        }
    }
    last
}

/// Xóa các `<row>` có r > `max_row1` khỏi sheetData và cập nhật `<dimension>`.
fn strip_trailing_rows(sheet_xml: &str, max_row1: usize) -> String {
    if max_row1 == 0 || max_row1 == usize::MAX {
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
        if row1 > max_row1 {
            edits.push(SurgeryEdit {
                start: row.range().start,
                end: row.range().end,
                replacement: String::new(),
            });
        }
    }

    // Cập nhật <dimension> nếu tìm thấy
    if let Some(dim) = doc
        .descendants()
        .find(|n| n.tag_name().name() == "dimension")
    {
        if let Some(ref_val) = dim.attribute("ref") {
            if let Some((prefix, old_end)) = ref_val.rsplit_once(':') {
                // "A1:M45" → lấy col letter từ old_end, thay row bằng max_row1
                let col_part: String = old_end
                    .chars()
                    .take_while(|c| c.is_ascii_alphabetic())
                    .collect();
                let new_ref = format!("{prefix}:{col_part}{max_row1}");
                if let Some(attr) = dim.attributes().find(|a| a.name() == "ref") {
                    edits.push(SurgeryEdit {
                        start: attr.range_value().start,
                        end: attr.range_value().end,
                        replacement: new_ref,
                    });
                }
            }
        }
    }

    if edits.is_empty() {
        return sheet_xml.to_string();
    }

    apply_surgery(sheet_xml, edits)
}

/// Trích toàn bộ cell XML của rows 4, 5, 6 từ sheet JP đầu tiên có nội dung (bỏ qua
/// 変更履歴 và DEL sheets). Dùng làm reference để chuẩn hóa header rows cho tất cả các sheet.
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
/// Đảm bảo A4="項", A5=blank, A6="番" và không merge cell.
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

    // 1. Replace cells in rows 4, 5, 6 với JP reference
    for row in sd.children().filter(|n| n.tag_name().name() == "row") {
        let row1 = row
            .attribute("r")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let Some(jp_cells_xml) = jp_ref_cells.get(&row1) else {
            continue;
        };

        let first_cell = row.children().find(|c| c.tag_name().name() == "c");
        let last_cell = row.children().filter(|c| c.tag_name().name() == "c").last();

        if let (Some(first), Some(last)) = (first_cell, last_cell) {
            edits.push(SurgeryEdit {
                start: first.range().start,
                end: last.range().end,
                replacement: jp_cells_xml.clone(),
            });
        }
    }

    // 2. Xóa mergeCell entries bao phủ A4, A5, A6 (row0: 3, 4, 5 — col0: 0)
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

/// Thu thập từ điển replace (vn_text → jp_text) từ các ô mà output đã giữ nội dung JP.
pub fn build_dictionary(vn_path: &str, output_path: &str) -> AppResult<HashMap<String, String>> {
    build_replace_dictionary(vn_path, output_path, |sheet_name| {
        (7, content_bounds(sheet_name).last_col0)
    })
}

/// Áp dụng từ điển replace lên file, ĐỒNG THỜI kiểm tra output sau chuẩn hoá (so sự có mặt dữ
/// liệu — có/không — với file VN tại từng ô) trong CÙNG 1 lượt loop (xem
/// `sync_service::apply_dictionary_and_verify`).
pub fn apply_dictionary_and_verify(
    file_path: &str,
    vn_path: &str,
    output_path: &str,
    dictionary: &HashMap<String, String>,
) -> AppResult<(usize, Vec<CellDataMismatch>)> {
    apply_dictionary_and_verify_data(file_path, vn_path, output_path, dictionary, |sheet_name| {
        (7, content_bounds(sheet_name).last_col0)
    })
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
