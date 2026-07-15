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
