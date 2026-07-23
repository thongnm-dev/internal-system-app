import { safeInvoke } from "./_base";

export type AiWorkflowResult = {
  id: number;
  name: string;
  description: string;
  layout: string;
  created_by: string;
  step_count: number;
  created_at: string;
  updated_at: string;
};

export type AiWorkflowStepResult = {
  id: number;
  workflow_id: number;
  name: string;
  step_type: string;
  skill_name: string;
  description: string;
  icon: string;
  step_order: number;
  created_at: string;
};

export type CreateWorkflowRequest = {
  name: string;
  description: string;
};

export type UpdateWorkflowRequest = {
  name: string;
  description: string;
};

export type CreateStepRequest = {
  name: string;
  step_type: string;
  skill_name: string;
  description: string;
  icon: string;
  step_order: number;
};

export type UpdateStepRequest = {
  name: string;
  step_type: string;
  skill_name: string;
  description: string;
  icon: string;
  step_order: number;
};

export function aiWorkflowCreate(username: string, request: CreateWorkflowRequest) {
  return safeInvoke<AiWorkflowResult>("ai_workflow_create", { username, request });
}

export function aiWorkflowList(username: string) {
  return safeInvoke<AiWorkflowResult[]>("ai_workflow_list", { username });
}

export function aiWorkflowUpdate(id: number, username: string, request: UpdateWorkflowRequest) {
  return safeInvoke<AiWorkflowResult>("ai_workflow_update", { id, username, request });
}

export function aiWorkflowDelete(id: number, username: string) {
  return safeInvoke<void>("ai_workflow_delete", { id, username });
}

export function aiWorkflowStepList(workflowId: number) {
  return safeInvoke<AiWorkflowStepResult[]>("ai_workflow_step_list", { workflowId });
}

export function aiWorkflowStepCreate(workflowId: number, request: CreateStepRequest) {
  return safeInvoke<AiWorkflowStepResult>("ai_workflow_step_create", { workflowId, request });
}

export function aiWorkflowStepUpdate(id: number, request: UpdateStepRequest) {
  return safeInvoke<AiWorkflowStepResult>("ai_workflow_step_update", { id, request });
}

export function aiWorkflowStepDelete(id: number) {
  return safeInvoke<void>("ai_workflow_step_delete", { id });
}

export function aiWorkflowStepReorder(workflowId: number, stepIds: number[]) {
  return safeInvoke<void>("ai_workflow_step_reorder", { workflowId, stepIds });
}

export function aiWorkflowSaveLayout(id: number, username: string, layoutJson: string) {
  return safeInvoke<void>("ai_workflow_save_layout", { id, username, layoutJson });
}
