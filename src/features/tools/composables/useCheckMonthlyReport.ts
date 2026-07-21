import { open } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { compareMonthlyReport, fetchCsvFromSystem, previewMonthlyReportCsv } from "@/tauri/commands/check-monthly-report";
import { getStaffNo } from "@/tauri/commands/user";
import type { MessageMode } from "@/_/types/app";
import type { CompareRow, ImportCsvPreviewResult, ImportCsvResult } from "@/_/types/check-monthly-report";

export function useCheckMonthlyReport() {
  const csvPath = ref("");
  const previewResult = ref<ImportCsvPreviewResult | null>(null);
  const result = ref<ImportCsvResult | null>(null);
  const message = ref("No CSV imported. Upload a CSV file to create monthly report check data.");
  const messageMode = ref<MessageMode>("info");
  const isImporting = ref(false);

  const autoFetchEnabled = ref(false);
  const autoFetchedCsvPath = ref("");
  const isFetchingFromSystem = ref(false);

  const schedulePath = ref("");
  const targetMonth = ref<Date>(new Date());
  const targetUser = ref("");
  const compareResult = ref<CompareRow[] | null>(null);
  const isComparing = ref(false);

  const compareTotals = computed(() => {
    const rows = compareResult.value ?? [];
    const csv = rows.reduce((s, r) => s + r.csv_hours, 0);
    const schedule = rows.reduce((s, r) => s + r.schedule_hours, 0);
    const mismatches = rows.filter((r) => r.status !== "match" && r.status !== "csv-only-warning").length;
    const warnings = rows.filter((r) => r.status === "csv-only-warning").length;
    return { csv, schedule, diff: csv - schedule, mismatches, warnings };
  });

  function updateCsvPath(value: string) {
    csvPath.value = value;
  }

  async function pickCsvFile() {
    if (!canUseTauriRuntime()) {
      message.value = tauriRuntimeMessage;
      messageMode.value = "error";
      return;
    }
    try {
      const selected = await open({ multiple: false, filters: [{ name: "CSV", extensions: ["csv"] }] });
      if (typeof selected === "string") {
        csvPath.value = selected;
        previewResult.value = null;
        result.value = null;
        message.value = "CSV selected. Click Import to preview it.";
        messageMode.value = "info";
      }
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    }
  }

  async function previewCsv() {
    if (!csvPath.value.trim()) {
      message.value = "Please select a CSV file before importing.";
      messageMode.value = "error";
      return;
    }
    isImporting.value = true;
    message.value = "Importing CSV for preview...";
    messageMode.value = "info";
    try {
      const r = await previewMonthlyReportCsv(csvPath.value);
      previewResult.value = r;
      result.value = null;
      message.value = `Imported ${r.row_count.toLocaleString("en-US")} rows for preview. Click Save to store them.`;
      messageMode.value = "info";
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      isImporting.value = false;
    }
  }

  async function resolveStaffNo(): Promise<string | null> {
    if (!targetUser.value.trim()) {
      message.value = "Target user is required for auto-fetch.";
      messageMode.value = "error";
      return null;
    }
    try {
      return await getStaffNo(targetUser.value);
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      return null;
    }
  }

  async function fetchFromSystem(): Promise<string | null> {
    const staff = await resolveStaffNo();
    if (!staff) return null;
    isFetchingFromSystem.value = true;
    try {
      const year = targetMonth.value.getFullYear();
      const month = targetMonth.value.getMonth() + 1;
      const lastDay = new Date(year, month, 0).getDate();
      const mm = String(month).padStart(2, "0");
      const dateFrom = `${year}/${mm}/01`;
      const dateTo = `${year}/${mm}/${String(lastDay).padStart(2, "0")}`;
      const path = await fetchCsvFromSystem(dateFrom, dateTo, staff);
      autoFetchedCsvPath.value = path;
      return path;
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      return null;
    } finally {
      isFetchingFromSystem.value = false;
    }
  }

  async function previewAutoFetch() {
    const path = await fetchFromSystem();
    if (!path) return;
    isImporting.value = true;
    message.value = "Fetching CSV from system for preview...";
    messageMode.value = "info";
    try {
      const r = await previewMonthlyReportCsv(path);
      previewResult.value = r;
      result.value = null;
      message.value = `Fetched ${r.row_count.toLocaleString("en-US")} rows from system.`;
      messageMode.value = "info";
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      isImporting.value = false;
    }
  }

  async function runCompare(source?: "csv" | "auto") {
    const resolvedSource = source ?? "csv";
    let effectiveCsvPath = csvPath.value;

    if (resolvedSource === "auto") {
      const path = await fetchFromSystem();
      if (!path) return;
      effectiveCsvPath = path;
    }

    if (!effectiveCsvPath || !schedulePath.value) return;
    isComparing.value = true;
    try {
      const year = targetMonth.value.getFullYear();
      const month = targetMonth.value.getMonth() + 1;
      compareResult.value = await compareMonthlyReport(
        effectiveCsvPath,
        schedulePath.value,
        year,
        month,
        targetUser.value || undefined,
      );
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
      compareResult.value = null;
    } finally {
      isComparing.value = false;
    }
  }

  return {
    csvPath, isImporting, message, messageMode,
    pickCsvFile, previewCsv, previewResult, result,
    updateCsvPath,
    autoFetchEnabled, autoFetchedCsvPath, isFetchingFromSystem,
    previewAutoFetch,
    schedulePath, targetMonth, targetUser,
    compareResult, compareTotals, isComparing, runCompare,
  };
}
