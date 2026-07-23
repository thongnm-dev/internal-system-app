import type { AiTaskCategory } from "@/tauri/commands/ai-task";

export const TASK_CATEGORY_META: Record<AiTaskCategory, { label: string; badgeClass: string }> = {
  screen: { label: "Screen", badgeClass: "bg-sky-100 text-sky-700" },
  batch:  { label: "Batch",  badgeClass: "bg-violet-100 text-violet-700" },
  part:   { label: "Part",   badgeClass: "bg-amber-100 text-amber-700" },
  other:  { label: "Other", badgeClass: "bg-canvas text-muted" },
};

export const TASK_CATEGORY_OPTIONS: { label: string; value: AiTaskCategory }[] = [
  { label: "Screen", value: "screen" },
  { label: "Batch", value: "batch" },
  { label: "Part", value: "part" },
  { label: "Other", value: "other" },
];
