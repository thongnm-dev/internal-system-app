import { safeInvoke } from "./_base";

export type WorkInfoEntry = {
  phase: string;
  job_id: string;
  job_name: string;
  hours: number;
  sheet_name: string;
};

export type DayWork = {
  day: string;
  entries: WorkInfoEntry[];
};

export type WorkerSchedule = {
  worker_name: string;
  days: DayWork[];
};

export type ScheduleResult = {
  file_path: string;
  target_month: string;
  workers: WorkerSchedule[];
};

export function readScheduleExcel(filePath: string, targetYear: number, targetMonth: number, userFilter?: string) {
  return safeInvoke<ScheduleResult>("read_schedule_excel", {
    filePath,
    targetYear,
    targetMonth,
    userFilter,
  });
}
