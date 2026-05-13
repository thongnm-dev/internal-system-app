import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";
import { defaultCsvPath, tauriRuntimeMessage } from "../config/appConfig";
import { canUseTauriRuntime, friendlyError, safeInvoke } from "../core/tauriRuntime";
import { formatHours, overtimeMinutes, totalMinutes } from "../core/timeMath";
import type {
  AnalysisResult,
  MenuKey,
  MessageMode,
  SelectedPhaseDetail,
  SummaryMetric,
  SystemInfo,
} from "../types/statistics";

export function useDashboardController() {
  const [csvPath, setCsvPath] = useState(defaultCsvPath);
  const [result, setResult] = useState<AnalysisResult | null>(null);
  const [systemInfo, setSystemInfo] = useState<SystemInfo>({
    username: "-",
    timestamp: "-",
    ip_address: "-",
    version: "-",
  });
  const [message, setMessage] = useState("Ready to read CSV data.");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");
  const [activeMenu, setActiveMenu] = useState<MenuKey>("overview");
  const [selectedPhaseDetail, setSelectedPhaseDetail] = useState<SelectedPhaseDetail | null>(null);
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

  const refreshSystemInfo = async () => {
    try {
      const info = await safeInvoke<SystemInfo>("get_system_info");
      setSystemInfo(info);
    } catch {
      setSystemInfo((current) => ({
        ...current,
        timestamp: new Date().toLocaleString(),
      }));
    }
  };

  useEffect(() => {
    void analyze(defaultCsvPath);
    void refreshSystemInfo();
    const timer = window.setInterval(() => void refreshSystemInfo(), 1000);
    return () => window.clearInterval(timer);
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

  const pickCsvFile = async () => {
    if (!canUseTauriRuntime()) {
      setMessage(tauriRuntimeMessage);
      setMessageMode("error");
      return;
    }

    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });

      if (typeof selected === "string") {
        setCsvPath(selected);
        void analyze(selected);
      }
    } catch (error) {
      setMessage(friendlyError(error));
      setMessageMode("error");
    }
  };

  return {
    activeMenu,
    analyze,
    csvPath,
    isLoading,
    message,
    messageMode,
    pickCsvFile,
    result,
    setActiveMenu,
    setCsvPath,
    selectedPhaseDetail,
    setSelectedPhaseDetail,
    summaryMetrics,
    systemInfo,
  };
}
