import { safeInvoke } from "./_base";

// ── ai_tasks ────────────────────────────────────────────────────────────

export type AiTaskCategory = "screen" | "batch" | "part" | "other";

export type AiTaskResult = {
  id: number;
  task_cd: string;
  task_name: string;
  category: string;
  is_complete: boolean;
  completed_at: string;
  created_at: string;
  created_by: string;
  updated_at: string;
  updated_by: string;
};

export type CreateTaskRequest = {
  task_cd: string;
  task_name: string;
  category: string;
};

export type UpdateTaskRequest = {
  task_cd: string;
  task_name: string;
  category: string;
  is_complete: boolean;
};

export function aiTaskCreate(username: string, request: CreateTaskRequest) {
  return safeInvoke<AiTaskResult>("ai_task_create", { username, request });
}

export function aiTaskList(keyword?: string, isComplete?: boolean) {
  return safeInvoke<AiTaskResult[]>("ai_task_list", {
    keyword: keyword ?? null,
    isComplete: isComplete ?? null,
  });
}

export function aiTaskUpdate(id: number, username: string, request: UpdateTaskRequest) {
  return safeInvoke<AiTaskResult>("ai_task_update", { id, username, request });
}

// ── ai_task_wf_proc ─────────────────────────────────────────────────────

export type AiTaskWfProcResult = {
  id: number;
  task_id: number;
  wf_id: number;
  latest_step_id: number | null;
  created_at: string;
  created_by: string;
  updated_at: string;
  updated_by: string;
};

export type CreateWfProcRequest = {
  task_id: number;
  wf_id: number;
};

export function aiTaskWfProcCreate(username: string, request: CreateWfProcRequest) {
  return safeInvoke<AiTaskWfProcResult>("ai_task_wf_proc_create", { username, request });
}

export function aiTaskWfProcList(taskId: number) {
  return safeInvoke<AiTaskWfProcResult[]>("ai_task_wf_proc_list", { taskId });
}

export function aiTaskWfProcUpdate(id: number, latestStepId: number, username: string) {
  return safeInvoke<AiTaskWfProcResult>("ai_task_wf_proc_update", { id, latestStepId, username });
}

// ── ai_task_wf_proc_step ────────────────────────────────────────────────

export type WfProcStepStatus = "pending" | "in_progress" | "completed" | "skipped";

export type AiTaskWfProcStepResult = {
  id: number;
  wf_proc_id: number;
  wf_step_id: number;
  status: string;
  created_at: string;
  created_by: string;
  updated_at: string;
  updated_by: string;
};

export type CreateWfProcStepRequest = {
  wf_proc_id: number;
  wf_step_id: number;
  status: string;
};

export function aiTaskWfProcStepCreate(username: string, request: CreateWfProcStepRequest) {
  return safeInvoke<AiTaskWfProcStepResult>("ai_task_wf_proc_step_create", { username, request });
}

export function aiTaskWfProcStepList(wfProcId: number) {
  return safeInvoke<AiTaskWfProcStepResult[]>("ai_task_wf_proc_step_list", { wfProcId });
}

export function aiTaskWfProcStepUpdate(id: number, status: string, username: string) {
  return safeInvoke<AiTaskWfProcStepResult>("ai_task_wf_proc_step_update", { id, status, username });
}
