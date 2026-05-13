import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";
import { defaultCsvPath, tauriRuntimeMessage } from "../config/appConfig";
import { canUseTauriRuntime, friendlyError, safeInvoke } from "../core/tauriRuntime";
import { formatHours, overtimeMinutes, totalMinutes } from "../core/timeMath";
import type {
  AnalysisResult,
  ImportBatchSummary,
  ImportCsvPreviewResult,
  ImportCsvResult,
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
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [importCsvPath, setImportCsvPath] = useState("");
  const [importPreviewResult, setImportPreviewResult] = useState<ImportCsvPreviewResult | null>(null);
  const [importResult, setImportResult] = useState<ImportCsvResult | null>(null);
  const [importBatches, setImportBatches] = useState<ImportBatchSummary[]>([]);
  const [importMessage, setImportMessage] = useState("No CSV imported. Upload a CSV file to create monthly report check data.");
  const [importMessageMode, setImportMessageMode] = useState<MessageMode>("info");
  const [isImporting, setIsImporting] = useState(false);
  const [isSavingImport, setIsSavingImport] = useState(false);

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
    void refreshImportBatches();
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

  const pickImportCsvFile = async () => {
    if (!canUseTauriRuntime()) {
      setImportMessage(tauriRuntimeMessage);
      setImportMessageMode("error");
      return;
    }

    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });

      if (typeof selected === "string") {
        setImportCsvPath(selected);
        setImportPreviewResult(null);
        setImportResult(null);
        setImportMessage("CSV selected. Click Import to preview it.");
        setImportMessageMode("info");
      }
    } catch (error) {
      setImportMessage(friendlyError(error));
      setImportMessageMode("error");
    }
  };

  const refreshImportBatches = async () => {
    try {
      const batches = await safeInvoke<ImportBatchSummary[]>("list_import_batches");
      setImportBatches(batches);
    } catch {
      setImportBatches([]);
    }
  };

  const previewMonthlyReportCsv = async () => {
    if (!importCsvPath.trim()) {
      setImportMessage("Please select a CSV file before importing.");
      setImportMessageMode("error");
      return;
    }

    setIsImporting(true);
    setImportMessage("Importing CSV for preview...");
    setImportMessageMode("info");
    try {
      const result = await safeInvoke<ImportCsvPreviewResult>("preview_monthly_report_csv", { path: importCsvPath });
      setImportPreviewResult(result);
      setImportResult(null);
      setImportMessage(`Imported ${result.row_count.toLocaleString("en-US")} rows for preview. Click Save to store them.`);
      setImportMessageMode("info");
    } catch (error) {
      setImportMessage(friendlyError(error));
      setImportMessageMode("error");
    } finally {
      setIsImporting(false);
    }
  };

  const saveMonthlyReportCsv = async () => {
    if (!importPreviewResult) {
      setImportMessage("Please import a CSV preview before saving.");
      setImportMessageMode("error");
      return;
    }

    setIsSavingImport(true);
    setImportMessage("Saving imported CSV rows to database...");
    setImportMessageMode("info");
    try {
      const result = await safeInvoke<ImportCsvResult>("import_monthly_report_csv", { path: importCsvPath });
      setImportResult(result);
      setImportMessage(`Saved ${result.row_count.toLocaleString("en-US")} rows to database.`);
      setImportMessageMode("info");
      await refreshImportBatches();
    } catch (error) {
      setImportMessage(friendlyError(error));
      setImportMessageMode("error");
    } finally {
      setIsSavingImport(false);
    }
  };

  return {
    activeMenu,
    analyze,
    csvPath,
    importBatches,
    importCsvPath,
    importMessage,
    importMessageMode,
    importMonthlyReportCsv: previewMonthlyReportCsv,
    importPreviewResult,
    importResult,
    isSidebarCollapsed,
    isImporting,
    isSavingImport,
    isLoading,
    message,
    messageMode,
    pickCsvFile,
    pickImportCsvFile,
    refreshImportBatches,
    saveMonthlyReportCsv,
    result,
    setActiveMenu,
    setCsvPath,
    setImportCsvPath,
    setIsSidebarCollapsed,
    selectedPhaseDetail,
    setSelectedPhaseDetail,
    summaryMetrics,
    systemInfo,
  };
}
