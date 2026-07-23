export type WorkflowStepType = "skill" | "implement" | "review" | "release" | "custom";

export type WorkflowStep = {
  id: string;
  name: string;
  type: WorkflowStepType;
  description: string;
  icon: string;
};

export type Workflow = {
  id: string;
  name: string;
  description: string;
  steps: WorkflowStep[];
  createdAt: string;
  updatedAt: string;
};

export const STEP_TYPE_META: Record<WorkflowStepType, { label: string; icon: string; badgeClass: string }> = {
  skill:     { label: "Skill",     icon: "pi pi-book",         badgeClass: "bg-sky-100 text-sky-700" },
  implement: { label: "Implement", icon: "pi pi-code",         badgeClass: "bg-violet-100 text-violet-700" },
  review:    { label: "Review",    icon: "pi pi-check-circle", badgeClass: "bg-amber-100 text-amber-700" },
  release:   { label: "Release",   icon: "pi pi-upload",       badgeClass: "bg-emerald-100 text-emerald-700" },
  custom:    { label: "Custom",    icon: "pi pi-cog",          badgeClass: "bg-canvas text-muted" },
};
