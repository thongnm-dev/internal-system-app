import { open } from "@tauri-apps/plugin-dialog";
import { ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { previewMonthlyReportCsv } from "@/tauri/commands/check-monthly-report";
import type { MessageMode } from "@/_/types/app";
import type { ImportCsvPreviewResult, ImportCsvResult } from "@/_/types/check-monthly-report";

export function useCheckMonthlyReport() {
  const csvPath = ref("");
  const previewResult = ref<ImportCsvPreviewResult | null>(null);
  const result = ref<ImportCsvResult | null>(null);
  const message = ref("No CSV imported. Upload a CSV file to create monthly report check data.");
  const messageMode = ref<MessageMode>("info");
  const isImporting = ref(false);

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

  return {
    csvPath, isImporting, message, messageMode,
    pickCsvFile, previewCsv, previewResult, result,
    updateCsvPath,
  };
}
