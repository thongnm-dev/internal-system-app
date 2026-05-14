import { createContext, type ReactNode, useContext, useEffect, useMemo, useState } from "react";
import { defaultCsvPath } from "../config/appConfig";
import { friendlyError, safeInvoke } from "../core/tauriRuntime";
import { formatHours, overtimeMinutes, totalMinutes } from "../core/timeMath";
import type { AnalysisResult, MessageMode, SummaryMetric } from "../types/statistics";

type ReportDataContextValue = {
  analyze: (path?: string) => Promise<void>;
  csvPath: string;
  isLoading: boolean;
  message: string;
  messageMode: MessageMode;
  result: AnalysisResult | null;
  setCsvPath: (value: string) => void;
  summaryMetrics: SummaryMetric[];
};

const ReportDataContext = createContext<ReportDataContextValue | null>(null);

export function ReportDataProvider({ children }: { children: ReactNode }) {
  const [csvPath, setCsvPath] = useState(defaultCsvPath);
  const [result, setResult] = useState<AnalysisResult | null>(null);
  const [message, setMessage] = useState("Ready to read CSV data.");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");
  const [isLoading, setIsLoading] = useState(false);

  const analyze = async (path = csvPath) => {
    setIsLoading(true);
    setMessage("Reading and analyzing CSV...");
    setMessageMode("info");
    try {
      const analysis = await safeInvoke<AnalysisResult>("analyze_csv", { path });
      setResult(analysis);
      setMessage(`Analyzed ${analysis.row_count.toLocaleString("en-US")} rows.`);
    } catch (error) {
      setMessage(friendlyError(error));
      setMessageMode("error");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    void analyze(defaultCsvPath);
  }, []);

  const summaryMetrics = useMemo<SummaryMetric[]>(() => {
    if (!result) {
      return [
        { label: "Rows", value: "-" },
        { label: "Total hours", value: "-" },
        { label: "Regular hours", value: "-" },
        { label: "Overtime", value: "-" },
      ];
    }

    return [
      { label: "Rows", value: result.row_count.toLocaleString("en-US") },
      { label: "Total hours", value: formatHours(totalMinutes(result.grand_total)) },
      { label: "Regular hours", value: formatHours(result.grand_total.regular_minutes) },
      { label: "Overtime", value: formatHours(overtimeMinutes(result.grand_total)) },
    ];
  }, [result]);

  return (
    <ReportDataContext.Provider
      value={{
        analyze,
        csvPath,
        isLoading,
        message,
        messageMode,
        result,
        setCsvPath,
        summaryMetrics,
      }}
    >
      {children}
    </ReportDataContext.Provider>
  );
}

export function useReportDataController() {
  const context = useContext(ReportDataContext);
  if (!context) {
    throw new Error("useReportDataController must be used inside ReportDataProvider.");
  }
  return context;
}
