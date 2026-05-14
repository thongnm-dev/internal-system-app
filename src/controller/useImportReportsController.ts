import { useEffect, useState } from "react";
import { tauriRuntimeMessage } from "../config/appConfig";
import { canUseTauriRuntime, friendlyError, safeInvoke } from "../core/tauriRuntime";
import type { ImportReportListItem, ImportReportSearchCriteria, MessageMode } from "../types/statistics";

const emptyCriteria: ImportReportSearchCriteria = {
  target_month_from: "",
  target_month_to: "",
  report_name: "",
  keyword: "",
};

export function useImportReportsController() {
  const [criteria, setCriteria] = useState<ImportReportSearchCriteria>(emptyCriteria);
  const [items, setItems] = useState<ImportReportListItem[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [message, setMessage] = useState("Search imported monthly reports saved from the CSV import screen.");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");

  const search = async (nextCriteria = criteria) => {
    if (!canUseTauriRuntime()) {
      setItems([]);
      setMessage(tauriRuntimeMessage);
      setMessageMode("error");
      return;
    }

    setIsSearching(true);
    setMessage("Searching imported reports...");
    setMessageMode("info");
    try {
      const result = await safeInvoke<ImportReportListItem[]>("search_import_batches", {
        criteria: nextCriteria,
      });
      setItems(result);
      setMessage(`Found ${result.length.toLocaleString("en-US")} imported reports.`);
      setMessageMode("info");
    } catch (error) {
      setItems([]);
      setMessage(friendlyError(error));
      setMessageMode("error");
    } finally {
      setIsSearching(false);
    }
  };

  const reset = () => {
    setCriteria(emptyCriteria);
    void search(emptyCriteria);
  };

  useEffect(() => {
    void search(emptyCriteria);
  }, []);

  return {
    criteria,
    isSearching,
    items,
    message,
    messageMode,
    reset,
    search,
    setCriteria,
  };
}
