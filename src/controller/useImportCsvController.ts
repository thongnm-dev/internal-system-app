import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { tauriRuntimeMessage } from "../config/appConfig";
import { canUseTauriRuntime, friendlyError, safeInvoke } from "../core/tauriRuntime";
import type { ImportBatchSummary, ImportCsvPreviewResult, ImportCsvResult, MessageMode } from "../types/statistics";

export function useImportCsvController() {
  const [csvPath, setCsvPath] = useState("");
  const [previewResult, setPreviewResult] = useState<ImportCsvPreviewResult | null>(null);
  const [result, setResult] = useState<ImportCsvResult | null>(null);
  const [batches, setBatches] = useState<ImportBatchSummary[]>([]);
  const [message, setMessage] = useState("No CSV imported. Upload a CSV file to create monthly report check data.");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");
  const [isImporting, setIsImporting] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const refreshBatches = async () => {
    try {
      const batches = await safeInvoke<ImportBatchSummary[]>("list_import_batches");
      setBatches(batches);
    } catch {
      setBatches([]);
    }
  };

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
        setPreviewResult(null);
        setResult(null);
        setMessage("CSV selected. Click Import to preview it.");
        setMessageMode("info");
      }
    } catch (error) {
      setMessage(friendlyError(error));
      setMessageMode("error");
    }
  };

  const previewCsv = async () => {
    if (!csvPath.trim()) {
      setMessage("Please select a CSV file before importing.");
      setMessageMode("error");
      return;
    }

    setIsImporting(true);
    setMessage("Importing CSV for preview...");
    setMessageMode("info");
    try {
      const result = await safeInvoke<ImportCsvPreviewResult>("preview_monthly_report_csv", { path: csvPath });
      setPreviewResult(result);
      setResult(null);
      setMessage(`Imported ${result.row_count.toLocaleString("en-US")} rows for preview. Click Save to store them.`);
      setMessageMode("info");
    } catch (error) {
      setMessage(friendlyError(error));
      setMessageMode("error");
    } finally {
      setIsImporting(false);
    }
  };

  const saveCsv = async () => {
    if (!previewResult) {
      setMessage("Please import a CSV preview before saving.");
      setMessageMode("error");
      return;
    }

    setIsSaving(true);
    setMessage("Saving imported CSV rows to database...");
    setMessageMode("info");
    try {
      const result = await safeInvoke<ImportCsvResult>("import_monthly_report_csv", { path: csvPath });
      setResult(result);
      setMessage(`Saved ${result.row_count.toLocaleString("en-US")} rows to database.`);
      setMessageMode("info");
      await refreshBatches();
    } catch (error) {
      setMessage(friendlyError(error));
      setMessageMode("error");
    } finally {
      setIsSaving(false);
    }
  };

  useEffect(() => {
    void refreshBatches();
  }, []);

  return {
    batches,
    csvPath,
    isImporting,
    isSaving,
    message,
    messageMode,
    pickCsvFile,
    previewCsv,
    previewResult,
    result,
    saveCsv,
    setCsvPath,
  };
}
