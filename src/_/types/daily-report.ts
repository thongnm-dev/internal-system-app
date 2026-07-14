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
  created_at: string;
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
