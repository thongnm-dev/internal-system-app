import { ref, computed } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import {
  listStoredProcedures,
  getStoredProcedureContent,
  executeSingleStoredProcedure,
  executeStoredProcedures,
} from "@/tauri/commands/app-config";
import type { SpInfo, SpExecutionResult, SpExecutionSummary } from "@/_/types/app-config";

export function useStoreProcedure() {
  const globalLoading = useGlobalLoading();
  const procedures = ref<SpInfo[]>([]);
  const results = ref<SpExecutionResult[]>([]);
  const summary = ref<SpExecutionSummary | null>(null);
  const loading = ref(false);
  const executing = ref(false);
  const executingNames = ref<Set<string>>(new Set());
  const error = ref("");
  const viewingName = ref("");
  const viewingContent = ref("");
  const viewingLoading = ref(false);
  const filterText = ref("");
  const filterStatus = ref<"all" | "success" | "error">("all");

  const filteredResults = computed(() => {
    let list = results.value.length > 0 ? results.value : procedures.value.map((p) => ({
      name: p.name,
      success: true,
      message: "",
    }));

    if (filterText.value) {
      const q = filterText.value.toLowerCase();
      list = list.filter((r) => r.name.toLowerCase().includes(q));
    }

    if (filterStatus.value === "success" && results.value.length > 0) {
      list = list.filter((r) => r.success);
    } else if (filterStatus.value === "error" && results.value.length > 0) {
      list = list.filter((r) => !r.success);
    }

    return list;
  });

  const hasResults = computed(() => results.value.length > 0);

  async function load() {
    if (!canUseTauriRuntime()) return;
    loading.value = true;
    error.value = "";
    try {
      procedures.value = await listStoredProcedures();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function executeAll(): Promise<boolean> {
    if (!canUseTauriRuntime()) return false;
    executing.value = true;
    error.value = "";
    globalLoading.start();
    try {
      const data = await executeStoredProcedures();
      summary.value = data;
      results.value = data.results;
      return data.error_count === 0;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      executing.value = false;
      globalLoading.stop();
    }
  }

  async function executeSingle(name: string): Promise<boolean> {
    if (!canUseTauriRuntime()) return false;
    executingNames.value = new Set([...executingNames.value, name]);
    try {
      const result = await executeSingleStoredProcedure(name);
      const idx = results.value.findIndex((r) => r.name === name);
      if (idx >= 0) {
        results.value[idx] = result;
      } else {
        results.value = [...results.value, result];
      }
      if (summary.value) {
        const oldResult = summary.value.results.find((r) => r.name === name);
        const wasSuccess = oldResult?.success ?? false;
        if (wasSuccess && !result.success) {
          summary.value.success_count--;
          summary.value.error_count++;
        } else if (!wasSuccess && result.success) {
          summary.value.success_count++;
          summary.value.error_count--;
        }
        const sIdx = summary.value.results.findIndex((r) => r.name === name);
        if (sIdx >= 0) summary.value.results[sIdx] = result;
        else summary.value.results.push(result);
      } else {
        summary.value = {
          total: 1,
          success_count: result.success ? 1 : 0,
          error_count: result.success ? 0 : 1,
          results: [result],
        };
      }
      return result.success;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      const next = new Set(executingNames.value);
      next.delete(name);
      executingNames.value = next;
    }
  }

  async function viewScript(name: string) {
    if (!canUseTauriRuntime()) return;
    viewingName.value = name;
    viewingContent.value = "";
    viewingLoading.value = true;
    try {
      viewingContent.value = await getStoredProcedureContent(name);
    } catch (e) {
      viewingContent.value = String(e);
    } finally {
      viewingLoading.value = false;
    }
  }

  function closeViewer() {
    viewingName.value = "";
    viewingContent.value = "";
  }

  function resetResults() {
    results.value = [];
    summary.value = null;
    filterStatus.value = "all";
  }

  function init() {
    globalLoading.run(() => load());
  }

  return {
    procedures,
    results,
    summary,
    loading,
    executing,
    error,
    filterText,
    filterStatus,
    filteredResults,
    hasResults,
    executingNames,
    viewingName,
    viewingContent,
    viewingLoading,
    executeAll,
    executeSingle,
    viewScript,
    closeViewer,
    resetResults,
    init,
  };
}
