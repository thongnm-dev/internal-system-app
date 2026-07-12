import { computed, ref } from "vue";
import { useProjects } from "./useProjects";
import type { ProjectSummary } from "@/shared/types/analysis";

export type ProjectTaskCategory = "PG" | "Review PG" | "UT" | "Review UT" | "Other";

export const PROJECT_TASK_CATEGORIES: ProjectTaskCategory[] = [
  "PG",
  "Review PG",
  "UT",
  "Review UT",
  "Other",
];

export type ProjectTask = {
  id: string;
  shortName: string;
  description: string;
  categories: ProjectTaskCategory[];
  assignee: string;
  estimateHour: string;
  dueDate: string;
  /** Backlog issue key used to map/sync this task with the backlog. */
  issueKey: string;
  createdAt: string;
};

export type ProjectTaskInput = Omit<ProjectTask, "id" | "createdAt">;

function storageKey(projectCode: string) {
  return `pjjyuji.project-tasks.${projectCode || "unknown"}`;
}

function loadTasks(projectCode: string): ProjectTask[] {
  try {
    const saved = window.localStorage.getItem(storageKey(projectCode));
    return saved ? (JSON.parse(saved) as ProjectTask[]) : [];
  } catch {
    return [];
  }
}

function persistTasks(projectCode: string, tasks: ProjectTask[]) {
  try {
    window.localStorage.setItem(storageKey(projectCode), JSON.stringify(tasks));
  } catch {
    /* ignore quota / serialization errors */
  }
}

export function emptyProjectTaskInput(assignee = ""): ProjectTaskInput {
  return {
    shortName: "",
    description: "",
    categories: [],
    assignee,
    estimateHour: "",
    dueDate: "",
    issueKey: "",
  };
}

/**
 * Project-scoped task store, persisted to localStorage per `project_code`.
 * Kept independent from the Daily Report task store.
 */
export function useProjectTasks(projectCode: string) {
  const tasks = ref<ProjectTask[]>(loadTasks(projectCode));
  const { result } = useProjects();

  const project = computed<ProjectSummary | null>(
    () => result.value?.projects.find((p) => p.project_code === projectCode) ?? null,
  );

  const projectName = computed(() => project.value?.project_name ?? "");

  function addTask(input: ProjectTaskInput): ProjectTask | null {
    const shortName = input.shortName.trim();
    if (!shortName) return null;
    const task: ProjectTask = {
      id: `task-${Date.now()}`,
      shortName,
      description: input.description.trim(),
      categories: input.categories.slice(),
      assignee: input.assignee.trim(),
      estimateHour: input.estimateHour.trim(),
      dueDate: input.dueDate,
      issueKey: input.issueKey.trim(),
      createdAt: new Date().toISOString(),
    };
    tasks.value = [...tasks.value, task];
    persistTasks(projectCode, tasks.value);
    return task;
  }

  function removeTask(id: string) {
    tasks.value = tasks.value.filter((t) => t.id !== id);
    persistTasks(projectCode, tasks.value);
  }

  return { addTask, project, projectName, removeTask, tasks };
}
