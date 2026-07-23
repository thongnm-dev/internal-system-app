import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useAuthStore } from "@/app/stores/auth";
import { useToast } from "@/shared/composables/useToast";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { aiWorkflowList, aiWorkflowStepList } from "@/tauri/commands/ai-workflow";
import type { AiWorkflowResult, AiWorkflowStepResult } from "@/tauri/commands/ai-workflow";
import { aiUsageListAccounts, aiUsageOpenTerminal, aiUsageSetActive } from "@/tauri/commands/ai-usage";
import type { AiAccount } from "@/_/types/ai-usage";
import { explorerOpen, explorerReadDir, explorerReadTextFile } from "@/tauri/commands/explorer";
import type { FileEntry } from "@/tauri/commands/explorer";
import type { WorkflowStepType } from "@/_/types/ai-workflow";
import { aiTaskCreate, aiTaskList } from "@/tauri/commands/ai-task";
import type { AiTaskResult } from "@/tauri/commands/ai-task";
import type { TaskDialogPayload } from "../components/AiTaskDialog.vue";

export type CoworkWorkflow = {
  id: number;
  name: string;
  description: string;
  stepCount: number;
};

export type CoworkStep = {
  id: number;
  name: string;
  type: WorkflowStepType;
  skillName: string;
  description: string;
  icon: string;
  stepOrder: number;
};

function toWorkflow(r: AiWorkflowResult): CoworkWorkflow {
  return { id: r.id, name: r.name, description: r.description, stepCount: r.step_count };
}

function toStep(r: AiWorkflowStepResult): CoworkStep {
  return {
    id: r.id,
    name: r.name,
    type: r.step_type as WorkflowStepType,
    skillName: r.skill_name,
    description: r.description,
    icon: r.icon,
    stepOrder: r.step_order,
  };
}

export function useAiCowork() {
  const toast = useToast();
  const auth = useAuthStore();
  const username = computed(() => auth.user?.username ?? "");

  // Row 1 — project directory.
  const projectDir = ref("");
  const dirEntries = ref<FileEntry[]>([]);
  const isLoadingDir = ref(false);

  // Markdown preview dialog.
  const mdPreviewOpen = ref(false);
  const mdPreviewName = ref("");
  const mdPreviewContent = ref("");
  const isLoadingMdPreview = ref(false);

  // Column 1 — workflow picker + steps.
  const workflows = ref<CoworkWorkflow[]>([]);
  const selectedWorkflowId = ref<number | null>(null);
  const steps = ref<CoworkStep[]>([]);
  const isLoadingWorkflows = ref(false);
  const isApplying = ref(false);
  const appliedWorkflowId = ref<number | null>(null);

  // Skill folders phát hiện dưới `<projectDir>/.claude/skills` — dùng để enable nút "Open Terminal" của step type "skill".
  const skillFolders = ref<string[]>([]);
  const skillFolderNames = computed(() => skillFolders.value.map((s) => s.toLowerCase()));
  const showSkillWarning = ref(false);
  const openingTerminalStepId = ref<number | null>(null);
  const showSkillListDialog = ref(false);
  const isLoadingSkillList = ref(false);

  // Column 2 — AI accounts.
  const accounts = ref<AiAccount[]>([]);
  const isLoadingAccounts = ref(false);
  const settingActiveId = ref<number | null>(null);

  // Column "Tasks" — danh sách task đã chọn cho lần chạy này.
  const selectedTasks = ref<AiTaskResult[]>([]);
  // Task đã tick checkbox xác nhận lại (con trong danh sách selectedTasks).
  const confirmedTaskIds = ref<Set<number>>(new Set());

  // Task picker dialog (search + tạo mới, multi-select).
  const showTaskPicker = ref(false);
  const taskSearchQuery = ref("");
  const taskSearchResults = ref<AiTaskResult[]>([]);
  const isSearchingTasks = ref(false);
  const pickerSelected = ref<Map<number, AiTaskResult>>(new Map());
  const pickerSelectedCount = computed(() => pickerSelected.value.size);

  const selectedWorkflow = computed(
    () => workflows.value.find((w) => w.id === selectedWorkflowId.value) ?? null,
  );

  async function pickProjectDir() {
    if (!canUseTauriRuntime()) {
      toast.error(tauriRuntimeMessage);
      return;
    }
    try {
      const selected = await open({ directory: true, title: "Choose project directory" });
      if (typeof selected === "string") {
        projectDir.value = selected;
        await loadDirectory();
      }
    } catch (e) {
      toast.error(friendlyError(e));
    }
  }

  function clearProjectDir() {
    projectDir.value = "";
    dirEntries.value = [];
  }

  async function loadDirectory() {
    const dir = projectDir.value.trim();
    if (!dir) {
      dirEntries.value = [];
      return;
    }
    isLoadingDir.value = true;
    try {
      const result = await explorerReadDir(dir);
      projectDir.value = result.path;
      dirEntries.value = result.entries;
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isLoadingDir.value = false;
    }
  }

  async function openMarkdownPreview(entry: FileEntry) {
    mdPreviewName.value = entry.name;
    mdPreviewContent.value = "";
    mdPreviewOpen.value = true;
    isLoadingMdPreview.value = true;
    try {
      mdPreviewContent.value = await explorerReadTextFile(entry.path);
    } catch (e) {
      toast.error(friendlyError(e));
      mdPreviewOpen.value = false;
    } finally {
      isLoadingMdPreview.value = false;
    }
  }

  async function showInFolder(path: string) {
    try {
      await explorerOpen(path);
    } catch (e) {
      toast.error(friendlyError(e));
    }
  }

  async function loadWorkflows() {
    if (!username.value) return;
    isLoadingWorkflows.value = true;
    try {
      const results = await aiWorkflowList(username.value);
      workflows.value = results.map(toWorkflow);
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isLoadingWorkflows.value = false;
    }
  }

  /** Nạp danh sách folder skill dưới `<projectDir>/.claude/skills` (im lặng nếu chưa có folder). */
  async function loadSkillFolders() {
    const dir = projectDir.value.trim();
    if (!dir) {
      skillFolders.value = [];
      return;
    }
    try {
      const result = await explorerReadDir(`${dir}/.claude/skills`);
      skillFolders.value = result.entries.filter((e) => e.is_dir).map((e) => e.name);
    } catch {
      skillFolders.value = [];
    }
  }

  /** Mở dialog xem danh sách skill available trong `.claude/skills` của project directory. */
  async function openSkillListDialog() {
    showSkillListDialog.value = true;
    isLoadingSkillList.value = true;
    try {
      await loadSkillFolders();
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isLoadingSkillList.value = false;
    }
  }

  /** Step type "skill" match với 1 folder skill (so khớp theo skill-name, không phân biệt hoa/thường). */
  function hasMatchingSkill(step: CoworkStep): boolean {
    const skillName = step.skillName.trim().toLowerCase();
    return step.type === "skill" && skillName !== "" && skillFolderNames.value.includes(skillName);
  }

  async function applyWorkflow() {
    if (selectedWorkflowId.value === null) return;
    isApplying.value = true;
    try {
      const results = await aiWorkflowStepList(selectedWorkflowId.value);
      steps.value = results.map(toStep).sort((a, b) => a.stepOrder - b.stepOrder);
      appliedWorkflowId.value = selectedWorkflowId.value;

      await loadSkillFolders();
      const skillSteps = steps.value.filter((s) => s.type === "skill");
      showSkillWarning.value = skillSteps.length > 0 && !skillSteps.some(hasMatchingSkill);
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isApplying.value = false;
    }
  }

  /** Mở terminal tại project directory với account AI đang active để chạy step skill. */
  async function openStepTerminal(step: CoworkStep) {
    if (!hasMatchingSkill(step)) return;
    const dir = projectDir.value.trim();
    if (!dir) return;

    const active = accounts.value.find((a) => a.is_active && a.config_dir.trim());
    if (!active) {
      toast.error("Không tìm thấy account AI subscription đang active có CLAUDE_CONFIG_DIR.");
      return;
    }

    openingTerminalStepId.value = step.id;
    try {
      await aiUsageOpenTerminal(active.config_dir, dir);
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      openingTerminalStepId.value = null;
    }
  }

  async function loadAccounts() {
    isLoadingAccounts.value = true;
    try {
      accounts.value = await aiUsageListAccounts();
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isLoadingAccounts.value = false;
    }
  }

  async function setActiveAccount(id: number) {
    settingActiveId.value = id;
    try {
      await aiUsageSetActive(id);
      await loadAccounts();
      toast.success("Account switched.");
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      settingActiveId.value = null;
    }
  }

  let taskSearchTimer: ReturnType<typeof setTimeout> | null = null;

  async function doTaskSearch(query: string) {
    isSearchingTasks.value = true;
    try {
      taskSearchResults.value = await aiTaskList(query.trim() || undefined, false);
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isSearchingTasks.value = false;
    }
  }

  function triggerTaskSearch() {
    if (taskSearchTimer) clearTimeout(taskSearchTimer);
    taskSearchTimer = setTimeout(() => doTaskSearch(taskSearchQuery.value), 300);
  }

  function openTaskPicker() {
    taskSearchQuery.value = "";
    pickerSelected.value = new Map();
    showTaskPicker.value = true;
    void doTaskSearch("");
  }

  function isTaskPicked(task: AiTaskResult): boolean {
    return pickerSelected.value.has(task.id);
  }

  function toggleTaskPicked(task: AiTaskResult) {
    const map = new Map(pickerSelected.value);
    if (map.has(task.id)) map.delete(task.id);
    else map.set(task.id, task);
    pickerSelected.value = map;
  }

  async function createInlineTask(payload: TaskDialogPayload) {
    if (!payload.task_cd || !username.value) return;
    const created = await aiTaskCreate(username.value, {
      task_cd: payload.task_cd,
      task_name: payload.task_name,
      category: payload.category,
    });
    taskSearchResults.value = [created, ...taskSearchResults.value];
    const map = new Map(pickerSelected.value);
    map.set(created.id, created);
    pickerSelected.value = map;
    toast.success("Task created.");
  }

  function confirmTaskPicker() {
    const existingIds = new Set(selectedTasks.value.map((t) => t.id));
    const toAdd = Array.from(pickerSelected.value.values()).filter((t) => !existingIds.has(t.id));
    selectedTasks.value = [...selectedTasks.value, ...toAdd];
    showTaskPicker.value = false;
  }

  function removeSelectedTask(id: number) {
    selectedTasks.value = selectedTasks.value.filter((t) => t.id !== id);
    if (confirmedTaskIds.value.has(id)) {
      const set = new Set(confirmedTaskIds.value);
      set.delete(id);
      confirmedTaskIds.value = set;
    }
  }

  function isTaskConfirmed(id: number): boolean {
    return confirmedTaskIds.value.has(id);
  }

  function toggleTaskConfirmed(id: number) {
    const set = new Set(confirmedTaskIds.value);
    if (set.has(id)) set.delete(id);
    else set.add(id);
    confirmedTaskIds.value = set;
  }

  async function init() {
    await Promise.all([loadWorkflows(), loadAccounts()]);
  }

  return {
    projectDir,
    dirEntries,
    isLoadingDir,
    mdPreviewOpen,
    mdPreviewName,
    mdPreviewContent,
    isLoadingMdPreview,
    openMarkdownPreview,
    showInFolder,
    workflows,
    selectedWorkflowId,
    selectedWorkflow,
    steps,
    isLoadingWorkflows,
    isApplying,
    appliedWorkflowId,
    showSkillWarning,
    openingTerminalStepId,
    hasMatchingSkill,
    openStepTerminal,
    skillFolders,
    showSkillListDialog,
    isLoadingSkillList,
    openSkillListDialog,
    accounts,
    isLoadingAccounts,
    settingActiveId,
    selectedTasks,
    showTaskPicker,
    taskSearchQuery,
    taskSearchResults,
    isSearchingTasks,
    pickerSelectedCount,
    openTaskPicker,
    triggerTaskSearch,
    isTaskPicked,
    toggleTaskPicked,
    createInlineTask,
    confirmTaskPicker,
    removeSelectedTask,
    isTaskConfirmed,
    toggleTaskConfirmed,
    pickProjectDir,
    clearProjectDir,
    loadDirectory,
    loadWorkflows,
    applyWorkflow,
    loadAccounts,
    setActiveAccount,
    init,
  };
}
