import { useState } from "react";
import { useReportDataController } from "./useReportDataController";

export type ProjectFilters = {
  code: string;
  keyword: string;
  name: string;
};

export function useProjectsController() {
  const { result, summaryMetrics } = useReportDataController();
  const [filters, setFilters] = useState<ProjectFilters>({
    code: "",
    keyword: "",
    name: "",
  });

  const searchProjects = () => {
    // Filtering is local; this action remains available for the existing search UI flow.
  };

  const resetFilters = () => {
    setFilters({
      code: "",
      keyword: "",
      name: "",
    });
  };

  return {
    filters,
    isSearching: false,
    result,
    resetFilters,
    searchError: "",
    searchProjects,
    setFilters,
    summaryMetrics,
  };
}
