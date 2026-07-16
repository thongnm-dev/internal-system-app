import { safeInvoke } from "./_base";

export interface CollectConfig {
  input: string;
  output: string;
  keyword: string;
  files: string;
  ext: string;
  limit_copy: number;
  skip_dir: string;
  no_default_skip: boolean;
  flat: boolean;
  group_by_parens: boolean;
  overwrite: boolean;
  create_history: boolean;
  delete_non_vn: boolean;
  report_dir: string;
  dry_run: boolean;
}

export interface CollectRunResult {
  ok: boolean;
  log: string[];
  summary: string;
}

export const DEFAULT_COLLECT_CONFIG: CollectConfig = {
  input: "",
  output: "",
  keyword: "",
  files: "",
  ext: "xlsx",
  limit_copy: 50,
  skip_dir: "",
  no_default_skip: false,
  flat: false,
  group_by_parens: false,
  overwrite: true,
  create_history: true,
  delete_non_vn: false,
  report_dir: "reports",
  dry_run: false,
};

export function collectLoadIni() {
  return safeInvoke<CollectConfig>("collect_load_ini");
}

export function collectRun(config: CollectConfig) {
  return safeInvoke<CollectRunResult>("collect_run", { config });
}

export function collectByFolders(config: CollectConfig) {
  return safeInvoke<CollectRunResult>("collect_by_folders", { config });
}
