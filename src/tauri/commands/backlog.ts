import { safeInvoke } from "./_base";

export type BacklogIssueType = {
  id: number;
  projectId: number;
  name: string;
  color?: string;
  displayOrder?: number;
};

export type BacklogStatus = {
  id: number;
  projectId?: number;
  name: string;
  color?: string;
  displayOrder?: number;
};

export type BacklogCategory = {
  id: number;
  name: string;
  displayOrder?: number;
};

export type BacklogUser = {
  id: number;
  userId?: string;
  name: string;
  roleType?: number;
  lang?: string;
  mailAddress?: string;
  keyword?: string;
};

export function backlogListIssueTypes(projectKey: string) {
  return safeInvoke<BacklogIssueType[]>("backlog_list_issue_types", { projectKey });
}

export function backlogListStatuses(projectKey: string) {
  return safeInvoke<BacklogStatus[]>("backlog_list_statuses", { projectKey });
}

export function backlogListCategories(projectKey: string) {
  return safeInvoke<BacklogCategory[]>("backlog_list_categories", { projectKey });
}

export function backlogListProjectUsers(projectKey: string) {
  return safeInvoke<BacklogUser[]>("backlog_list_project_users", { projectKey });
}

export type BacklogPriority = {
  id: number;
  name: string;
};

export type BacklogIssue = {
  id: number;
  projectId: number;
  issueKey: string;
  keyId: number;
  issueType: BacklogIssueType | null;
  summary: string;
  description: string | null;
  priority: BacklogPriority | null;
  status: BacklogStatus | null;
  assignee: BacklogUser | null;
  category: BacklogCategory[] | null;
  milestone: unknown[] | null;
  startDate: string | null;
  dueDate: string | null;
  estimatedHours: number | null;
  actualHours: number | null;
  parentIssueId: number | null;
  createdUser: BacklogUser | null;
  created: string | null;
  updated: string | null;
};

export type BacklogIssueQuery = {
  projectKey: string;
  count?: number;
  offset?: number;
  statusIds?: number[];
  assigneeIds?: number[];
  issueTypeIds?: number[];
  categoryIds?: number[];
  milestoneIds?: number[];
  priorityIds?: number[];
  keyword?: string;
  sort?: string;
  order?: string;
  parentIssueId?: number;
};

export type BacklogIssueList = {
  issues: BacklogIssue[];
  totalCount: number;
};

export type BacklogProjectLookup = {
  projectId: string;
  projectKey: string;
  projectName: string;
};

export function backlogListIssues(query: BacklogIssueQuery) {
  return safeInvoke<BacklogIssueList>("backlog_list_issues", { query });
}

export function backlogGetProjectLookup(projectKey: string) {
  return safeInvoke<BacklogProjectLookup>("backlog_get_project_lookup", { projectKey });
}

export function backlogListPriorities() {
  return safeInvoke<BacklogPriority[]>("backlog_list_priorities");
}

export type BacklogCreateIssueRequest = {
  projectId: number;
  summary: string;
  issueTypeId: number;
  priorityId: number;
  description?: string;
  assigneeId?: number;
  startDate?: string;
  dueDate?: string;
  estimatedHours?: number;
  actualHours?: number;
  categoryId?: number[];
  milestoneId?: number[];
  parentIssueId?: number;
};

export function backlogCreateIssue(request: BacklogCreateIssueRequest) {
  return safeInvoke<BacklogIssue>("backlog_create_issue", { request });
}
