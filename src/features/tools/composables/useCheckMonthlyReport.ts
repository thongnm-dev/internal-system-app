import { open } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { compareMonthlyReport, previewMonthlyReportCsv } from "@/tauri/commands/check-monthly-report";
import type { MessageMode } from "@/_/types/app";
import type { CompareRow, ImportCsvPreviewResult, ImportCsvResult } from "@/_/types/check-monthly-report";

export function useCheckMonthlyReport() {
  const csvPath = ref("");
  const previewResult = ref<ImportCsvPreviewResult | null>(null);
  const result = ref<ImportCsvResult | null>(null);
  const message = ref("No CSV imported. Upload a CSV file to create monthly report check data.");
  const messageMode = ref<MessageMode>("info");
  const isImporting = ref(false);

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

  async function runCompare() {
    if (!csvPath.value || !schedulePath.value) return;
    isComparing.value = true;
    try {
      const year = targetMonth.value.getFullYear();
      const month = targetMonth.value.getMonth() + 1;
      compareResult.value = await compareMonthlyReport(
        csvPath.value,
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
    schedulePath, targetMonth, targetUser,
    compareResult, compareTotals, isComparing, runCompare,
  };
}
