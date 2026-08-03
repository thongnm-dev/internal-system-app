//! Model cho công cụ so sánh khác biệt giữa 2 file
//! (Markdown, Text, Word `.docx`, Excel).

use serde::Serialize;

/// Kết quả so sánh trả về frontend. Tùy loại file mà `text_diff` hoặc
/// `excel_diff` được điền (loại còn lại là `None`).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareResult {
    /// Loại file chung: `"text" | "markdown" | "word" | "excel"`.
    pub kind: String,
    /// Diff theo dòng (text/markdown/word). `None` nếu là Excel.
    pub text_diff: Option<TextDiff>,
    /// Diff theo ô (Excel). `None` nếu là text/markdown/word.
    pub excel_diff: Option<ExcelDiff>,
}

/// Diff theo dòng cho text/markdown/word.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDiff {
    /// Danh sách dòng đã ghép diff (theo thứ tự hiển thị).
    pub lines: Vec<DiffLine>,
    /// Số dòng thêm.
    pub added: usize,
    /// Số dòng xóa.
    pub removed: usize,
}

/// Một dòng trong kết quả diff văn bản.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    /// `"equal" | "insert" | "delete"`.
    pub tag: String,
    /// Số dòng phía file A (1-based), `None` với dòng thêm mới.
    pub old_line: Option<usize>,
    /// Số dòng phía file B (1-based), `None` với dòng bị xóa.
    pub new_line: Option<usize>,
    /// Nội dung dòng (đã bỏ ký tự xuống dòng cuối).
    pub content: String,
}

/// Diff theo ô cho Excel — gồm nhiều sheet.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcelDiff {
    /// Kết quả từng sheet (union tên sheet của 2 file).
    pub sheets: Vec<SheetDiff>,
}

/// Trạng thái một cột/dòng sau khi căn chỉnh (alignment).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisMarker {
    /// `"equal" | "added" | "removed"`.
    pub tag: String,
    /// Nhãn cột gốc (A, B, C…) hoặc số dòng gốc — dùng hiển thị header.
    pub label: String,
    /// Dòng bị strikethrough toàn bộ (ở file A hoặc B) → xem là đã xóa.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub strikethrough: bool,
}

/// Kết quả diff của một sheet Excel (đã căn chỉnh cột + dòng bằng LCS).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetDiff {
    /// Tên sheet.
    pub name: String,
    /// Marker cho từng cột ở lưới kết quả (theo thứ tự hiển thị).
    pub columns: Vec<AxisMarker>,
    /// Marker cho từng dòng ở lưới kết quả (theo thứ tự hiển thị).
    pub rows_meta: Vec<AxisMarker>,
    /// Lưới ô đã diff: `cells[row][col]` (khớp `columns` × `rows_meta`).
    pub cells: Vec<Vec<CellDiff>>,
    /// Số ô thay đổi giá trị.
    pub changed: usize,
    /// Số ô thêm mới (chỉ có ở B, không rỗng).
    pub added: usize,
    /// Số ô bị xóa (chỉ có ở A, không rỗng).
    pub removed: usize,
    /// Số cột thêm mới.
    pub col_added: usize,
    /// Số cột bị xóa.
    pub col_removed: usize,
    /// Số dòng thêm mới.
    pub row_added: usize,
    /// Số dòng bị xóa.
    pub row_removed: usize,
    /// Số dòng bị strikethrough (xóa bằng gạch ngang).
    pub row_strikethrough: usize,
}

/// Trạng thái một ô sau khi so sánh.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CellDiff {
    /// `"equal" | "changed" | "added" | "removed"`.
    pub tag: String,
    /// Giá trị phía file A.
    pub old: String,
    /// Giá trị phía file B.
    pub new: String,
}
