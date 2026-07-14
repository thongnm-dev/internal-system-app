export type ProjectMember = {
  username: string;
  name: string;
};

export type ProjectDetailResult = {
  id: number;
  code: string;
  name: string;
  client: string;
  backlog_key: string;
  backlog_url: string;
  backlog_space: string;
  is_active: boolean;
  members: ProjectMember[];
  created_at: string;
  updated_at: string;
};

export type BacklogProjectLookup = {
  project_id: string | number;
  project_key: string;
  project_name: string;
};

export type CreateProjectRequest = {
  code: string;
  name: string;
  client?: string;
  backlog_key?: string;
  backlog_url?: string;
  backlog_space?: string;
  members: ProjectMember[];
};

export type ProjectSummaryResult = {
  id: number;
  code: string;
  name: string;
  client: string;
  is_active: boolean;
  member_count: number;
  created_at: string;
};

export type ProjectTaskResult = {
  id: string;
  project_id: number;
  short_name: string;
  description: string;
  categories: string[];
  assignee: string;
  estimate_hour: string;
  due_date: string;
  issue_key: string;
  is_user_added: boolean;
  created_at: string;
};

export type CreateProjectTaskRequest = {
  short_name: string;
  description: string;
  categories: string[];
  assignee: string;
  estimate_hour: string;
  due_date: string;
  issue_key: string;
};
