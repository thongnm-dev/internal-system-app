import { ref, computed } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import {
  listStoredProcedures,
  getStoredProcedureContent,
  executeSingleStoredProcedure,
} from "@/tauri/commands/app-config";
import type { SpInfo, SpExecutionResult, SpExecutionSummary } from "@/_/types/app-config";

const GROUP_PREFIXES: [string, string][] = [
  ["sp_ai_task_wf_proc_step_", "AI Task WF Proc Step"],
  ["sp_ai_task_wf_proc_", "AI Task WF Proc"],
  ["sp_ai_task_", "AI Task"],
  ["sp_ai_workflow_", "AI Workflow"],
  ["sp_project_task_", "Project Task"],
  ["sp_project_", "Project"],
  ["sp_auth_", "Auth"],
  ["sp_daily_note_", "Daily Work Notes"],
  ["sp_daily_report_", "Daily Report"],
  ["sp_category_", "Category"],
  ["sp_user_", "User Management"],
  ["sp_role_", "User Management"],
  ["sp_menu_config_", "Menu Config"],
  ["sp_menu_permission_", "Menu Permission"],
  ["sp_aws_", "AWS Storage"],
  ["sp_download_", "Download History"],
  ["sp_upload_", "Upload History"],
];

function resolveGroup(name: string): string {
  for (const [prefix, group] of GROUP_PREFIXES) {
    if (name.startsWith(prefix)) return group;
  }
  return "Other";
}

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
  const filterGroup = ref("");

  const groupOptions = computed(() => {
    const groups = new Set<string>();
    for (const p of procedures.value) groups.add(resolveGroup(p.name));
    return [...groups].sort();
  });

  const filteredResults = computed(() => {
    let list = procedures.value.map((p) => ({
      name: p.name,
      success: true,
      message: "",
    }));

    if (filterText.value) {
      const q = filterText.value.toLowerCase();
      list = list.filter((r) => r.name.toLowerCase().includes(q));
    }

    if (filterGroup.value) {
      list = list.filter((r) => resolveGroup(r.name) === filterGroup.value);
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

  async function executeFiltered(): Promise<boolean> {
    if (!canUseTauriRuntime()) return false;
    const names = filteredResults.value.map((r) => r.name);
    if (names.length === 0) return true;
    executing.value = true;
    error.value = "";
    globalLoading.start();
    try {
      const execResults: SpExecutionResult[] = [];
      let successCount = 0;
      for (const name of names) {
        const result = await executeSingleStoredProcedure(name);
        execResults.push(result);
        if (result.success) successCount++;
      }
      summary.value = {
        total: names.length,
        success_count: successCount,
        error_count: names.length - successCount,
        results: execResults,
      };
      results.value = execResults;
      return names.length === successCount;
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

  function resetFilters() {
    filterText.value = "";
    filterGroup.value = "";
  }

  function resetResults() {
    results.value = [];
    summary.value = null;
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
    filterGroup,
    groupOptions,
    filteredResults,
    hasResults,
    executingNames,
    viewingName,
    viewingContent,
    viewingLoading,
    executeFiltered,
    executeSingle,
    viewScript,
    closeViewer,
    resetFilters,
    resetResults,
    init,
  };
}
