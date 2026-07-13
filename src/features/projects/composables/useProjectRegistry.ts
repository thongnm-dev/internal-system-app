import { computed, ref } from "vue";
import {
  deleteProject as deleteProjectCommand,
  friendlyError,
  listProjects,
  type ProjectSummaryResult,
} from "@/tauri/commands";

export type ProjectFilters = {
  code: string;
  keyword: string;
  name: string;
};

/**
 * Project registry list backed by the Tauri `list_projects` command.
 * Independent from `useProjects` (the CSV analysis report), which keeps its
 * own analysis-shaped data.
 */
export function useProjectRegistry() {
  const filters = ref<ProjectFilters>({ code: "", keyword: "", name: "" });
  const projects = ref<ProjectSummaryResult[]>([]);
  const isLoading = ref(false);
  const loadError = ref("");

  async function loadProjects() {
    isLoading.value = true;
    loadError.value = "";
    try {
      projects.value = await listProjects();
    } catch (e) {
      loadError.value = friendlyError(e);
      projects.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  loadProjects();

  function normalize(value: string) {
    return value.trim().toLocaleLowerCase();
  }

  const filteredProjects = computed(() => {
    const code = normalize(filters.value.code);
    const name = normalize(filters.value.name);
    const keyword = normalize(filters.value.keyword);
    return projects.value.filter((p) => {
      const matchesCode = !code || normalize(p.code).includes(code);
      const matchesName = !name || normalize(p.name).includes(name);
      const matchesKeyword =
        !keyword || normalize([p.code, p.name, p.client].join(" ")).includes(keyword);
      return matchesCode && matchesName && matchesKeyword;
    });
  });

  function resetFilters() {
    filters.value = { code: "", keyword: "", name: "" };
  }

  function setFilters(f: ProjectFilters) {
    filters.value = f;
  }

  async function removeProject(projectId: number): Promise<boolean> {
    loadError.value = "";
    try {
      await deleteProjectCommand(projectId);
      projects.value = projects.value.filter((p) => p.id !== projectId);
      return true;
    } catch (e) {
      loadError.value = friendlyError(e);
      return false;
    }
  }

  return {
    filters,
    filteredProjects,
    isLoading,
    loadError,
    loadProjects,
    projects,
    removeProject,
    resetFilters,
    setFilters,
  };
}
