import { ref } from "vue";
import type { AnalysisResult } from "@/_/types/analysis";

export type ProjectFilters = {
  code: string;
  keyword: string;
  name: string;
};

export function useProjects() {
  const filters = ref<ProjectFilters>({ code: "", keyword: "", name: "" });
  const result = ref<AnalysisResult | null>(null);

  function searchProjects() {}
  function resetFilters() { filters.value = { code: "", keyword: "", name: "" }; }

  return { filters, isSearching: false, result, resetFilters, searchError: "", searchProjects, setFilters: (f: ProjectFilters) => { filters.value = f; } };
}
