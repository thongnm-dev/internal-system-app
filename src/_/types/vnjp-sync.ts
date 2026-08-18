export interface SheetMeta {
  name: string;
  tabColor: string | null;
  rowCount: number;
  colCount: number;
  redCellCount: number;
  strikeCellCount: number;
}

export interface RedCell {
  sheet: string;
  row: number;
  col: number;
  vnText: string;
  jpText: string;
  translation: string | null;
  /** `true` nếu đây là 1 shape/textbox nổi (không phải cell) — `row`/`col` là ô neo của shape. */
  isShape: boolean;
}

export interface StrikeCell {
  sheet: string;
  row: number;
  col: number;
  text: string;
  /** `true` nếu đây là 1 shape/textbox nổi — xem `RedCell.isShape`. */
  isShape: boolean;
}

export interface QualityIssue {
  sheet: string;
  row: number;
  col: number;
  issueType: string;
  content: string;
  description: string;
  /** `true` nếu vấn đề này nằm trong 1 shape/textbox nổi — xem `RedCell.isShape`. */
  isShape: boolean;
}

export interface SheetCompare {
  name: string;
  inVn: boolean;
  inJp: boolean;
  vnTabColor: string | null;
  jpTabColor: string | null;
  vnRows: number;
  jpRows: number;
  vnRedCount: number;
  jpStrikeCount: number;
}

export interface SyncAnalysis {
  vnPath: string;
  jpPath: string;
  vnSheets: SheetMeta[];
  jpSheets: SheetMeta[];
  sheetCompare: SheetCompare[];
  redCells: RedCell[];
  strikeCells: StrikeCell[];
  qualityIssues: QualityIssue[];
}

export interface TranslateItem {
  id: string;
  sheet: string;
  row: number;
  col: number;
  text: string;
}

export interface TranslateBatchRequest {
  items: TranslateItem[];
  provider: string;
  model: string;
  apiKey?: string | null;
}

export interface TranslateItemResult {
  id: string;
  translation: string;
  error: string | null;
}

export interface ApplyResult {
  outputPath: string;
  appliedCount: number;
  skippedCount: number;
  sheetsModified: string[];
  strikeRemovedCount: number;
  redBlackenedCount: number;
  cleanupSkippedCount: number;
  columnCorrectedCount: number;
  /** Số đoạn văn shape/textbox nổi đã ghi được vào đúng shape JP tương ứng (khớp theo tên shape). */
  shapeAppliedCount: number;
  /** Số đoạn văn shape VN bị bỏ qua vì không tìm thấy shape cùng tên trong JP. */
  shapeSkippedCount: number;
  /** Số sheet chỉ có ở VN (tab màu quy ước) đã được clone nguyên trạng sang JP. */
  clonedSheetCount: number;
  /** Số sheet JP có hậu tố "(DEL)" đã được xử lý (chỉ bỏ màu chữ về đen, không đụng gì khác). */
  delSheetCount: number;
  /** Số dòng đã được tự động chèn để canh khớp lệch dòng VN↔JP trước khi ghi nội dung. */
  rowsInserted: number;
}

export interface CleanupResult {
  outputPath: string;
  sheetsModified: string[];
  strikeRemovedCount: number;
  redBlackenedCount: number;
  skippedCount: number;
  /** Số sheet JP có hậu tố "(DEL)" đã được xử lý (chỉ bỏ màu chữ về đen, không đụng gì khác). */
  delSheetCount: number;
}

export interface RowAlignmentSuggestion {
  sheet: string;
  /** Số dòng JP (1-based) mà dòng mới sẽ chèn NGAY SAU đó. 0 = chèn ở đầu sheet. */
  jpInsertAfterRow: number;
  insertCount: number;
  vnRowStart: number;
  vnRowEnd: number;
  sampleVnText: string[];
  hasRed: boolean;
  hasStrike: boolean;
}

export interface RowAlignmentReport {
  suggestions: RowAlignmentSuggestion[];
}

export interface ConfirmedInsert {
  sheet: string;
  jpInsertAfterRow: number;
  insertCount: number;
  /** Phạm vi dòng VN (1-based) của group cần clone nguyên vào JP. Bỏ trống ⇒ chỉ chèn dòng trống. */
  vnRowStart?: number;
  vnRowEnd?: number;
}

export interface RowInsertResult {
  outputPath: string;
  sheetsModified: string[];
  rowsInserted: number;
}

export interface BetterMatch {
  row: number;
  col: number;
  similarity: number;
}

export interface RedCellVerification {
  sheet: string;
  row: number;
  col: number;
  /** Bản dịch VN→JP do AI tạo — CHỈ dùng để so sánh, không ghi vào tài liệu. */
  aiTranslation: string;
  similaritySamePos: number;
  betterMatch: BetterMatch | null;
}

export interface RedCellVerificationReport {
  items: RedCellVerification[];
}
