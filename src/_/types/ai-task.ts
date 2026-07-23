import type { AiTaskCategory, WfProcStepStatus } from "@/tauri/commands/ai-task";

export const TASK_CATEGORY_META: Record<AiTaskCategory, { label: string; badgeClass: string }> = {
  screen: { label: "Screen", badgeClass: "bg-sky-100 text-sky-700" },
  batch:  { label: "Batch",  badgeClass: "bg-violet-100 text-violet-700" },
  part:   { label: "Part",   badgeClass: "bg-amber-100 text-amber-700" },
  other:  { label: "Other",  badgeClass: "bg-canvas text-muted" },
};

export const TASK_CATEGORY_OPTIONS: { label: string; value: AiTaskCategory }[] = [
  { label: "Screen", value: "screen" },
  { label: "Batch", value: "batch" },
  { label: "Part", value: "part" },
  { label: "Other", value: "other" },
];

export const STEP_STATUS_META: Record<WfProcStepStatus, { label: string; badgeClass: string }> = {
  pending:     { label: "Pending",     badgeClass: "bg-canvas text-muted" },
  in_progress: { label: "In Progress", badgeClass: "bg-sky-100 text-sky-700" },
  completed:   { label: "Completed",   badgeClass: "bg-emerald-100 text-emerald-700" },
  skipped:     { label: "Skipped",     badgeClass: "bg-amber-100 text-amber-700" },
};

export const STEP_STATUS_OPTIONS: { label: string; value: WfProcStepStatus }[] = [
  { label: "Pending", value: "pending" },
  { label: "In Progress", value: "in_progress" },
  { label: "Completed", value: "completed" },
  { label: "Skipped", value: "skipped" },
];
