import { computed, ref } from "vue";
import { useAuthStore } from "@/app/stores/auth";
import { useToast } from "@/shared/composables/useToast";
import { friendlyError } from "@/tauri/commands/_base";
import { aiTaskCreate, aiTaskList, aiTaskUpdate } from "@/tauri/commands/ai-task";
import type { AiTaskResult } from "@/tauri/commands/ai-task";
import type { TaskDialogPayload } from "../components/AiTaskDialog.vue";

export type TaskFilters = {
  keyword: string;
};

export function useAiTasks() {
  const toast = useToast();
  const auth = useAuthStore();
  const username = computed(() => auth.user?.username ?? "");

  const filters = ref<TaskFilters>({ keyword: "" });
  const tasks = ref<AiTaskResult[]>([]);
  const isLoading = ref(false);
  const loadError = ref("");

  async function search() {
    isLoading.value = true;
    loadError.value = "";
    try {
      const kw = filters.value.keyword.trim() || undefined;
      tasks.value = await aiTaskList(kw);
    } catch (e) {
      loadError.value = friendlyError(e);
      tasks.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  function resetFilters() {
    filters.value = { keyword: "" };
  }

  async function createTask(payload: TaskDialogPayload) {
    if (!payload.task_cd || !username.value) return;
    const created = await aiTaskCreate(username.value, {
      task_cd: payload.task_cd,
      task_name: payload.task_name,
      category: payload.category,
    });
    tasks.value = [created, ...tasks.value];
    toast.success("Task created.");
  }

  async function saveTask(id: number, payload: TaskDialogPayload) {
    if (!payload.task_cd || !username.value) return;
    const updated = await aiTaskUpdate(id, username.value, {
      task_cd: payload.task_cd,
      task_name: payload.task_name,
      category: payload.category,
      is_complete: payload.is_complete,
    });
    const idx = tasks.value.findIndex((t) => t.id === updated.id);
    if (idx !== -1) tasks.value[idx] = updated;
    toast.success("Task updated.");
  }

  search();

  return {
    filters,
    tasks,
    isLoading,
    loadError,
    search,
    resetFilters,
    createTask,
    saveTask,
  };
}
