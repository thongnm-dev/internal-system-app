import { useEffect, useState } from "react";
import { tauriRuntimeMessage } from "../config/appConfig";
import { canUseTauriRuntime, friendlyError, safeInvoke } from "../core/tauriRuntime";
import type { ImportReportDetail, ImportReportListItem, ImportReportSearchCriteria, MessageMode } from "../types/statistics";

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

export function useImportReportDetailController(reportId: number | null) {
  const [detail, setDetail] = useState<ImportReportDetail | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState("Loading imported report detail.");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");

  useEffect(() => {
    if (!reportId) {
      setDetail(null);
      setMessage("Import report ID is missing.");
      setMessageMode("error");
      return;
    }

    const loadDetail = async () => {
      if (!canUseTauriRuntime()) {
        setDetail(null);
        setMessage(tauriRuntimeMessage);
        setMessageMode("error");
        return;
      }

      setIsLoading(true);
      setMessage(`Loading imported report #${reportId.toLocaleString("en-US")}...`);
      setMessageMode("info");
      try {
        const result = await safeInvoke<ImportReportDetail>("get_import_batch_detail", {
          batchId: reportId,
        });
        setDetail(result);
        setMessage("Imported report detail loaded.");
        setMessageMode("info");
      } catch (error) {
        setDetail(null);
        setMessage(friendlyError(error));
        setMessageMode("error");
      } finally {
        setIsLoading(false);
      }
    };

    void loadDetail();
  }, [reportId]);

  return {
    detail,
    isLoading,
    message,
    messageMode,
  };
}
