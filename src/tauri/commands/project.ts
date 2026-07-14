import { safeInvoke } from "./_base";
import type {
  BacklogProjectLookup,
  CreateProjectRequest,
  CreateProjectTaskRequest,
  ProjectDetailResult,
  ProjectSummaryResult,
  ProjectTaskResult,
} from "@/_/types/project";

export function createProject(request: CreateProjectRequest) {
  return safeInvoke<ProjectDetailResult>("create_project", { request });
}

export function updateProject(projectId: number, request: CreateProjectRequest) {
  return safeInvoke<ProjectDetailResult>("update_project", { projectId, request });
}

export function getProjectDetail(projectId: number) {
  return safeInvoke<ProjectDetailResult>("get_project_detail", { projectId });
}

export function listProjects() {
  return safeInvoke<ProjectSummaryResult[]>("list_projects");
}

export function deleteProject(projectId: number) {
  return safeInvoke<void>("delete_project", { projectId });
}

export function getBacklogProjectByKey(projectKey: string) {
  return safeInvoke<BacklogProjectLookup>("get_backlog_project_by_key", { projectKey });
}

export function createProjectTask(projectId: number, request: CreateProjectTaskRequest) {
  return safeInvoke<ProjectTaskResult>("create_project_task", { projectId, request });
}

export function listProjectTasks(projectId: number) {
  return safeInvoke<ProjectTaskResult[]>("list_project_tasks", { projectId });
}

export function deleteProjectTask(taskId: string) {
  return safeInvoke<void>("delete_project_task", { taskId });
}
