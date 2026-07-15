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
  category_id: number;
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
  category_id: number;
};

export type DailyReportUserTaskResult = {
  id: number;
  username: string;
  task_id: string;
  project_id: string;
  name: string;
  description: string;
  category_id: number;
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
  id: number;
  process_code: string;
  process_name: string;
  short_name: string;
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
  id?: number;
  project_id: string;
  name: string;
  description: string;
  category_id: number;
  assignee: string;
  estimate_hour: string;
  due_date: string;
  issue_key: string;
};
