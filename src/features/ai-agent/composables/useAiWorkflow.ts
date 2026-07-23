import { computed, ref } from "vue";
import { useToast } from "@/shared/composables/useToast";
import { useAuthStore } from "@/app/stores/auth";
import { friendlyError } from "@/tauri/commands/_base";
import {
  aiWorkflowCreate,
  aiWorkflowDelete,
  aiWorkflowList,
  aiWorkflowUpdate,
  aiWorkflowStepCreate,
  aiWorkflowStepDelete,
  aiWorkflowStepList,
  aiWorkflowSaveLayout,
  aiWorkflowStepReorder,
  aiWorkflowStepUpdate,
} from "@/tauri/commands/ai-workflow";
import type {
  AiWorkflowResult,
  AiWorkflowStepResult,
} from "@/tauri/commands/ai-workflow";
import type { WorkflowStepType } from "@/_/types/ai-workflow";
import { STEP_TYPE_META } from "@/_/types/ai-workflow";

export type Workflow = {
  id: number;
  name: string;
  description: string;
  layout: Record<string, { x: number; y: number }>;
  stepCount: number;
  createdAt: string;
  updatedAt: string;
};

export type WorkflowStep = {
  id: number;
  workflowId: number;
  name: string;
  type: WorkflowStepType;
  description: string;
  icon: string;
  stepOrder: number;
};

function parseLayout(raw: string): Record<string, { x: number; y: number }> {
  try {
    const parsed = JSON.parse(raw);
    if (typeof parsed === "object" && parsed !== null) return parsed;
  } catch { /* ignore */ }
  return {};
}

function toWorkflow(r: AiWorkflowResult): Workflow {
  return {
    id: r.id,
    name: r.name,
    description: r.description,
    layout: parseLayout(r.layout),
    stepCount: r.step_count,
    createdAt: r.created_at,
    updatedAt: r.updated_at,
  };
}

function toStep(r: AiWorkflowStepResult): WorkflowStep {
  return {
    id: r.id,
    workflowId: r.workflow_id,
    name: r.name,
    type: r.step_type as WorkflowStepType,
    description: r.description,
    icon: r.icon,
    stepOrder: r.step_order,
  };
}

export function useAiWorkflow() {
  const toast = useToast();
  const auth = useAuthStore();
  const username = computed(() => auth.user?.username ?? "");

  const workflows = ref<Workflow[]>([]);
  const activeId = ref<number | null>(null);
  const activeSteps = ref<WorkflowStep[]>([]);
  const selectedStepId = ref<number | null>(null);
  const searchQuery = ref("");
  const isLoading = ref(false);
  const error = ref("");

  const activeWorkflow = computed(() => workflows.value.find((w) => w.id === activeId.value) ?? null);
  const selectedStep = computed(() => activeSteps.value.find((s) => s.id === selectedStepId.value) ?? null);

  const filteredWorkflows = computed(() => {
    const q = searchQuery.value.trim().toLowerCase();
    if (!q) return workflows.value;
    return workflows.value.filter((w) => w.name.toLowerCase().includes(q) || w.description.toLowerCase().includes(q));
  });

  async function loadWorkflows() {
    if (!username.value) return;
    isLoading.value = true;
    error.value = "";
    try {
      const results = await aiWorkflowList(username.value);
      workflows.value = results.map(toWorkflow);
      if (activeId.value === null && workflows.value.length > 0) {
        activeId.value = workflows.value[0].id;
      }
      if (activeId.value !== null) {
        await loadSteps(activeId.value);
      }
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadSteps(workflowId: number) {
    try {
      const results = await aiWorkflowStepList(workflowId);
      activeSteps.value = results.map(toStep);
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  loadWorkflows();

  async function selectWorkflow(id: number) {
    activeId.value = id;
    selectedStepId.value = null;
    await loadSteps(id);
  }

  async function createWorkflow(name: string, description: string): Promise<Workflow | null> {
    if (!username.value) return null;
    error.value = "";
    try {
      const result = await aiWorkflowCreate(username.value, { name, description });
      const wf = toWorkflow(result);
      workflows.value.unshift(wf);
      activeId.value = wf.id;
      activeSteps.value = [];
      selectedStepId.value = null;
      toast.success("Workflow created");
      return wf;
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(friendlyError(e));
      return null;
    }
  }

  async function updateWorkflow(id: number, patch: { name?: string; description?: string }) {
    if (!username.value) return;
    const wf = workflows.value.find((w) => w.id === id);
    if (!wf) return;
    error.value = "";
    try {
      const result = await aiWorkflowUpdate(id, username.value, {
        name: patch.name ?? wf.name,
        description: patch.description ?? wf.description,
      });
      const updated = toWorkflow(result);
      const idx = workflows.value.findIndex((w) => w.id === id);
      if (idx !== -1) workflows.value[idx] = updated;
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(friendlyError(e));
    }
  }

  async function deleteWorkflow(id: number) {
    if (!username.value) return;
    error.value = "";
    try {
      await aiWorkflowDelete(id, username.value);
      workflows.value = workflows.value.filter((w) => w.id !== id);
      if (activeId.value === id) {
        activeId.value = workflows.value[0]?.id ?? null;
        selectedStepId.value = null;
        if (activeId.value !== null) {
          await loadSteps(activeId.value);
        } else {
          activeSteps.value = [];
        }
      }
      toast.success("Workflow deleted");
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(friendlyError(e));
    }
  }

  async function duplicateWorkflow(id: number) {
    if (!username.value) return;
    const source = workflows.value.find((w) => w.id === id);
    if (!source) return;
    error.value = "";
    try {
      const result = await aiWorkflowCreate(username.value, {
        name: `${source.name} (copy)`,
        description: source.description,
      });
      const wf = toWorkflow(result);

      for (const step of activeSteps.value) {
        await aiWorkflowStepCreate(wf.id, {
          name: step.name,
          step_type: step.type,
          description: step.description,
          icon: step.icon,
          step_order: step.stepOrder,
        });
      }

      workflows.value.unshift(wf);
      activeId.value = wf.id;
      selectedStepId.value = null;
      await loadSteps(wf.id);
      toast.success("Workflow duplicated");
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(friendlyError(e));
    }
  }

  async function addStep(afterStepId: number | null, step?: Partial<WorkflowStep>): Promise<WorkflowStep | null> {
    const wf = activeWorkflow.value;
    if (!wf) return null;

    const type: WorkflowStepType = step?.type ?? "custom";
    const meta = STEP_TYPE_META[type];

    let stepOrder: number;
    if (afterStepId !== null) {
      const idx = activeSteps.value.findIndex((s) => s.id === afterStepId);
      stepOrder = idx !== -1 ? idx + 1 : activeSteps.value.length;
    } else {
      stepOrder = activeSteps.value.length;
    }

    error.value = "";
    try {
      const result = await aiWorkflowStepCreate(wf.id, {
        name: step?.name ?? "New Step",
        step_type: type,
        description: step?.description ?? "",
        icon: step?.icon ?? meta.icon,
        step_order: stepOrder,
      });
      const newStep = toStep(result);

      if (afterStepId !== null) {
        const idx = activeSteps.value.findIndex((s) => s.id === afterStepId);
        activeSteps.value.splice(idx + 1, 0, newStep);
        await reorderAndSync();
      } else {
        activeSteps.value.push(newStep);
      }

      selectedStepId.value = newStep.id;
      updateWorkflowStepCount(wf.id, 1);
      return newStep;
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(friendlyError(e));
      return null;
    }
  }

  async function updateStep(stepId: number, patch: Partial<Omit<WorkflowStep, "id" | "workflowId">>) {
    const step = activeSteps.value.find((s) => s.id === stepId);
    if (!step) return;
    error.value = "";
    try {
      const result = await aiWorkflowStepUpdate(stepId, {
        name: patch.name ?? step.name,
        step_type: patch.type ?? step.type,
        description: patch.description ?? step.description,
        icon: patch.icon ?? step.icon,
        step_order: patch.stepOrder ?? step.stepOrder,
      });
      const updated = toStep(result);
      const idx = activeSteps.value.findIndex((s) => s.id === stepId);
      if (idx !== -1) activeSteps.value[idx] = updated;
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(friendlyError(e));
    }
  }

  async function deleteStep(stepId: number) {
    const wf = activeWorkflow.value;
    if (!wf) return;
    error.value = "";
    try {
      await aiWorkflowStepDelete(stepId);
      activeSteps.value = activeSteps.value.filter((s) => s.id !== stepId);
      if (selectedStepId.value === stepId) selectedStepId.value = null;
      updateWorkflowStepCount(wf.id, -1);
      await reorderAndSync();
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(friendlyError(e));
    }
  }

  async function moveStep(fromIndex: number, toIndex: number) {
    const wf = activeWorkflow.value;
    if (!wf) return;
    const [moved] = activeSteps.value.splice(fromIndex, 1);
    activeSteps.value.splice(toIndex, 0, moved);
    await reorderAndSync();
  }

  async function reorderAndSync() {
    const wf = activeWorkflow.value;
    if (!wf) return;
    activeSteps.value.forEach((s, i) => (s.stepOrder = i));
    try {
      await aiWorkflowStepReorder(wf.id, activeSteps.value.map((s) => s.id));
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  function updateWorkflowStepCount(workflowId: number, delta: number) {
    const wf = workflows.value.find((w) => w.id === workflowId);
    if (wf) wf.stepCount += delta;
  }

  async function saveLayout(workflowId: number, positions: Record<string, { x: number; y: number }>) {
    if (!username.value) return;
    try {
      await aiWorkflowSaveLayout(workflowId, username.value, JSON.stringify(positions));
      const wf = workflows.value.find((w) => w.id === workflowId);
      if (wf) wf.layout = positions;
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  function selectStep(stepId: number | null) {
    selectedStepId.value = stepId;
  }

  return {
    workflows,
    activeId,
    selectedStepId,
    searchQuery,
    activeWorkflow,
    activeSteps,
    selectedStep,
    filteredWorkflows,
    isLoading,
    error,
    selectWorkflow,
    createWorkflow,
    updateWorkflow,
    deleteWorkflow,
    duplicateWorkflow,
    addStep,
    updateStep,
    deleteStep,
    moveStep,
    saveLayout,
    selectStep,
  };
}
