import { safeInvoke } from "./_base";

export type SyncEntry = {
  project_code: string;
  project_name: string;
  task_name: string;
  process_code: string;
  category_label: string;
  hour: number;
  comment: string;
  regular_ot: number;
  midnight_ot: number;
};

export type SyncDailyReportRequest = {
  date: string;
  entries: SyncEntry[];
};

export type SyncResult = {
  success: boolean;
  message: string;
  synced_count: number;
  total_count: number;
};

export function syncDailyReport(request: SyncDailyReportRequest) {
  return safeInvoke<SyncResult>("sync_daily_report", { request });
}
