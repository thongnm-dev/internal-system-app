export type DailyReportEntryResult = {
  id: number;
  username: string;
  task_id: string;
  project_id: string;
  entry_date: string;
  comment: string;
  hour: number;
  is_ot: boolean;
  regular_ot: number;
  midnight_ot: number;
  phase: string;
  updated_at: string;
};

export type SaveDailyReportEntryRequest = {
  task_id: string;
  project_id: string;
  entry_date: string;
  comment: string;
  hour: number;
  is_ot: boolean;
  regular_ot: number;
  midnight_ot: number;
  phase: string;
};

export type DailyReportUserTaskResult = {
  id: number;
  username: string;
  task_id: string;
  project_id: string;
  code: string;
  name: string;
  description: string;
  categories: string[];
  assignee: string;
  estimate_hour: string;
  due_date: string;
  issue_key: string;
  is_completed: boolean;
  completed_at: string;
  created_at: string;
  is_user_added: boolean;
};

export type DailyReportTaskHoursResult = {
  task_id: string;
  total_hour: number;
};

export type DailyReportPhaseResult = {
  process_code: string;
  display_order: number;
};

export type DailyReportProjectResult = {
  id: number;
  code: string;
  name: string;
  client: string;
  is_member: boolean;
};

export type CreateDailyReportTaskRequest = {
  task_id: string;
  project_id: string;
  code: string;
  name: string;
  description: string;
  categories: string[];
  assignee: string;
  estimate_hour: string;
  due_date: string;
  issue_key: string;
};
