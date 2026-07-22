import { ref, computed } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import {
  listStoredProcedures,
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
  const error = ref("");
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
    executeAll,
    resetResults,
    init,
  };
}
