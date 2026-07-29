//! Service resize ảnh evidence (hardcopy) trong toàn bộ workbook Excel.
//!
//! Không dùng thư viện ghi lại toàn bộ spreadsheet (rủi ro làm hỏng style/chart/macro
//! không liên quan khi round-trip). Thay vào đó, thao tác trực tiếp lên phần XML của
//! từng drawing part (`xl/drawings/drawingN.xml`) bằng cách thay thế đúng đoạn byte
//! chứa anchor của `<xdr:pic>` — `<xdr:sp>` (shape/textbox) và mọi phần khác của file
//! zip được giữ nguyên tuyệt đối.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::evidence_resize::{EvidenceResizeOptions, EvidenceResizeResult};
use image::GenericImageView;
use regex::Regex;
use roxmltree::{Document, Node};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// 1cm tính theo EMU (English Metric Units) — đơn vị kích thước trong OOXML drawing.
const EMU_PER_CM: f64 = 360_000.0;
/// 1 point tính theo EMU — dùng để quy đổi chiều cao dòng (row height, tính theo point).
const EMU_PER_POINT: f64 = 12_700.0;
/// Chiều cao dòng mặc định của Excel khi sheet không khai báo `defaultRowHeight` (đơn vị: point).
const DEFAULT_ROW_HEIGHT_PT: f64 = 15.0;

/// Resize toàn bộ ảnh (picture) trong workbook, giữ nguyên shape/textbox.
///
/// `width_cm`/`height_cm` đều optional: chỉ có Width → Height tự tính theo tỉ lệ khung hình
/// gốc của chính từng ảnh; chỉ có Height → ngược lại, Width tự tính theo tỉ lệ gốc; có cả
/// hai → mọi ảnh có cùng kích thước Width x Height; không có cái nào → không resize ảnh nào
/// cả (các setting khác như zoom/font/view/start column vẫn được áp dụng bình thường, riêng
/// start column bị bỏ qua vì nó cần ghi lại anchor cùng lúc với resize). Các ảnh xếp dọc cùng
/// cột sẽ được đẩy xuống để không đè lên nhau khi kích thước mới lớn hơn kích thước gốc, giữ
/// nguyên khoảng cách gốc giữa 2 ảnh liền kề và không đụng vào row height/lưới cột-dòng.
pub fn resize_evidence_images(
    input_path: String,
    output_path: String,
    width_cm: Option<f64>,
    height_cm: Option<f64>,
    options: EvidenceResizeOptions,
) -> AppResult<EvidenceResizeResult> {
    let input = PathBuf::from(input_path.trim());
    let output = PathBuf::from(output_path.trim());

    if input.as_os_str().is_empty() {
        return Err(AppError::new(
            "Please select an Excel workbook before resizing.",
        ));
    }
    if !input.exists() {
        return Err(AppError::new(format!(
            "Excel workbook not found: {}",
            input.display()
        )));
    }
    if !matches!(input.extension().and_then(|v| v.to_str()), Some("xlsx")) {
        return Err(AppError::new("Only .xlsx workbooks are supported."));
    }
    if output.as_os_str().is_empty() {
        return Err(AppError::new(
            "Please choose where to save the resized workbook.",
        ));
    }
    if let Some(w) = width_cm {
        if !(w > 0.0) {
            return Err(AppError::new("Width must be greater than 0."));
        }
    }
    if let Some(h) = height_cm {
        if !(h > 0.0) {
            return Err(AppError::new("Height must be greater than 0."));
        }
    }
    if let Some(z) = options.zoom_percent {
        if !(10..=400).contains(&z) {
            return Err(AppError::new("Zoom level must be between 10% and 400%."));
        }
    }
    if let Some(fs) = options.font_size {
        if !(fs > 0.0) {
            return Err(AppError::new("Font size must be greater than 0."));
        }
    }
    let override_col = match options.start_column.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => match parse_column_letters(raw) {
            Some(col) => Some(col),
            None => {
                return Err(AppError::new(
                    "Start column must look like a column letter, e.g. B or B2.",
                ));
            }
        },
        _ => None,
    };
    let font_name = options.font_name.as_deref().map(str::trim).filter(|v| !v.is_empty());

    let width_emu: Option<i64> = width_cm.map(|w| (w * EMU_PER_CM).round() as i64);
    let height_emu_fixed: Option<i64> = height_cm.map(|h| (h * EMU_PER_CM).round() as i64);
    let resize_requested = width_emu.is_some() || height_emu_fixed.is_some();

    let file = File::open(&input)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| AppError::new(format!("Could not open workbook as a zip archive: {e}")))?;

    let entry_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).map(|f| f.name().to_string()))
        .collect::<Result<_, _>>()
        .map_err(|e| AppError::new(format!("Could not read workbook entries: {e}")))?;

    let drawing_names: Vec<String> = entry_names
        .iter()
        .filter(|name| {
            name.starts_with("xl/drawings/drawing")
                && name.ends_with(".xml")
                && !name.contains("_rels")
        })
        .cloned()
        .collect();

    let mut images_resized: u32 = 0;
    let mut drawings_processed: u32 = 0;
    let mut warnings: Vec<String> = Vec::new();
    let mut replaced_parts: HashMap<String, Vec<u8>> = HashMap::new();

    // Chỉ quét toàn bộ workbook tìm formula/table/chart/pivot table nếu thật sự cần (tính năng
    // tránh đè content đang bật) — quét 1 lần duy nhất, dùng chung cho mọi sheet.
    let row_insertion_safe = if options.avoid_covering_content {
        workbook_supports_row_insertion(&entry_names, &mut archive)?
    } else {
        false
    };

    // Dòng đã chèn theo từng sheet (tên part) — dùng cho pass điều chỉnh definedNames sau cùng.
    let mut insertions_by_sheet: HashMap<String, Vec<RowInsertion>> = HashMap::new();

    if resize_requested {
        for drawing_name in &drawing_names {
            let xml = read_entry_to_string(&mut archive, drawing_name)?;
            let rels_name = drawing_rels_name(drawing_name);
            let rels_map = if entry_names.iter().any(|n| n == &rels_name) {
                parse_rels(&read_entry_to_string(&mut archive, &rels_name)?)
                    .map_err(|e| AppError::new(format!("Could not parse {rels_name}: {e}")))?
            } else {
                HashMap::new()
            };

            let owning_sheet = find_owning_sheet_name(drawing_name, &entry_names, &mut archive)?;
            let sheet_xml = match &owning_sheet {
                Some(name) => Some(read_entry_to_string(&mut archive, name)?),
                None => None,
            };
            let row_heights = sheet_xml
                .as_deref()
                .map(parse_row_heights)
                .transpose()
                .map_err(|e| AppError::new(format!("Could not parse row heights: {e}")))?;

            let outcome = resize_drawing_pics(
                &xml,
                &rels_map,
                width_emu,
                height_emu_fixed,
                row_heights.as_ref(),
                sheet_xml.as_deref(),
                options.avoid_covering_content,
                row_insertion_safe,
                override_col,
                &mut archive,
            )?;

            drawings_processed += 1;
            images_resized += outcome.resized_count;
            warnings.extend(outcome.warnings);

            if outcome.resized_count > 0 {
                replaced_parts.insert(drawing_name.clone(), outcome.new_drawing_xml.into_bytes());
            }
            if let (Some(sheet_name), Some(new_sheet_xml)) = (&owning_sheet, outcome.new_sheet_xml) {
                replaced_parts.insert(sheet_name.clone(), new_sheet_xml.into_bytes());
            }
            if !outcome.insertions.is_empty() {
                if let Some(sheet_name) = owning_sheet {
                    insertions_by_sheet.entry(sheet_name).or_default().extend(outcome.insertions);
                }
            }
        }
    } else if override_col.is_some() {
        warnings.push(
            "Custom start column requires Width or Height to be set; it was not applied."
                .to_string(),
        );
    }

    // Điều chỉnh definedNames (ví dụ Print_Area) trong xl/workbook.xml cho các sheet có dòng
    // được chèn — best-effort: bỏ qua (kèm warning) nếu không đối chiếu được tên sheet hoặc
    // giá trị definedName không đúng dạng 1 range đơn giản.
    if !insertions_by_sheet.is_empty() {
        let workbook_name = "xl/workbook.xml".to_string();
        let rels_name = "xl/_rels/workbook.xml.rels".to_string();
        if entry_names.iter().any(|n| n == &workbook_name) && entry_names.iter().any(|n| n == &rels_name) {
            let mut current_workbook_xml = read_entry_to_string(&mut archive, &workbook_name)?;
            let rels_xml = read_entry_to_string(&mut archive, &rels_name)?;
            let mut workbook_changed = false;
            for (sheet_file, sheet_insertions) in &insertions_by_sheet {
                match resolve_sheet_display_name(&current_workbook_xml, &rels_xml, sheet_file) {
                    Some(display_name) => {
                        if let Some(patched) =
                            patch_defined_names(&current_workbook_xml, &display_name, sheet_insertions)
                        {
                            current_workbook_xml = patched;
                            workbook_changed = true;
                        }
                    }
                    None => {
                        warnings.push(format!(
                            "Could not resolve the display name of {sheet_file}; defined names/print areas were left as-is."
                        ));
                    }
                }
            }
            if workbook_changed {
                replaced_parts.insert(workbook_name, current_workbook_xml.into_bytes());
            }
        } else {
            warnings.push(
                "Workbook has no xl/workbook.xml or rels; defined names/print areas were left as-is."
                    .to_string(),
            );
        }
    }

    // Zoom / Page Break Preview: áp dụng cho mọi sheet trong workbook. Nếu sheet đã được sửa ở
    // pass trước đó (ví dụ chèn dòng để tránh đè content), phải đọc tiếp từ bản ĐÃ SỬA trong
    // `replaced_parts` chứ không phải đọc lại từ archive gốc — nếu không sẽ vô tình ghi đè mất
    // thay đổi đó.
    if options.zoom_percent.is_some() || options.page_break_preview {
        for sheet_name in entry_names
            .iter()
            .filter(|n| n.starts_with("xl/worksheets/sheet") && n.ends_with(".xml") && !n.contains("_rels"))
        {
            let xml = match replaced_parts.get(sheet_name) {
                Some(bytes) => String::from_utf8(bytes.clone())
                    .map_err(|e| AppError::new(format!("Invalid UTF-8 in {sheet_name}: {e}")))?,
                None => read_entry_to_string(&mut archive, sheet_name)?,
            };
            match patch_sheet_view(&xml, options.zoom_percent, options.page_break_preview) {
                Some(new_xml) => {
                    replaced_parts.insert(sheet_name.clone(), new_xml.into_bytes());
                }
                None => {
                    warnings.push(format!("Could not find a <sheetView> to update in {sheet_name}."));
                }
            }
        }
    }

    // Font mặc định của workbook: chỉ 1 part duy nhất (xl/styles.xml), áp dụng 1 lần.
    if font_name.is_some() || options.font_size.is_some() {
        let styles_name = "xl/styles.xml".to_string();
        if entry_names.iter().any(|n| n == &styles_name) {
            let xml = read_entry_to_string(&mut archive, &styles_name)?;
            match patch_default_font(&xml, font_name, options.font_size) {
                Some(new_xml) => {
                    replaced_parts.insert(styles_name, new_xml.into_bytes());
                }
                None => {
                    warnings.push("Could not find the workbook's default font in xl/styles.xml.".to_string());
                }
            }
        } else {
            warnings.push("Workbook has no xl/styles.xml; default font was not changed.".to_string());
        }
    }

    write_output_archive(archive, &entry_names, &replaced_parts, &output)?;

    Ok(EvidenceResizeResult {
        source_path: input.display().to_string(),
        output_path: output.display().to_string(),
        source_file_name: file_name(&input),
        output_file_name: file_name(&output),
        images_resized,
        drawings_processed,
        warnings,
    })
}

/// Vị trí Y tuyệt đối (EMU) của các dòng trong 1 sheet, dựng từ row height thực tế —
/// dùng để quy đổi anchor (row/rowOff) sang toạ độ tuyệt đối và ngược lại, mà không cần
/// (và không) thay đổi bất kỳ row height nào trong file.
struct RowHeights {
    default_emu: f64,
    custom: HashMap<u32, f64>,
}

impl RowHeights {
    fn height_of(&self, row_1based: u32) -> f64 {
        *self.custom.get(&row_1based).unwrap_or(&self.default_emu)
    }

    /// Toạ độ Y tuyệt đối (EMU) của anchor point `(row0based, off)`.
    fn y_of(&self, row0based: u32, off: i64) -> i64 {
        let mut acc = 0f64;
        for r in 1..=row0based {
            acc += self.height_of(r);
        }
        acc.round() as i64 + off
    }

    /// Ngược lại: từ toạ độ Y tuyệt đối, tìm `(row0based, off)` tương ứng.
    fn locate(&self, y: i64) -> (u32, i64) {
        let mut acc = 0f64;
        let mut row: u32 = 0;
        loop {
            let h = self.height_of(row + 1);
            if row > 500_000 || acc + h > y as f64 {
                break;
            }
            acc += h;
            row += 1;
        }
        (row, y - acc.round() as i64)
    }
}

/// Một đoạn thay thế byte-range dùng chung cho mọi pass trong file này — thay `xml[start..end]`
/// bằng `replacement`, áp dụng theo thứ tự offset giảm dần (`apply_edits`) để offset của các
/// edit chưa áp dụng không bị lệch.
struct Edit {
    start: usize,
    end: usize,
    replacement: String,
}

/// Áp dụng danh sách edit (byte-range splicing) lên `xml`, theo thứ tự offset giảm dần.
fn apply_edits(xml: &str, mut edits: Vec<Edit>) -> String {
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

/// Kết quả xử lý 1 drawing part: XML drawing mới, số ảnh đã resize, warnings, XML sheet mới
/// (nếu có dòng được chèn vào sheet sở hữu drawing part này), và danh sách dòng đã chèn (dùng
/// cho pass điều chỉnh `definedNames` ở cấp workbook).
struct DrawingResizeOutcome {
    new_drawing_xml: String,
    resized_count: u32,
    warnings: Vec<String>,
    new_sheet_xml: Option<String>,
    insertions: Vec<RowInsertion>,
}

/// Một ảnh đã xác định được vị trí gốc (top/bottom Y tuyệt đối) — sẽ được nhóm theo cột
/// và đẩy xuống (nếu cần) ở pass 2 để không đè lên ảnh liền kề.
struct TrackedPic {
    anchor_start: usize,
    anchor_end: usize,
    col: i64,
    col_off: i64,
    orig_top_y: i64,
    orig_bottom_y: i64,
    new_cx: i64,
    new_cy: i64,
    pic_xml: String,
}

/// Duyệt một drawing part, resize mọi `<xdr:pic>` được neo trực tiếp vào
/// `twoCellAnchor`/`oneCellAnchor` (ảnh lồng trong `<xdr:grpSp>` bị bỏ qua, giống shape).
///
/// Pass 1: tính kích thước mới + vị trí gốc (nếu xác định được row height của sheet).
/// Pass 2: với các ảnh xác định được vị trí, nhóm theo cột, sắp theo thứ tự từ trên
/// xuống, rồi đẩy ảnh sau xuống nếu ảnh trước nó phình to hơn — giữ nguyên gap gốc. Nếu
/// `avoid_covering_content` và ảnh phình to sẽ đè lên 1 dòng có nội dung, hoặc chèn 1 dòng mới
/// (nếu `row_insertion_safe`) hoặc giới hạn chiều cao ảnh đó lại.
#[allow(clippy::too_many_arguments)]
fn resize_drawing_pics(
    xml: &str,
    rels: &HashMap<String, String>,
    width_emu: Option<i64>,
    height_emu_fixed: Option<i64>,
    row_heights: Option<&RowHeights>,
    sheet_xml: Option<&str>,
    avoid_covering_content: bool,
    row_insertion_safe: bool,
    override_col: Option<i64>,
    archive: &mut ZipArchive<File>,
) -> AppResult<DrawingResizeOutcome> {
    let doc = Document::parse(xml)
        .map_err(|e| AppError::new(format!("Could not parse drawing XML: {e}")))?;

    let mut edits: Vec<Edit> = Vec::new();
    let mut tracked: Vec<TrackedPic> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut resized_count: u32 = 0;

    for anchor in doc.descendants().filter(|n| {
        n.is_element() && matches!(n.tag_name().name(), "twoCellAnchor" | "oneCellAnchor")
    }) {
        let pic_node = match anchor
            .children()
            .find(|c| c.is_element() && c.tag_name().name() == "pic")
        {
            Some(n) => n,
            None => continue,
        };

        let from_node = anchor
            .children()
            .find(|c| c.is_element() && c.tag_name().name() == "from");
        let from_text = match from_node {
            Some(n) => &xml[n.range()],
            None => {
                warnings.push(
                    "Skipped a picture with no anchor position (missing <xdr:from>).".to_string(),
                );
                continue;
            }
        };

        // Width và Height đều optional: đủ cả 2 → ép cứng cả khối; chỉ 1 trong 2 → cạnh còn
        // lại tự tính theo đúng tỉ lệ khung hình gốc của chính ảnh đó (đọc từ file ảnh thật).
        let (cx, cy) = match (width_emu, height_emu_fixed) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => match decode_native_dimensions(pic_node, rels, archive) {
                Ok((nw, nh)) => (w, (w as f64 * nh as f64 / nw as f64).round() as i64),
                Err(msg) => {
                    warnings.push(msg);
                    continue;
                }
            },
            (None, Some(h)) => match decode_native_dimensions(pic_node, rels, archive) {
                Ok((nw, nh)) => ((h as f64 * nw as f64 / nh as f64).round() as i64, h),
                Err(msg) => {
                    warnings.push(msg);
                    continue;
                }
            },
            (None, None) => unreachable!("resize_drawing_pics is only called when a size was requested"),
        };

        // Nếu pic có <a:xfrm><a:ext cx cy/></a:xfrm> bên trong <xdr:spPr>, đồng bộ luôn giá trị này
        // để nội bộ shape khớp với kích thước anchor mới.
        let mut pic_xml = xml[pic_node.range()].to_string();
        if let Some(xfrm_node) = pic_node
            .descendants()
            .find(|n| n.is_element() && n.tag_name().name() == "xfrm")
        {
            if let Some(ext_node) = xfrm_node
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == "ext")
            {
                let pic_start = pic_node.range().start;
                let ext_range = ext_node.range();
                let ext_slice = &xml[ext_range.clone()];
                let patched = patch_cx_cy(ext_slice, cx, cy);
                let local_start = ext_range.start - pic_start;
                let local_end = ext_range.end - pic_start;
                pic_xml.replace_range(local_start..local_end, &patched);
            }
        }

        // Xác định vị trí gốc (top/bottom Y tuyệt đối) nếu có bảng row height của sheet —
        // dùng để đẩy ảnh xuống ở pass 2. Nếu không xác định được (thiếu row height, anchor
        // không đúng schema...), giữ nguyên vị trí gốc, chỉ đổi kích thước (như trước đây).
        let placement = row_heights.and_then(|rh| resolve_placement(&anchor, from_node?, rh));

        match placement {
            Some((col, col_off, orig_top_y, orig_bottom_y)) => {
                let (col, col_off) = match override_col {
                    Some(oc) => (oc, 0),
                    None => (col, col_off),
                };
                let anchor_range = anchor.range();
                tracked.push(TrackedPic {
                    anchor_start: anchor_range.start,
                    anchor_end: anchor_range.end,
                    col,
                    col_off,
                    orig_top_y,
                    orig_bottom_y,
                    new_cx: cx,
                    new_cy: cy,
                    pic_xml,
                });
            }
            None => {
                let final_from = match override_col {
                    Some(oc) => match from_node.and_then(parse_anchor_point) {
                        Some((_, _, row0, row_off)) => format!(
                            "<xdr:from><xdr:col>{oc}</xdr:col><xdr:colOff>0</xdr:colOff><xdr:row>{row0}</xdr:row><xdr:rowOff>{row_off}</xdr:rowOff></xdr:from>"
                        ),
                        None => {
                            warnings.push(
                                "Could not override the start column for a picture (unexpected anchor format); its original position was kept.".to_string(),
                            );
                            from_text.to_string()
                        }
                    },
                    None => from_text.to_string(),
                };
                let replacement = format!(
                    "<xdr:oneCellAnchor>{final_from}<xdr:ext cx=\"{cx}\" cy=\"{cy}\"/>{pic_xml}<xdr:clientData/></xdr:oneCellAnchor>"
                );
                let anchor_range = anchor.range();
                edits.push(Edit {
                    start: anchor_range.start,
                    end: anchor_range.end,
                    replacement,
                });
            }
        }
        resized_count += 1;
    }

    if row_heights.is_none() && resized_count > 0 {
        warnings.push(
            "Could not determine this sheet's row heights — picture positions were left as-is (only size was changed), so pictures may still overlap.".to_string(),
        );
    }

    // Pass 2: nhóm theo cột, sắp theo vị trí gốc từ trên xuống, đẩy ảnh sau xuống nếu cần
    // để không đè lên ảnh trước, luôn giữ nguyên khoảng cách gốc giữa 2 ảnh liền kề. Nếu ảnh
    // phình to sẽ đè lên 1 dòng có nội dung nằm giữa nó và ảnh kế tiếp: chèn 1 dòng mới để
    // nhường chỗ (nếu an toàn) hoặc giới hạn chiều cao ảnh đó lại (nếu không an toàn).
    let mut by_col: HashMap<i64, Vec<TrackedPic>> = HashMap::new();
    for pic in tracked {
        by_col.entry(pic.col).or_default().push(pic);
    }
    let mut all_insertions: Vec<RowInsertion> = Vec::new();
    if !by_col.is_empty() {
        let rh = row_heights.expect("tracked pics only exist when row_heights is Some");
        for (_, mut group) in by_col {
            group.sort_by_key(|p| p.orig_top_y);
            let mut extra_row_shift: u32 = 0;
            let mut extra_y_shift: i64 = 0;
            let mut prev_new_bottom: Option<i64> = None;
            let mut prev_orig_bottom: Option<i64> = None;

            for i in 0..group.len() {
                // Vị trí/số dòng của CHÍNH ảnh này phải dùng shift TRƯỚC khi xét collision ở
                // gap phía sau nó — 1 dòng mới được chèn ở gap sau ảnh i chỉ ảnh hưởng ảnh
                // i+1 trở đi, không lùi lại ảnh hưởng đến chính ảnh i.
                let shift_row_before = extra_row_shift;
                let shift_y_before = extra_y_shift;

                let new_top_y = match prev_new_bottom {
                    None => group[i].orig_top_y + shift_y_before,
                    Some(prev_new_bottom) => {
                        let gap = (group[i].orig_top_y - prev_orig_bottom.unwrap()).max(0);
                        prev_new_bottom + gap
                    }
                };
                let mut new_cy = group[i].new_cy;
                let mut new_bottom_y = new_top_y + new_cy;

                if avoid_covering_content {
                    if let Some(sheet_xml) = sheet_xml {
                        if let Some(next) = group.get(i + 1) {
                            let gap_start_row = rh.locate(group[i].orig_bottom_y).0 + 1;
                            let gap_end_row = rh.locate(next.orig_top_y).0 + 1;
                            if let Some(&content_row) =
                                find_content_rows(sheet_xml, gap_start_row, gap_end_row).first()
                            {
                                let content_top_adjusted =
                                    rh.y_of(content_row - 1, 0) + extra_y_shift;
                                if new_bottom_y > content_top_adjusted {
                                    let overlap = new_bottom_y - content_top_adjusted;
                                    if row_insertion_safe {
                                        // Lấy chiều cao dòng ngay trước content_row làm "template"
                                        // (thay vì chèn 1 dòng cao bất thường) — số dòng cần chèn
                                        // = làm tròn lên overlap / chiều cao template.
                                        let template_row = content_row.saturating_sub(1);
                                        let template_height_emu = if template_row >= 1 {
                                            rh.height_of(template_row)
                                        } else {
                                            rh.default_emu
                                        };
                                        let template_height_emu = if template_height_emu > 0.0 {
                                            template_height_emu
                                        } else {
                                            rh.default_emu
                                        };
                                        let count = ((overlap as f64) / template_height_emu).ceil().max(1.0) as u32;
                                        let height_pt = template_height_emu / EMU_PER_POINT;
                                        all_insertions.push(RowInsertion {
                                            at_row: content_row,
                                            height_pt,
                                            count,
                                        });
                                        extra_row_shift += count;
                                        extra_y_shift += (count as f64 * template_height_emu).round() as i64;
                                        warnings.push(format!(
                                            "Inserted {count} new row(s) (~{height_pt:.2}pt each, based on row {template_row}'s height) before row {content_row} so a resized picture wouldn't cover its content."
                                        ));
                                    } else {
                                        new_cy -= overlap;
                                        new_bottom_y = content_top_adjusted;
                                        warnings.push(format!(
                                            "A picture's height was reduced to avoid covering row {content_row}'s content (workbook has formulas/tables/charts, so automatic row insertion was disabled)."
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                let orig_equiv_top = new_top_y - shift_y_before;
                let (orig_row0, row_off) = rh.locate(orig_equiv_top);
                let final_row0 = orig_row0 + shift_row_before;

                prev_new_bottom = Some(new_bottom_y);
                prev_orig_bottom = Some(group[i].orig_bottom_y);

                let from_block = format!(
                    "<xdr:from><xdr:col>{}</xdr:col><xdr:colOff>{}</xdr:colOff><xdr:row>{final_row0}</xdr:row><xdr:rowOff>{row_off}</xdr:rowOff></xdr:from>",
                    group[i].col, group[i].col_off
                );
                let replacement = format!(
                    "<xdr:oneCellAnchor>{from_block}<xdr:ext cx=\"{}\" cy=\"{new_cy}\"/>{}<xdr:clientData/></xdr:oneCellAnchor>",
                    group[i].new_cx, group[i].pic_xml
                );
                edits.push(Edit {
                    start: group[i].anchor_start,
                    end: group[i].anchor_end,
                    replacement,
                });
            }
        }
    }

    // Nếu có dòng được chèn: dịch số dòng của MỌI anchor khác trong drawing part này (shape,
    // textbox, chart, group, ảnh bị bỏ qua...) mà chưa có edit riêng — vị trí của chúng vẫn
    // không đổi tương đối so với nội dung sheet sau khi dòng mới được chèn vào.
    let new_sheet_xml = if !all_insertions.is_empty() {
        let already_edited: Vec<(usize, usize)> = edits.iter().map(|e| (e.start, e.end)).collect();
        edits.extend(shift_untouched_anchors(xml, &all_insertions, &already_edited));
        match sheet_xml {
            Some(sheet_xml) => Some(apply_row_insertions(sheet_xml, &all_insertions)?),
            None => None,
        }
    } else {
        None
    };

    let new_drawing_xml = apply_edits(xml, edits);

    Ok(DrawingResizeOutcome {
        new_drawing_xml,
        resized_count,
        warnings,
        new_sheet_xml,
        insertions: all_insertions,
    })
}

/// Tính `(col, colOff, orig_top_y, orig_bottom_y)` (toạ độ Y tuyệt đối, EMU) của 1 anchor,
/// dựa trên `<xdr:from>` (và `<xdr:to>` nếu là `twoCellAnchor`, hoặc `<xdr:ext>` nếu đã là
/// `oneCellAnchor`). Trả về `None` nếu anchor không đúng schema mong đợi.
fn resolve_placement(
    anchor: &Node<'_, '_>,
    from_node: Node<'_, '_>,
    row_heights: &RowHeights,
) -> Option<(i64, i64, i64, i64)> {
    let (col, col_off, row0, row_off) = parse_anchor_point(from_node)?;
    let orig_top_y = row_heights.y_of(row0, row_off);

    let orig_bottom_y = if anchor.tag_name().name() == "twoCellAnchor" {
        let to_node = anchor
            .children()
            .find(|c| c.is_element() && c.tag_name().name() == "to")?;
        let (_, _, to_row0, to_row_off) = parse_anchor_point(to_node)?;
        row_heights.y_of(to_row0, to_row_off)
    } else {
        let ext_node = anchor
            .children()
            .find(|c| c.is_element() && c.tag_name().name() == "ext")?;
        let orig_cy: i64 = ext_node.attribute("cy")?.parse().ok()?;
        orig_top_y + orig_cy
    };

    Some((col, col_off, orig_top_y, orig_bottom_y.max(orig_top_y)))
}

/// Parse `<xdr:col>`/`<xdr:colOff>`/`<xdr:row>`/`<xdr:rowOff>` từ 1 node `from`/`to`.
fn parse_anchor_point(node: Node<'_, '_>) -> Option<(i64, i64, u32, i64)> {
    let get = |tag: &str| -> Option<i64> {
        node.children()
            .find(|c| c.is_element() && c.tag_name().name() == tag)
            .and_then(|c| c.text())
            .and_then(|t| t.trim().parse::<i64>().ok())
    };
    let col = get("col")?;
    let col_off = get("colOff")?;
    let row = get("row")?;
    let row_off = get("rowOff")?;
    if row < 0 {
        return None;
    }
    Some((col, col_off, row as u32, row_off))
}

/// Parse `<sheetFormatPr defaultRowHeight="…">` + mọi `<row r="N" ht="…">` của 1 worksheet
/// thành bảng row height (đơn vị EMU).
fn parse_row_heights(xml: &str) -> Result<RowHeights, roxmltree::Error> {
    let doc = Document::parse(xml)?;

    let default_pt = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "sheetFormatPr")
        .and_then(|n| n.attribute("defaultRowHeight"))
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(DEFAULT_ROW_HEIGHT_PT);

    let mut custom = HashMap::new();
    for row_node in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "row")
    {
        if let (Some(r), Some(ht)) = (
            row_node.attribute("r").and_then(|v| v.parse::<u32>().ok()),
            row_node.attribute("ht").and_then(|v| v.parse::<f64>().ok()),
        ) {
            custom.insert(r, ht * EMU_PER_POINT);
        }
    }

    Ok(RowHeights {
        default_emu: default_pt * EMU_PER_POINT,
        custom,
    })
}

/// Đọc kích thước pixel gốc (natural width/height) của ảnh mà 1 `<xdr:pic>` tham chiếu tới,
/// dùng để tính cạnh còn lại theo đúng tỉ lệ khung hình khi chỉ Width hoặc chỉ Height được set.
/// Trả về `Err(message)` (dùng làm warning, bỏ qua ảnh đó) nếu ảnh liên kết ngoài, không tìm
/// thấy relationship, không đọc được media file, hoặc không decode được.
fn decode_native_dimensions(
    pic_node: Node<'_, '_>,
    rels: &HashMap<String, String>,
    archive: &mut ZipArchive<File>,
) -> Result<(u32, u32), String> {
    let embed_id = pic_node
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "blip")
        .and_then(|n| n.attributes().find(|a| a.name() == "embed"))
        .map(|a| a.value().to_string())
        .ok_or_else(|| {
            "Skipped a picture with no embedded image reference (externally linked image).".to_string()
        })?;
    let target = rels
        .get(&embed_id)
        .ok_or_else(|| format!("Skipped a picture: relationship {embed_id} not found."))?;
    let media_path = resolve_media_path(target);
    let bytes = read_entry_bytes(archive, &media_path).map_err(|e| format!("Skipped {media_path}: {e}"))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Skipped {media_path}: could not decode image ({e})."))?;
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Err(format!("Skipped {media_path}: invalid image dimensions."));
    }
    Ok((w, h))
}

/// Thay giá trị `cx`/`cy` trong một đoạn XML `<a:ext .../>`, giữ nguyên phần còn lại.
fn patch_cx_cy(xml_fragment: &str, cx: i64, cy: i64) -> String {
    let cx_re = Regex::new(r#"cx="\d+""#).expect("valid regex");
    let cy_re = Regex::new(r#"cy="\d+""#).expect("valid regex");
    let replaced = cx_re.replace(xml_fragment, format!(r#"cx="{cx}""#).as_str());
    let replaced = cy_re.replace(&replaced, format!(r#"cy="{cy}""#).as_str());
    replaced.into_owned()
}

/// Parse các chữ cái đứng đầu của 1 cell reference (ví dụ "B2" → "B", "AA10" → "AA") thành
/// chỉ số cột 0-based theo kiểu Excel (A=0, B=1, ..., Z=25, AA=26, ...). Phần số (row) bị bỏ qua.
/// Trả về `None` nếu không có chữ cái nào ở đầu.
fn parse_column_letters(input: &str) -> Option<i64> {
    let letters: String = input
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect();
    if letters.is_empty() {
        return None;
    }
    let mut col: i64 = 0;
    for c in letters.to_ascii_uppercase().chars() {
        col = col * 26 + (c as i64 - 'A' as i64 + 1);
    }
    Some(col - 1)
}

/// Thay giá trị của attribute `attr` trong 1 đoạn XML (ví dụ `val="…"`), giữ nguyên phần còn lại.
fn patch_attr_value(xml_fragment: &str, attr: &str, new_value: &str) -> String {
    let re = Regex::new(&format!(r#"{attr}="[^"]*""#)).expect("valid regex");
    re.replace(xml_fragment, format!(r#"{attr}="{new_value}""#).as_str())
        .into_owned()
}

/// Set zoom (%) và/hoặc chế độ Page Break Preview trên `<sheetView>` đầu tiên của 1 worksheet.
/// Chỉ sửa phần opening tag (attributes), không đụng vào `<pane>`/`<selection>` bên trong nếu có.
/// Trả về `None` nếu không tìm thấy `<sheetView>` nào.
fn patch_sheet_view(xml: &str, zoom_percent: Option<u32>, page_break_preview: bool) -> Option<String> {
    let doc = Document::parse(xml).ok()?;
    let node = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "sheetView")?;

    let range = node.range();
    let head = &xml[range.clone()];
    let head_end_rel = head.find('>')? + 1;
    let is_self_closing = head[..head_end_rel].ends_with("/>");

    let mut attrs: Vec<(String, String)> = node
        .attributes()
        .map(|a| (a.name().to_string(), a.value().to_string()))
        .collect();

    if let Some(z) = zoom_percent {
        upsert_attr(&mut attrs, "zoomScale", z.to_string());
    }
    if page_break_preview {
        upsert_attr(&mut attrs, "view", "pageBreakPreview".to_string());
    }

    let attrs_str = attrs
        .iter()
        .map(|(k, v)| format!(r#"{k}="{v}""#))
        .collect::<Vec<_>>()
        .join(" ");
    let new_head = if is_self_closing {
        format!("<sheetView {attrs_str}/>")
    } else {
        format!("<sheetView {attrs_str}>")
    };

    let mut result = xml.to_string();
    result.replace_range(range.start..range.start + head_end_rel, &new_head);
    Some(result)
}

fn upsert_attr(attrs: &mut Vec<(String, String)>, name: &str, value: String) {
    match attrs.iter_mut().find(|(k, _)| k == name) {
        Some(existing) => existing.1 = value,
        None => attrs.push((name.to_string(), value)),
    }
}

/// Đổi font mặc định của workbook: patch `<name>`/`<sz>` của font index 0 trong `<fonts>`
/// (font mà `cellStyleXfs`'s "Normal" style trỏ tới, áp dụng cho mọi cell chưa set font riêng).
/// Trả về `None` nếu không tìm thấy `<fonts>`/font đầu tiên.
fn patch_default_font(xml: &str, font_name: Option<&str>, font_size: Option<f64>) -> Option<String> {
    let doc = Document::parse(xml).ok()?;
    let font0 = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "fonts")?
        .children()
        .find(|c| c.is_element() && c.tag_name().name() == "font")?;

    let font_range = font0.range();
    let mut font_xml = xml[font_range.clone()].to_string();

    // Áp dụng theo thứ tự từ cuối văn bản về đầu để offset của các phần chưa sửa không đổi.
    // `<sz>`/`<name>` đã tồn tại thì patch tại chỗ (2 range riêng biệt, không đụng nhau); phần
    // nào còn thiếu thì gom chung vào 1 chuỗi chèn duy nhất (theo đúng thứ tự schema sz→name)
    // để tránh 2 edit "chèn" trùng vị trí (không xác định được thứ tự chèn trước/sau).
    let mut local_edits: Vec<(usize, usize, String)> = Vec::new();
    let mut missing_insert = String::new();

    if let Some(size) = font_size {
        let value = format_trimmed(size);
        match font0
            .children()
            .find(|c| c.is_element() && c.tag_name().name() == "sz")
        {
            Some(sz_node) => {
                let r = sz_node.range();
                let patched = patch_attr_value(&xml[r.clone()], "val", &value);
                local_edits.push((r.start - font_range.start, r.end - font_range.start, patched));
            }
            None => missing_insert.push_str(&format!(r#"<sz val="{value}"/>"#)),
        }
    }

    if let Some(name) = font_name {
        match font0
            .children()
            .find(|c| c.is_element() && c.tag_name().name() == "name")
        {
            Some(name_node) => {
                let r = name_node.range();
                let patched = patch_attr_value(&xml[r.clone()], "val", name);
                local_edits.push((r.start - font_range.start, r.end - font_range.start, patched));
            }
            None => missing_insert.push_str(&format!(r#"<name val="{name}"/>"#)),
        }
    }

    if !missing_insert.is_empty() {
        let insert_at = insertion_point(&font_xml);
        local_edits.push((insert_at, insert_at, missing_insert));
    }

    local_edits.sort_by(|a, b| b.0.cmp(&a.0));
    for (start, end, replacement) in local_edits {
        font_xml.replace_range(start..end, &replacement);
    }

    let mut result = xml.to_string();
    result.replace_range(font_range, &font_xml);
    Some(result)
}

/// Vị trí (byte offset, tương đối trong `font_xml`) ngay sau tag mở `<font ...>` — nơi chèn
/// thêm 1 phần tử con mới (ví dụ `<name>`/`<sz>` khi chưa tồn tại).
fn insertion_point(font_xml: &str) -> usize {
    font_xml.find('>').map(|i| i + 1).unwrap_or(font_xml.len())
}

fn format_trimmed(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// 1 nhóm dòng mới được chèn vào sheet để nhường chỗ cho ảnh phình to, tránh đè lên nội dung
/// cell. `at_row` là số dòng gốc (1-based, theo `<row r="…">` TRƯỚC khi chèn) — các dòng mới
/// được chèn ngay trước dòng đó. `height_pt` là chiều cao của MỖI dòng mới (lấy từ dòng ngay
/// trước `at_row` làm "template", không phải 1 dòng cao bất thường) và `count` là số dòng cần
/// chèn để tổng chiều cao vừa đủ lấp khoảng đè (làm tròn lên).
#[derive(Clone, Copy)]
struct RowInsertion {
    at_row: u32,
    height_pt: f64,
    count: u32,
}

/// Tổng số dòng đã được chèn vào TRƯỚC (hoặc đúng tại) dòng gốc `orig_row_1based` — dùng để quy
/// đổi 1 số dòng gốc sang số dòng cuối cùng (sau khi chèn): `final = orig_row_1based + shift_count(...)`.
fn shift_count(insertions: &[RowInsertion], orig_row_1based: u32) -> u32 {
    insertions
        .iter()
        .filter(|i| i.at_row <= orig_row_1based)
        .map(|i| i.count)
        .sum()
}

/// Quét toàn bộ workbook xem có an toàn để chèn dòng (đánh số lại) hay không: KHÔNG an toàn nếu
/// có bất kỳ formula nào (ở bất kỳ sheet nào — kể cả tham chiếu chéo sheet), hoặc có table/
/// chart/pivot table — vì chèn dòng đòi hỏi phải viết lại tham chiếu, và tự động viết lại công
/// thức một cách chắc chắn đúng là ngoài phạm vi an toàn của tool này.
fn workbook_supports_row_insertion(
    entry_names: &[String],
    archive: &mut ZipArchive<File>,
) -> AppResult<bool> {
    let has_risky_parts = entry_names.iter().any(|n| {
        n.starts_with("xl/tables/")
            || n.starts_with("xl/charts/")
            || n.starts_with("xl/pivotTables/")
            || n.starts_with("xl/pivotCache")
    });
    if has_risky_parts {
        return Ok(false);
    }

    for sheet_name in entry_names
        .iter()
        .filter(|n| n.starts_with("xl/worksheets/sheet") && n.ends_with(".xml") && !n.contains("_rels"))
    {
        let xml = read_entry_to_string(archive, sheet_name)?;
        if sheet_has_formula(&xml) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Có formula (`<f>`) ở bất kỳ đâu trong sheet không. Lỗi parse coi như "có" (an toàn, thận trọng).
fn sheet_has_formula(xml: &str) -> bool {
    match Document::parse(xml) {
        Ok(doc) => doc
            .descendants()
            .any(|n| n.is_element() && n.tag_name().name() == "f"),
        Err(_) => true,
    }
}

/// Tìm các dòng (1-based) trong khoảng `[from_row, to_row]` (2 đầu bao gồm) có nội dung thật sự —
/// tức có ít nhất 1 `<c>` chứa `<v>`/`<is>`/`<f>` (cell chỉ có style, không giá trị, không tính).
fn find_content_rows(sheet_xml: &str, from_row_1based: u32, to_row_1based: u32) -> Vec<u32> {
    if from_row_1based > to_row_1based {
        return Vec::new();
    }
    let doc = match Document::parse(sheet_xml) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let mut rows: Vec<u32> = doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "row")
        .filter_map(|row_node| {
            let r = row_node.attribute("r").and_then(|v| v.parse::<u32>().ok())?;
            if r < from_row_1based || r > to_row_1based {
                return None;
            }
            let has_content = row_node.children().any(|c| {
                c.is_element()
                    && c.tag_name().name() == "c"
                    && c.children()
                        .any(|cc| cc.is_element() && matches!(cc.tag_name().name(), "v" | "is" | "f"))
            });
            has_content.then_some(r)
        })
        .collect();
    rows.sort_unstable();
    rows
}

/// Tìm sheet (tên part, ví dụ `xl/worksheets/sheet3.xml`) sở hữu 1 drawing part, qua
/// `xl/worksheets/_rels/sheetK.xml.rels`. Trả về `None` nếu không tìm được (drawing part mồ côi).
fn find_owning_sheet_name(
    drawing_name: &str,
    entry_names: &[String],
    archive: &mut ZipArchive<File>,
) -> AppResult<Option<String>> {
    for rels_name in entry_names
        .iter()
        .filter(|n| n.starts_with("xl/worksheets/_rels/") && n.ends_with(".xml.rels"))
    {
        let rels_xml = read_entry_to_string(archive, rels_name)?;
        let rels_map = parse_rels(&rels_xml)
            .map_err(|e| AppError::new(format!("Could not parse {rels_name}: {e}")))?;
        let owns_drawing = rels_map
            .values()
            .any(|target| join_relative(&["xl", "worksheets"], target) == drawing_name);
        if !owns_drawing {
            continue;
        }

        let sheet_file = rels_name
            .trim_start_matches("xl/worksheets/_rels/")
            .trim_end_matches(".rels");
        let sheet_name = format!("xl/worksheets/{sheet_file}");
        if entry_names.iter().any(|n| n == &sheet_name) {
            return Ok(Some(sheet_name));
        }
    }
    Ok(None)
}

/// Đánh số lại 1 cell reference kiểu `"A79"` theo hàm `shift` (chỉ đổi phần số/dòng, giữ nguyên
/// phần chữ/cột). Trả về nguyên văn nếu không parse được (không đúng định dạng cell ref).
fn shift_cell_ref(cell_ref: &str, insertions: &[RowInsertion]) -> String {
    let split_at = cell_ref.find(|c: char| c.is_ascii_digit());
    let Some(split_at) = split_at else {
        return cell_ref.to_string();
    };
    let (letters, digits) = cell_ref.split_at(split_at);
    let Ok(row) = digits.parse::<u32>() else {
        return cell_ref.to_string();
    };
    format!("{letters}{}", row + shift_count(insertions, row))
}

/// Đánh số lại 1 chuỗi range/list range cách nhau bởi khoảng trắng (dùng cho `mergeCell@ref`,
/// `hyperlink@ref`, `dataValidation@sqref`, `autoFilter@ref`) — mỗi phần tử có thể là 1 cell
/// (`"A5"`) hoặc 1 range (`"A5:B79"`); giữ nguyên nếu không parse được phần nào.
fn shift_range_refs(value: &str, insertions: &[RowInsertion]) -> String {
    value
        .split_whitespace()
        .map(|part| {
            part.split(':')
                .map(|cell| shift_cell_ref(cell, insertions))
                .collect::<Vec<_>>()
                .join(":")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Patch attribute `attr_name` của 1 node (nếu có) bằng `shift_range_refs`, ghi vào `edits` nếu
/// giá trị thực sự đổi. Chỉ sửa opening tag của node, không đụng con cháu.
fn patch_ref_attr(
    xml: &str,
    node: Node<'_, '_>,
    attr_name: &str,
    insertions: &[RowInsertion],
    edits: &mut Vec<Edit>,
) {
    let Some(attr) = node.attributes().find(|a| a.name() == attr_name) else {
        return;
    };
    let new_value = shift_range_refs(attr.value(), insertions);
    if new_value == attr.value() {
        return;
    }
    let range = node.range();
    let head_text = &xml[range.clone()];
    let Some(head_end) = head_text.find('>').map(|i| i + 1) else {
        return;
    };
    let patched_head = patch_attr_value(&head_text[..head_end], attr_name, &new_value);
    edits.push(Edit {
        start: range.start,
        end: range.start + head_end,
        replacement: patched_head,
    });
}

/// Viết lại 1 worksheet part để chèn các dòng mới trong `insertions`: đánh số lại mọi
/// `<row r>`/`<c r>` từ điểm chèn trở xuống, dịch các range tham chiếu (`mergeCells`,
/// `hyperlinks`, `dataValidations`, `autoFilter`), rồi chèn các `<row>` mới (rỗng, chỉ có
/// chiều cao) vào đúng vị trí đã sắp xếp trong `sheetData`.
fn apply_row_insertions(sheet_xml: &str, insertions: &[RowInsertion]) -> AppResult<String> {
    if insertions.is_empty() {
        return Ok(sheet_xml.to_string());
    }
    let doc = Document::parse(sheet_xml)
        .map_err(|e| AppError::new(format!("Could not parse worksheet XML: {e}")))?;

    let mut sorted_insertions = insertions.to_vec();
    sorted_insertions.sort_by_key(|i| i.at_row);

    let mut edits: Vec<Edit> = Vec::new();

    // Đánh số lại <row r="N"> và mọi <c r="XN"> con của nó.
    let all_rows: Vec<Node> = doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "row")
        .collect();
    for row_node in &all_rows {
        if let Some(r) = row_node.attribute("r").and_then(|v| v.parse::<u32>().ok()) {
            let shift = shift_count(&sorted_insertions, r);
            if shift > 0 {
                let range = row_node.range();
                let head_text = &sheet_xml[range.clone()];
                if let Some(head_end) = head_text.find('>').map(|i| i + 1) {
                    let patched_head = patch_attr_value(&head_text[..head_end], "r", &(r + shift).to_string());
                    edits.push(Edit {
                        start: range.start,
                        end: range.start + head_end,
                        replacement: patched_head,
                    });
                }
            }
        }
        for c_node in row_node
            .children()
            .filter(|c| c.is_element() && c.tag_name().name() == "c")
        {
            let Some(r_attr) = c_node.attribute("r") else { continue };
            let new_ref = shift_cell_ref(r_attr, &sorted_insertions);
            if new_ref == r_attr {
                continue;
            }
            let range = c_node.range();
            let head_text = &sheet_xml[range.clone()];
            if let Some(head_end) = head_text.find('>').map(|i| i + 1) {
                let patched_head = patch_attr_value(&head_text[..head_end], "r", &new_ref);
                edits.push(Edit {
                    start: range.start,
                    end: range.start + head_end,
                    replacement: patched_head,
                });
            }
        }
    }

    // Dịch các range tham chiếu tới cell/row (mergeCells, hyperlinks, dataValidations, autoFilter).
    for node in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "mergeCell")
    {
        patch_ref_attr(sheet_xml, node, "ref", &sorted_insertions, &mut edits);
    }
    for node in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "hyperlink")
    {
        patch_ref_attr(sheet_xml, node, "ref", &sorted_insertions, &mut edits);
    }
    for node in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "dataValidation")
    {
        patch_ref_attr(sheet_xml, node, "sqref", &sorted_insertions, &mut edits);
    }
    for node in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "autoFilter")
    {
        patch_ref_attr(sheet_xml, node, "ref", &sorted_insertions, &mut edits);
    }

    // Chèn các <row> mới (rỗng, không có cell) vào đúng vị trí trong sheetData, ngay trước
    // dòng gốc đầu tiên có r >= at_row (hoặc trước </sheetData> nếu không còn dòng nào sau đó).
    if let Some(sheet_data) = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "sheetData")
    {
        let end_of_sheet_data = sheet_data.range().end;
        let insert_before_close = end_of_sheet_data.saturating_sub("</sheetData>".len());

        let mut inserted_so_far: u32 = 0;
        for ins in &sorted_insertions {
            let height_str = format!("{:.2}", ins.height_pt.max(0.01));
            let mut new_rows_xml = String::new();
            for j in 0..ins.count {
                let final_row = ins.at_row + inserted_so_far + j;
                new_rows_xml.push_str(&format!(r#"<row r="{final_row}" ht="{height_str}" customHeight="1"/>"#));
            }

            let insert_at = all_rows
                .iter()
                .find(|r| {
                    r.attribute("r")
                        .and_then(|v| v.parse::<u32>().ok())
                        .is_some_and(|rn| rn >= ins.at_row)
                })
                .map(|r| r.range().start)
                .unwrap_or(insert_before_close);

            edits.push(Edit {
                start: insert_at,
                end: insert_at,
                replacement: new_rows_xml,
            });
            inserted_so_far += ins.count;
        }
    }

    Ok(apply_edits(sheet_xml, edits))
}

/// Dịch số dòng (`<xdr:row>` trong `from`/`to`) của MỌI anchor (pic/shape/chart/group) trong 1
/// drawing part có row bị ảnh hưởng bởi `insertions` — trừ những anchor đã có edit riêng
/// (`already_edited`, theo byte range) vì những anchor đó đã tự tính đúng số dòng cuối cùng rồi.
/// Không đụng đến nội dung/kích thước của anchor, chỉ số dòng neo.
fn shift_untouched_anchors(
    xml: &str,
    insertions: &[RowInsertion],
    already_edited: &[(usize, usize)],
) -> Vec<Edit> {
    let doc = match Document::parse(xml) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let mut edits = Vec::new();
    for anchor in doc.descendants().filter(|n| {
        n.is_element() && matches!(n.tag_name().name(), "twoCellAnchor" | "oneCellAnchor")
    }) {
        let range = anchor.range();
        if already_edited
            .iter()
            .any(|(s, e)| *s < range.end && range.start < *e)
        {
            continue;
        }
        for point_tag in ["from", "to"] {
            let Some(point_node) = anchor
                .children()
                .find(|c| c.is_element() && c.tag_name().name() == point_tag)
            else {
                continue;
            };
            let Some(row_node) = point_node
                .children()
                .find(|c| c.is_element() && c.tag_name().name() == "row")
            else {
                continue;
            };
            let Some(text_node) = row_node.children().find(|c| c.is_text()) else {
                continue;
            };
            let Some(row0) = text_node.text().and_then(|t| t.trim().parse::<u32>().ok()) else {
                continue;
            };
            let shift = shift_count(insertions, row0 + 1);
            if shift > 0 {
                let text_range = text_node.range();
                edits.push(Edit {
                    start: text_range.start,
                    end: text_range.end,
                    replacement: (row0 + shift).to_string(),
                });
            }
        }
    }
    edits
}

/// Tìm tên hiển thị (ví dụ "修正後") của 1 sheet, dựa trên tên part của nó (ví dụ
/// `xl/worksheets/sheet3.xml`) — qua `xl/workbook.xml` (`<sheets><sheet name r:id>`) đối chiếu
/// với `xl/_rels/workbook.xml.rels` (`r:id -> Target`).
fn resolve_sheet_display_name(workbook_xml: &str, rels_xml: &str, sheet_file_name: &str) -> Option<String> {
    let rels_map = parse_rels(rels_xml).ok()?;
    let doc = Document::parse(workbook_xml).ok()?;
    doc.descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "sheet")
        .find_map(|sheet_node| {
            let rid = sheet_node.attributes().find(|a| a.name() == "id")?.value();
            let target = rels_map.get(rid)?;
            let resolved = join_relative(&["xl"], target);
            (resolved == sheet_file_name)
                .then(|| sheet_node.attribute("name").map(str::to_string))
                .flatten()
        })
}

/// Dịch số dòng trong 1 giá trị `definedName` dạng `SheetName!$A$1:$H$79` (chấp nhận tên sheet
/// có ngoặc đơn `'...'`) nếu tên sheet khớp `sheet_name`. Trả về `None` (bỏ qua, không đoán) nếu
/// tham chiếu tới sheet khác, là union nhiều range (chứa dấu phẩy), hoặc không đúng định dạng
/// `$COL$ROW` đơn giản.
fn shift_defined_name_range(value: &str, sheet_name: &str, insertions: &[RowInsertion]) -> Option<String> {
    let trimmed = value.trim();
    let (sheet_part, range_part) = trimmed.split_once('!')?;
    if sheet_part.trim().trim_matches('\'') != sheet_name {
        return None;
    }
    if range_part.contains(',') {
        return None;
    }
    let re = Regex::new(r"^(\$?)([A-Za-z]+)(\$?)(\d+)$").ok()?;
    let shift_one = |part: &str| -> Option<String> {
        let caps = re.captures(part)?;
        let row: u32 = caps[4].parse().ok()?;
        let new_row = row + shift_count(insertions, row);
        Some(format!("{}{}{}{}", &caps[1], &caps[2], &caps[3], new_row))
    };
    let mut new_parts = Vec::new();
    for part in range_part.split(':') {
        new_parts.push(shift_one(part)?);
    }
    Some(format!("{sheet_part}!{}", new_parts.join(":")))
}

/// Dịch mọi `definedName` (ví dụ `Print_Area`) tham chiếu tới `sheet_display_name` trong
/// `xl/workbook.xml`, theo `insertions`. Trả về `None` nếu không có gì thay đổi.
fn patch_defined_names(workbook_xml: &str, sheet_display_name: &str, insertions: &[RowInsertion]) -> Option<String> {
    let doc = Document::parse(workbook_xml).ok()?;
    let mut edits = Vec::new();
    for dn_node in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "definedName")
    {
        let Some(text_node) = dn_node.children().find(|c| c.is_text()) else {
            continue;
        };
        let Some(text) = text_node.text() else { continue };
        if let Some(new_value) = shift_defined_name_range(text, sheet_display_name, insertions) {
            let r = text_node.range();
            edits.push(Edit {
                start: r.start,
                end: r.end,
                replacement: new_value,
            });
        }
    }
    if edits.is_empty() {
        None
    } else {
        Some(apply_edits(workbook_xml, edits))
    }
}

/// Ghép `target` (đường dẫn tương đối trong file `.rels`) với thư mục `base` để ra đường dẫn
/// part tuyệt đối trong file zip (ví dụ base `["xl","drawings"]` + `../media/image1.png` →
/// `xl/media/image1.png`).
fn join_relative(base: &[&str], target: &str) -> String {
    let mut segments: Vec<&str> = base.to_vec();
    for part in target.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }
    segments.join("/")
}

/// Ghép target trong `.rels` của 1 drawing part (base `xl/drawings`) — dùng để tìm media file.
fn resolve_media_path(target: &str) -> String {
    join_relative(&["xl", "drawings"], target)
}

/// Parse file `.rels` thành map `relationship id -> target`.
fn parse_rels(xml: &str) -> Result<HashMap<String, String>, roxmltree::Error> {
    let doc = Document::parse(xml)?;
    let mut map = HashMap::new();
    for node in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "Relationship")
    {
        if let (Some(id), Some(target)) = (node.attribute("Id"), node.attribute("Target")) {
            map.insert(id.to_string(), target.to_string());
        }
    }
    Ok(map)
}

/// Tên part `.rels` tương ứng với một drawing part.
fn drawing_rels_name(drawing_name: &str) -> String {
    let name = drawing_name.rsplit('/').next().unwrap_or(drawing_name);
    format!("xl/drawings/_rels/{name}.rels")
}

fn read_entry_to_string(archive: &mut ZipArchive<File>, name: &str) -> AppResult<String> {
    let bytes = read_entry_bytes(archive, name)?;
    String::from_utf8(bytes).map_err(|e| AppError::new(format!("Invalid UTF-8 in {name}: {e}")))
}

fn read_entry_bytes(archive: &mut ZipArchive<File>, name: &str) -> AppResult<Vec<u8>> {
    let mut entry = archive
        .by_name(name)
        .map_err(|e| AppError::new(format!("Could not read {name}: {e}")))?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Ghi lại toàn bộ workbook ra `output`: copy nguyên vẹn mọi entry, chỉ thay thế các
/// drawing part đã resize. Ghi ra file tạm trước, chỉ move đè lên `output` sau khi
/// zip mới đã ghi xong và archive nguồn đã được drop (an toàn cả khi output == input).
fn write_output_archive(
    archive: ZipArchive<File>,
    entry_names: &[String],
    replaced: &HashMap<String, Vec<u8>>,
    output: &Path,
) -> AppResult<()> {
    let mut archive = archive;
    let output_dir = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let temp_file_name = format!(
        ".{}.tmp",
        output
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("evidence_resize")
    );
    let temp_path = output_dir.join(temp_file_name);

    {
        let temp_file = File::create(&temp_path)?;
        let mut writer = ZipWriter::new(temp_file);

        for name in entry_names {
            if name.ends_with('/') {
                writer
                    .add_directory(name.clone(), SimpleFileOptions::default())
                    .map_err(|e| AppError::new(format!("Could not write directory {name}: {e}")))?;
                continue;
            }

            let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            writer
                .start_file(name.clone(), options)
                .map_err(|e| AppError::new(format!("Could not write {name}: {e}")))?;

            if let Some(bytes) = replaced.get(name) {
                writer.write_all(bytes)?;
            } else {
                let mut entry = archive
                    .by_name(name)
                    .map_err(|e| AppError::new(format!("Could not read {name}: {e}")))?;
                std::io::copy(&mut entry, &mut writer)?;
            }
        }

        writer
            .finish()
            .map_err(|e| AppError::new(format!("Could not finalize workbook archive: {e}")))?;
    }

    drop(archive);

    if output.exists() {
        std::fs::remove_file(output)?;
    }
    std::fs::rename(&temp_path, output)?;

    Ok(())
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string()
}
