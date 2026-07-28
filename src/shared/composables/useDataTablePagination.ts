import { reactive } from "vue";
import { getPaginationConfig } from "@/tauri/commands/pagination";

interface PaginationProps {
  rows: number;
  rowsPerPageOptions: number[];
  paginatorTemplate: string;
  currentPageReportTemplate: string;
}

const DEFAULTS: PaginationProps = {
  rows: 20,
  rowsPerPageOptions: [20, 50, 100],
  paginatorTemplate:
    "FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport",
  currentPageReportTemplate: "Showing {first} to {last} of {totalRecords}",
};

const DEFAULTS_COMPACT: PaginationProps = {
  rows: 10,
  rowsPerPageOptions: [10, 20, 50],
  paginatorTemplate: DEFAULTS.paginatorTemplate,
  currentPageReportTemplate: DEFAULTS.currentPageReportTemplate,
};

const pagination = reactive<PaginationProps>({ ...DEFAULTS });
const paginationCompact = reactive<PaginationProps>({ ...DEFAULTS_COMPACT });

let loaded = false;

async function load() {
  if (loaded) return;
  loaded = true;

  const cfg = await getPaginationConfig();
  if (!cfg) return;

  pagination.rows = cfg.rows || DEFAULTS.rows;
  pagination.rowsPerPageOptions = cfg.rows_per_page_options?.length
    ? cfg.rows_per_page_options
    : DEFAULTS.rowsPerPageOptions;
  pagination.paginatorTemplate =
    cfg.paginator_template || DEFAULTS.paginatorTemplate;
  pagination.currentPageReportTemplate =
    cfg.current_page_report_template || DEFAULTS.currentPageReportTemplate;

  paginationCompact.rows = cfg.rows_compact || DEFAULTS_COMPACT.rows;
  paginationCompact.rowsPerPageOptions =
    cfg.rows_per_page_options_compact?.length
      ? cfg.rows_per_page_options_compact
      : DEFAULTS_COMPACT.rowsPerPageOptions;
  paginationCompact.paginatorTemplate = pagination.paginatorTemplate;
  paginationCompact.currentPageReportTemplate =
    pagination.currentPageReportTemplate;
}

export function useDataTablePagination() {
  load();
  return { pagination, paginationCompact };
}
