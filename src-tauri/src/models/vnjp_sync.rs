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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StrikeCell {
    pub sheet: String,
    pub row: usize,
    pub col: usize,
    pub text: String,
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
}
