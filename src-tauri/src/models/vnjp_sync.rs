use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SheetMeta {
    pub name: String,
    pub tab_color: Option<String>,
    pub row_count: usize,
    pub col_count: usize,
    pub red_cell_count: usize,
    pub strike_cell_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RedCell {
    pub sheet: String,
    pub row: usize,
    pub col: usize,
    pub vn_text: String,
    pub jp_text: String,
    pub translation: Option<String>,
    /// `true` nếu ô này thực chất là 1 shape/textbox nổi (không phải cell) — khi đó
    /// `row`/`col` là vị trí Ô NEO (anchor) của shape, không phải nội dung cell thật.
    #[serde(default)]
    pub is_shape: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StrikeCell {
    pub sheet: String,
    pub row: usize,
    pub col: usize,
    pub text: String,
    /// `true` nếu ô này thực chất là 1 shape/textbox nổi — xem `RedCell::is_shape`.
    #[serde(default)]
    pub is_shape: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QualityIssue {
    pub sheet: String,
    pub row: usize,
    pub col: usize,
    pub issue_type: String,
    pub content: String,
    pub description: String,
    /// `true` nếu vấn đề này nằm trong 1 shape/textbox nổi — xem `RedCell::is_shape`.
    #[serde(default)]
    pub is_shape: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SheetCompare {
    pub name: String,
    pub in_vn: bool,
    pub in_jp: bool,
    pub vn_tab_color: Option<String>,
    pub jp_tab_color: Option<String>,
    pub vn_rows: usize,
    pub jp_rows: usize,
    pub vn_red_count: usize,
    pub jp_strike_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncAnalysis {
    pub vn_path: String,
    pub jp_path: String,
    pub vn_sheets: Vec<SheetMeta>,
    pub jp_sheets: Vec<SheetMeta>,
    pub sheet_compare: Vec<SheetCompare>,
    pub red_cells: Vec<RedCell>,
    pub strike_cells: Vec<StrikeCell>,
    pub quality_issues: Vec<QualityIssue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateItem {
    pub id: String,
    pub sheet: String,
    pub row: usize,
    pub col: usize,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateBatchRequest {
    pub items: Vec<TranslateItem>,
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateItemResult {
    pub id: String,
    pub translation: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub output_path: String,
    pub applied_count: usize,
    pub skipped_count: usize,
    pub sheets_modified: Vec<String>,
    pub strike_removed_count: usize,
    pub red_blackened_count: usize,
    pub cleanup_skipped_count: usize,
    pub column_corrected_count: usize,
    /// Số đoạn văn trong textbox/shape nổi đã ghi được vào đúng shape JP tương ứng (khớp theo tên shape).
    #[serde(default)]
    pub shape_applied_count: usize,
    /// Số đoạn văn shape VN bị bỏ qua vì không tìm thấy shape cùng tên trong JP.
    #[serde(default)]
    pub shape_skipped_count: usize,
    /// Số sheet chỉ có ở VN (tab màu quy ước) đã được clone nguyên trạng sang JP.
    #[serde(default)]
    pub cloned_sheet_count: usize,
    /// Số sheet JP có hậu tố "(DEL)" đã được xử lý (chỉ bỏ màu chữ về đen, không đụng gì khác).
    #[serde(default)]
    pub del_sheet_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub output_path: String,
    pub sheets_modified: Vec<String>,
    pub strike_removed_count: usize,
    pub red_blackened_count: usize,
    pub skipped_count: usize,
    /// Số sheet JP có hậu tố "(DEL)" đã được xử lý (chỉ bỏ màu chữ về đen, không đụng gì khác).
    #[serde(default)]
    pub del_sheet_count: usize,
}

/// Một vị trí VN có dòng mà JP chưa có (lệch dòng), đề xuất chèn thêm dòng trống vào JP.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RowAlignmentSuggestion {
    pub sheet: String,
    /// Số dòng JP (1-based) mà dòng mới sẽ được chèn NGAY SAU đó. 0 = chèn ở đầu sheet.
    pub jp_insert_after_row: usize,
    pub insert_count: usize,
    pub vn_row_start: usize,
    pub vn_row_end: usize,
    pub sample_vn_text: Vec<String>,
    pub has_red: bool,
    pub has_strike: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RowAlignmentReport {
    pub suggestions: Vec<RowAlignmentSuggestion>,
}

/// Một đề xuất chèn dòng đã được TL xác nhận, gửi lại để tool thực hiện chèn thật.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedInsert {
    pub sheet: String,
    pub jp_insert_after_row: usize,
    pub insert_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RowInsertResult {
    pub output_path: String,
    pub sheets_modified: Vec<String>,
    pub rows_inserted: usize,
}

/// Gợi ý vị trí JP khác có độ tương đồng cao hơn hẳn so với vị trí đang xét — dấu hiệu lệch dòng.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BetterMatch {
    pub row: usize,
    pub col: usize,
    pub similarity: f32,
}

/// Kết quả kiểm tra AI cho 1 ô đỏ: dịch VN→JP (chỉ dùng để so sánh, không ghi vào tài liệu)
/// rồi so độ tương đồng với nội dung JP hiện có.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RedCellVerification {
    pub sheet: String,
    pub row: usize,
    pub col: usize,
    pub ai_translation: String,
    pub similarity_same_pos: f32,
    pub better_match: Option<BetterMatch>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedCellVerificationReport {
    pub items: Vec<RedCellVerification>,
}
