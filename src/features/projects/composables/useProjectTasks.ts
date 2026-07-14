import { computed, ref, onMounted } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import {
  getProjectDetail,
  createProjectTask,
  updateProjectTask,
  listProjectTasks,
  deleteProjectTask,
} from "@/tauri/commands/project";
import type { ProjectDetailResult, ProjectTaskResult, CreateProjectTaskRequest } from "@/_/types/project";

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
  issueKey: string;
  createdAt: string;
};

export type ProjectTaskInput = Omit<ProjectTask, "id" | "createdAt">;

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

function toProjectTask(r: ProjectTaskResult): ProjectTask {
  return {
    id: r.id,
    shortName: r.short_name,
    description: r.description,
    categories: r.categories as ProjectTaskCategory[],
    assignee: r.assignee,
    estimateHour: r.estimate_hour,
    dueDate: r.due_date,
    issueKey: r.issue_key,
    createdAt: r.created_at,
  };
}

export function useProjectTasks(projectId: string) {
  const tasks = ref<ProjectTask[]>([]);
  const project = ref<ProjectDetailResult | null>(null);
  const projectLoading = ref(false);
  const loading = ref(false);

  const numericId = Number(projectId);
  const validId = !Number.isNaN(numericId);

  const projectName = computed(() => {
    if (!project.value) return "";
    const p = project.value;
    const parts = [p.code, p.name].filter(Boolean);
    return parts.join(" - ");
  });

  const projectLabel = computed(() => {
    if (!project.value) return projectId;
    const p = project.value;
    if (p.client) {
      return `#【${p.client}】${p.code}_${p.name}`;
    }
    return `#${p.code}_${p.name}`;
  });

  async function loadProject() {
    if (!canUseTauriRuntime() || !validId) return;

    projectLoading.value = true;
    try {
      project.value = await getProjectDetail(numericId);
    } catch {
      project.value = null;
    } finally {
      projectLoading.value = false;
    }
  }

  async function loadTasks() {
    if (!canUseTauriRuntime() || !validId) return;

    loading.value = true;
    try {
      const results = await listProjectTasks(numericId);
      tasks.value = results.map(toProjectTask);
    } catch {
      tasks.value = [];
    } finally {
      loading.value = false;
    }
  }

  async function addTask(input: ProjectTaskInput): Promise<ProjectTask | null> {
    const shortName = input.shortName.trim();
    if (!shortName || !canUseTauriRuntime() || !validId) return null;

    const request: CreateProjectTaskRequest = {
      short_name: shortName,
      description: input.description.trim(),
      categories: input.categories.slice(),
      assignee: input.assignee.trim(),
      estimate_hour: String(input.estimateHour ?? "").trim(),
      due_date: input.dueDate,
      issue_key: input.issueKey.trim(),
    };

    const result = await createProjectTask(numericId, request);
    const task = toProjectTask(result);
    tasks.value = [...tasks.value, task];
    return task;
  }

  async function updateTask(id: string, input: ProjectTaskInput): Promise<ProjectTask | null> {
    const shortName = input.shortName.trim();
    if (!shortName || !canUseTauriRuntime()) return null;

    const request: CreateProjectTaskRequest = {
      short_name: shortName,
      description: input.description.trim(),
      categories: input.categories.slice(),
      assignee: input.assignee.trim(),
      estimate_hour: String(input.estimateHour ?? "").trim(),
      due_date: input.dueDate,
      issue_key: input.issueKey.trim(),
    };

    const result = await updateProjectTask(id, request);
    const task = toProjectTask(result);
    tasks.value = tasks.value.map((t) => (t.id === id ? task : t));
    return task;
  }

  async function removeTask(id: string) {
    if (!canUseTauriRuntime()) return;

    await deleteProjectTask(id);
    tasks.value = tasks.value.filter((t) => t.id !== id);
  }

  onMounted(() => {
    loadProject();
    loadTasks();
  });

  return { addTask, updateTask, loading, project, projectName, projectLabel, projectLoading, removeTask, tasks };
}
