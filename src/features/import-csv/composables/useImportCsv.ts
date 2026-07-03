import { open } from "@tauri-apps/plugin-dialog";
import { onMounted, ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError, importMonthlyReportCsv, listImportBatches, previewMonthlyReportCsv } from "@/tauri/commands";
import type { MessageMode } from "@/shared/types/app";
import type { ImportBatchSummary, ImportCsvPreviewResult, ImportCsvResult } from "@/shared/types/import-csv";

function defaultReportName(path: string) {
  const fileName = path.split(/[\\/]/).pop() ?? path;
  return fileName.replace(/\.csv$/i, "");
}

export function useImportCsv() {
  const csvPath = ref("");
  const reportName = ref("");
  const note = ref("");
  const previewResult = ref<ImportCsvPreviewResult | null>(null);
  const result = ref<ImportCsvResult | null>(null);
  const batches = ref<ImportBatchSummary[]>([]);
  const message = ref("No CSV imported. Upload a CSV file to create monthly report check data.");
  const messageMode = ref<MessageMode>("info");
  const isImporting = ref(false);
  const isSaving = ref(false);

  function updateCsvPath(value: string) {
    csvPath.value = value;
    if (!reportName.value) reportName.value = defaultReportName(value);
  }

  async function refreshBatches() {
    try {
      batches.value = await listImportBatches();
    } catch {
      batches.value = [];
    }
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
        reportName.value = defaultReportName(selected);
        note.value = "";
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

  async function saveCsv() {
    if (!previewResult.value) {
      message.value = "Please import a CSV preview before saving.";
      messageMode.value = "error";
      return;
    }
    isSaving.value = true;
    message.value = "Saving imported CSV rows to database...";
    messageMode.value = "info";
    try {
      const r = await importMonthlyReportCsv(csvPath.value, reportName.value, note.value);
      result.value = r;
      message.value = `Saved ${r.row_count.toLocaleString("en-US")} rows to database.`;
      messageMode.value = "info";
      await refreshBatches();
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      isSaving.value = false;
    }
  }

  onMounted(() => void refreshBatches());

  return {
    batches, csvPath, isImporting, isSaving, message, messageMode, note,
    pickCsvFile, previewCsv, reportName, previewResult, result, saveCsv,
    updateCsvPath, setNote: (v: string) => { note.value = v; }, setReportName: (v: string) => { reportName.value = v; },
  };
}
