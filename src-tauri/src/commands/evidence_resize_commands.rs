//! Tauri command handler cho chức năng resize ảnh evidence (hardcopy) trong Excel.

use crate::models::evidence_resize::{EvidenceResizeOptions, EvidenceResizeResult};
use crate::services::evidence_resize_service;

/// Resize toàn bộ ảnh (picture) trong workbook Excel, giữ nguyên shape/textbox.
///
/// `width_cm`/`height_cm` đều optional: chỉ 1 trong 2 → cạnh còn lại tự tính theo tỉ lệ
/// khung hình gốc của từng ảnh; cả hai → mọi ảnh có cùng kích thước Width x Height; không
/// có cái nào → không resize ảnh nào cả. `options` cấu hình thêm view/zoom, cột bắt đầu của
/// ảnh, và font mặc định của workbook.
#[tauri::command]
pub fn resize_evidence_images(
    input_path: String,
    output_path: String,
    width_cm: Option<f64>,
    height_cm: Option<f64>,
    options: EvidenceResizeOptions,
) -> Result<EvidenceResizeResult, String> {
    evidence_resize_service::resize_evidence_images(input_path, output_path, width_cm, height_cm, options)
        .map_err(crate::app::error::log_err)
}
