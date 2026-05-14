import { useState } from "react";
import type { SummaryMetric } from "../types/statistics";

export type ProjectFilters = {
  code: string;
  keyword: string;
  name: string;
};

const defaultSummaryMetrics: SummaryMetric[] = [
  { label: "Rows", value: "-" },
  { label: "Total hours", value: "-" },
  { label: "Regular hours", value: "-" },
  { label: "Overtime", value: "-" },
];

export function useProjectsController() {
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
    result: null,
    resetFilters,
    searchError: "",
    searchProjects,
    setFilters,
    summaryMetrics: defaultSummaryMetrics,
  };
}
