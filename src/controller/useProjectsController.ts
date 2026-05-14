import { useMemo, useState } from "react";
import { friendlyError, safeInvoke } from "../core/tauriRuntime";
import { useReportDataController } from "./useReportDataController";

export type ProjectFilters = {
  code: string;
  keyword: string;
  name: string;
};

type ApiKeyApplication = {
  application_name: string;
  id: number;
};

export function useProjectsController() {
  const { result, summaryMetrics } = useReportDataController();
  const [filters, setFilters] = useState<ProjectFilters>({
    code: "",
    keyword: "",
    name: "",
  });
  const [apiKeyApplications, setApiKeyApplications] = useState<ApiKeyApplication[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [searchError, setSearchError] = useState("");

  const applicationName = useMemo(() => {
    const names = apiKeyApplications.map((application) => application.application_name.trim()).filter(Boolean);
    return names.length > 0 ? names.join(", ") : "-";
  }, [apiKeyApplications]);

  const searchProjects = async () => {
    setIsSearching(true);
    setSearchError("");
    try {
      const applications = await safeInvoke<ApiKeyApplication[]>("list_api_key_applications");
      setApiKeyApplications(applications);
    } catch (error) {
      setApiKeyApplications([]);
      setSearchError(friendlyError(error));
    } finally {
      setIsSearching(false);
    }
  };

  const resetFilters = () => {
    setFilters({
      code: "",
      keyword: "",
      name: "",
    });
  };

  return {
    applicationName,
    filters,
    isSearching,
    result,
    resetFilters,
    searchError,
    searchProjects,
    setFilters,
    summaryMetrics,
  };
}
