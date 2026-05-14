import type { SelectedPhaseDetail } from "../types/statistics";
import { useReportDataController } from "./useReportDataController";

export function useProjectsController(onPhaseClick: (detail: SelectedPhaseDetail) => void) {
  const { result, summaryMetrics } = useReportDataController();

  return {
    onPhaseClick,
    result,
    summaryMetrics,
  };
}
