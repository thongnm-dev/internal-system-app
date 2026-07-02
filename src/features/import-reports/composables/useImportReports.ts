import { onMounted, ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError, getImportBatchDetail, searchImportBatches } from "@/tauri/commands";
import type { ImportReportDetail, ImportReportListItem, ImportReportSearchCriteria, MessageMode } from "@/shared/types/statistics";

const emptyCriteria: ImportReportSearchCriteria = {
  target_month_from: "",
  target_month_to: "",
  report_name: "",
  keyword: "",
};

export function useImportReports() {
  const criteria = ref<ImportReportSearchCriteria>({ ...emptyCriteria });
  const items = ref<ImportReportListItem[]>([]);
  const isSearching = ref(false);
  const message = ref("Search imported monthly reports saved from the CSV import screen.");
  const messageMode = ref<MessageMode>("info");

  async function search(nextCriteria = criteria.value) {
    if (!canUseTauriRuntime()) {
      items.value = [];
      message.value = tauriRuntimeMessage;
      messageMode.value = "error";
      return;
    }
    isSearching.value = true;
    message.value = "Searching imported reports...";
    messageMode.value = "info";
    try {
      const result = await searchImportBatches(nextCriteria);
      items.value = result;
      message.value = `Found ${result.length.toLocaleString("en-US")} imported reports.`;
      messageMode.value = "info";
    } catch (e) {
      items.value = [];
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      isSearching.value = false;
    }
  }

  function reset() {
    criteria.value = { ...emptyCriteria };
    void search({ ...emptyCriteria });
  }

  onMounted(() => void search({ ...emptyCriteria }));

  return { criteria, isSearching, items, message, messageMode, reset, search };
}

export function useImportReportDetail(reportId: number | null) {
  const detail = ref<ImportReportDetail | null>(null);
  const isLoading = ref(false);
  const message = ref("Loading imported report detail.");
  const messageMode = ref<MessageMode>("info");

  onMounted(async () => {
    if (!reportId) {
      message.value = "Import report ID is missing.";
      messageMode.value = "error";
      return;
    }
    if (!canUseTauriRuntime()) {
      message.value = tauriRuntimeMessage;
      messageMode.value = "error";
      return;
    }
    isLoading.value = true;
    message.value = `Loading imported report #${reportId.toLocaleString("en-US")}...`;
    messageMode.value = "info";
    try {
      detail.value = await getImportBatchDetail(reportId);
      message.value = "Imported report detail loaded.";
      messageMode.value = "info";
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      isLoading.value = false;
    }
  });

  return { detail, isLoading, message, messageMode };
}
