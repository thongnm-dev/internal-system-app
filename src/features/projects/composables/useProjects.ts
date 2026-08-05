import { ref } from "vue";
import type { MonthlyReportSummary } from "@/_/types/check-monthly-report";

export type ProjectFilters = {
  code: string;
  keyword: string;
  name: string;
};

export function useProjects() {
  const filters = ref<ProjectFilters>({ code: "", keyword: "", name: "" });
  const result = ref<MonthlyReportSummary | null>(null);

  function searchProjects() {}
  function resetFilters() { filters.value = { code: "", keyword: "", name: "" }; }

  return { filters, isSearching: false, result, resetFilters, searchError: "", searchProjects, setFilters: (f: ProjectFilters) => { filters.value = f; } };
}
