import { safeInvoke } from "./_base";

export type AiTaskCategory = "screen" | "batch" | "part" | "other";

export type AiTaskResult = {
  id: number;
  task_code: string;
  category: string;
  description: string;
  created_by: string;
  created_at: string;
};

export type CreateTaskRequest = {
  task_code: string;
  category: string;
  description: string;
};

export function aiTaskCreate(username: string, request: CreateTaskRequest) {
  return safeInvoke<AiTaskResult>("ai_task_create", { username, request });
}

export function aiTaskList(keyword?: string) {
  return safeInvoke<AiTaskResult[]>("ai_task_list", { keyword: keyword ?? null });
}
