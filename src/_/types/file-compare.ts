/** Kiểu dữ liệu cho công cụ so sánh khác biệt giữa 2 file. */

/** Loại file hỗ trợ so sánh. */
export type FileCompareKind = "text" | "markdown" | "word" | "excel";

/** Trạng thái một dòng trong diff văn bản. */
export type DiffLineTag = "equal" | "insert" | "delete";

/** Một dòng đã ghép diff (text/markdown/word). */
export interface DiffLine {
  tag: DiffLineTag;
  /** Số dòng phía file A (1-based); null với dòng thêm mới. */
  oldLine: number | null;
  /** Số dòng phía file B (1-based); null với dòng bị xóa. */
  newLine: number | null;
  content: string;
}

/** Diff theo dòng cho text/markdown/word. */
export interface TextDiff {
  lines: DiffLine[];
  added: number;
  removed: number;
}

/** Trạng thái một ô Excel sau khi so sánh. */
export type CellDiffTag = "equal" | "changed" | "added" | "removed";

/** Một ô đã diff. */
export interface CellDiff {
  tag: CellDiffTag;
  old: string;
  new: string;
}

/** Trạng thái một cột/dòng sau khi căn chỉnh (alignment). */
export type AxisTag = "equal" | "added" | "removed";

/** Marker cho một cột hoặc một dòng ở lưới kết quả. */
export interface AxisMarker {
  tag: AxisTag;
  /** Nhãn hiển thị header (chữ cột A/B/C… hoặc số dòng). */
  label: string;
  /** Dòng bị strikethrough toàn bộ ở file A → xem là đã xóa. */
  strikethrough?: boolean;
}

/** Kết quả diff của một sheet Excel (đã căn chỉnh cột + dòng). */
export interface SheetDiff {
  name: string;
  /** Marker từng cột ở lưới kết quả. */
  columns: AxisMarker[];
  /** Marker từng dòng ở lưới kết quả. */
  rowsMeta: AxisMarker[];
  /** Lưới ô đã diff: cells[row][col] (khớp columns × rowsMeta). */
  cells: CellDiff[][];
  changed: number;
  added: number;
  removed: number;
  colAdded: number;
  colRemoved: number;
  rowAdded: number;
  rowRemoved: number;
  rowStrikethrough: number;
}

/** Diff theo ô cho Excel (nhiều sheet). */
export interface ExcelDiff {
  sheets: SheetDiff[];
}

/** Kết quả so sánh trả về từ backend. */
export interface CompareResult {
  kind: FileCompareKind;
  textDiff: TextDiff | null;
  excelDiff: ExcelDiff | null;
}
