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
}
