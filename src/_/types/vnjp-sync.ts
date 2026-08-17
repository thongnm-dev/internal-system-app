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
}

export interface StrikeCell {
  sheet: string;
  row: number;
  col: number;
  text: string;
}

export interface QualityIssue {
  sheet: string;
  row: number;
  col: number;
  issueType: string;
  content: string;
  description: string;
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
}

export interface CleanupResult {
  outputPath: string;
  sheetsModified: string[];
  strikeRemovedCount: number;
  redBlackenedCount: number;
  skippedCount: number;
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
