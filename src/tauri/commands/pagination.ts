import { safeInvoke } from "./_base";

export interface PaginationConfig {
  rows: number;
  rows_per_page_options: number[];
  rows_compact: number;
  rows_per_page_options_compact: number[];
  paginator_template: string;
  current_page_report_template: string;
}

export function getPaginationConfig() {
  return safeInvoke<PaginationConfig>("get_pagination_config");
}
