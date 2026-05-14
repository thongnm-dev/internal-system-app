import { useReportDataController } from "./useReportDataController";

export function usePhasesController() {
  const { result, summaryMetrics } = useReportDataController();

  return {
    result,
    summaryMetrics,
  };
}
