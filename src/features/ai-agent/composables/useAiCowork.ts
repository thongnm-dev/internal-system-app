import { computed, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useAuthStore } from "@/app/stores/auth";
import { useToast } from "@/shared/composables/useToast";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { aiCoworkGetState, aiCoworkSaveState } from "@/tauri/commands/ai-cowork";
import { aiWorkflowList, aiWorkflowStepList } from "@/tauri/commands/ai-workflow";
import type { AiWorkflowResult, AiWorkflowStepResult } from "@/tauri/commands/ai-workflow";
import { aiUsageListAccounts, aiUsageOpenTerminal, aiUsageSetActive } from "@/tauri/commands/ai-usage";
import type { AiAccount } from "@/_/types/ai-usage";
import { explorerOpen, explorerReadDir, explorerReadTextFile } from "@/tauri/commands/explorer";
import type { FileEntry } from "@/tauri/commands/explorer";
import type { WorkflowStepType } from "@/_/types/ai-workflow";
import {
  aiTaskCreate,
  aiTaskList,
  aiTaskWfProcCreate,
  aiTaskWfProcList,
  aiTaskWfProcUpdate,
  aiTaskWfProcStepCreate,
  aiTaskWfProcStepList,
  aiTaskWfProcStepUpdate,
} from "@/tauri/commands/ai-task";
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
  isLatestStep: boolean;
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
    isLatestStep: r.is_latest_step,
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

  // Dialog báo lỗi khi các task đang chọn không cùng 1 workflow_proc_step.
  const showTaskStepConflict = ref(false);
  const taskStepConflictGroups = ref<{ stepLabel: string; taskCds: string[] }[]>([]);

  // Column 2 — AI accounts.
  const accounts = ref<AiAccount[]>([]);
  const isLoadingAccounts = ref(false);
  const settingActiveId = ref<number | null>(null);

  // Column "Tasks" — danh sách task đã chọn cho lần chạy này.
  const selectedTasks = ref<AiTaskResult[]>([]);
  // Task đã tick checkbox xác nhận lại (con trong danh sách selectedTasks).
  const confirmedTaskIds = ref<Set<number>>(new Set());

  // Bước workflow_proc_step hiện tại (chung) của các task đã xác nhận — nạp lại mỗi khi
  // danh sách task xác nhận / workflow áp dụng thay đổi, dùng để enable/disable nút mở terminal.
  const confirmedStepId = ref<number | null>(null);
  const confirmedStepConflict = ref(false);
  const confirmedStepPairs = ref<{ taskCd: string; stepId: number | null }[]>([]);
  let resolveStepToken = 0;

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

  /** Các task đã tick checkbox xác nhận. */
  const confirmedTasks = computed(() =>
    selectedTasks.value.filter((t) => confirmedTaskIds.value.has(t.id)),
  );

  /** CoworkStep tương ứng bước hiện tại chung của các task đã xác nhận (null nếu chưa/không xác định). */
  const confirmedStep = computed(() =>
    confirmedStepId.value === null
      ? null
      : steps.value.find((s) => s.id === confirmedStepId.value) ?? null,
  );
  /** Các task đã xác nhận đang ở bước cuối cùng của workflow. */
  const confirmedStepIsLatest = computed(() => confirmedStep.value?.isLatestStep === true);
  /** Các task đã xác nhận đang ở một bước xác định nhưng KHÔNG phải loại "skill". */
  const confirmedStepIsNonSkill = computed(
    () => confirmedStep.value !== null && confirmedStep.value.type !== "skill",
  );

  /**
   * Nạp lại bước workflow_proc_step hiện tại của các task đã xác nhận (theo workflow đang áp dụng).
   * Dùng token để tránh race khi có nhiều lần gọi chồng nhau.
   */
  async function resolveConfirmedTaskSteps() {
    const token = ++resolveStepToken;
    const tasks = confirmedTasks.value;
    if (tasks.length === 0) {
      confirmedStepPairs.value = [];
      confirmedStepConflict.value = false;
      confirmedStepId.value = null;
      return;
    }
    const wfId = appliedWorkflowId.value;
    // Task chưa có tiến trình workflow → coi như đang ở bước đầu tiên của workflow (để có thể bắt đầu).
    const firstStepId = steps.value[0]?.id ?? null;
    try {
      const pairs = await Promise.all(
        tasks.map(async (t) => {
          const procs = await aiTaskWfProcList(t.id);
          const proc = wfId !== null ? procs.find((p) => p.wf_id === wfId) : procs[0];
          return { taskCd: t.task_cd, stepId: proc?.latest_step_id ?? firstStepId };
        }),
      );
      if (token !== resolveStepToken) return; // đã có lần resolve mới hơn
      confirmedStepPairs.value = pairs;
      const distinct = new Set(pairs.map((p) => (p.stepId === null ? "none" : String(p.stepId))));
      confirmedStepConflict.value = distinct.size > 1;
      confirmedStepId.value = confirmedStepConflict.value ? null : pairs[0].stepId;
    } catch {
      if (token !== resolveStepToken) return;
      confirmedStepPairs.value = [];
      confirmedStepConflict.value = false;
      confirmedStepId.value = null;
    }
  }

  watch(
    [confirmedTaskIds, selectedTasks, appliedWorkflowId],
    () => {
      void resolveConfirmedTaskSteps();
    },
    { deep: true },
  );

  /** Lưu lại project directory hiện tại để lần mở màn hình sau tự động load lại. */
  function persistState() {
    void aiCoworkSaveState(projectDir.value).catch(() => undefined);
  }

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
        persistState();
      }
    } catch (e) {
      toast.error(friendlyError(e));
    }
  }

  function clearProjectDir() {
    projectDir.value = "";
    dirEntries.value = [];
    persistState();
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

  /** Nhãn hiển thị cho 1 workflow step id (dựa trên step của workflow đang áp dụng). */
  function stepLabelForId(stepId: number | null): string {
    if (stepId === null) return "Chưa có bước nào";
    return steps.value.find((s) => s.id === stepId)?.name ?? `Bước #${stepId}`;
  }

  /** Step này có phải bước hiện tại chung của các task đã xác nhận hay không. */
  function isCurrentTaskStep(step: CoworkStep): boolean {
    return (
      confirmedTasks.value.length > 0 &&
      !confirmedStepConflict.value &&
      confirmedStepId.value !== null &&
      step.id === confirmedStepId.value
    );
  }

  /**
   * Nút mở terminal của 1 step skill có được enable hay không, dựa trên bước hiện tại
   * của các task đã xác nhận:
   *  - Phải có skill folder khớp trong `.claude/skills` (điều kiện sẵn có).
   *  - Không có task xác nhận → cho mở tự do (giữ hành vi cũ).
   *  - Khi các task xác nhận lệch bước, vẫn để enable để lần bấm hiển thị dialog xung đột.
   *  - Nếu task đã ở bước cuối cùng (latest step) → không cho chạy terminal.
   *  - Chỉ enable đúng skill của bước hiện tại của task (skill-name tương ứng bước hiện tại).
   */
  function canOpenStepTerminal(step: CoworkStep): boolean {
    if (!hasMatchingSkill(step)) return false;
    if (confirmedTasks.value.length === 0) return true;
    if (confirmedStepConflict.value) return true;
    if (confirmedStepIsLatest.value) return false;
    return confirmedStepId.value !== null && step.id === confirmedStepId.value;
  }

  /** Tooltip cho nút mở terminal — giải thích lý do enable/disable. */
  function stepTerminalTitle(step: CoworkStep): string {
    if (!hasMatchingSkill(step)) return "Không tìm thấy skill khớp trong .claude/skills";
    if (confirmedTasks.value.length === 0 || confirmedStepConflict.value) return "Mở terminal cho skill này";
    if (confirmedStepIsLatest.value) return "Task đã ở bước cuối cùng của workflow — không thể mở terminal";
    if (confirmedStepId.value === null) return "Task chưa có bước hiện tại trong workflow";
    if (step.id !== confirmedStepId.value) return "Chỉ mở terminal được ở skill của bước hiện tại của task";
    return "Mở terminal cho skill của bước hiện tại";
  }

  /** Bật dialog lỗi liệt kê các task đã xác nhận đang lệch bước workflow_proc_step. */
  function showConfirmedStepConflictDialog() {
    const groups = new Map<number | null, string[]>();
    confirmedStepPairs.value.forEach((p) => {
      const list = groups.get(p.stepId) ?? [];
      list.push(p.taskCd);
      groups.set(p.stepId, list);
    });
    taskStepConflictGroups.value = Array.from(groups.entries()).map(([stepId, taskCds]) => ({
      stepLabel: stepLabelForId(stepId),
      taskCds,
    }));
    showTaskStepConflict.value = true;
  }

  /**
   * Đăng ký dữ liệu tiến trình workflow cho các task đã xác nhận khi bắt đầu chạy 1 step:
   *  - Tạo `ai_task_wf_proc` cho (task, workflow đang áp dụng) nếu chưa có.
   *  - Cập nhật `latest_step_id` = step đang chạy.
   *  - Tạo `ai_task_wf_proc_step` cho (proc, step) với status `in_progress` (hoặc cập nhật nếu đã có).
   */
  async function registerTaskWorkflowProgress(step: CoworkStep) {
    const wfId = appliedWorkflowId.value;
    if (wfId === null) return;
    const user = username.value;
    for (const task of confirmedTasks.value) {
      const procs = await aiTaskWfProcList(task.id);
      let proc = procs.find((p) => p.wf_id === wfId);
      if (!proc) {
        proc = await aiTaskWfProcCreate(user, { task_id: task.id, wf_id: wfId });
      }
      if (proc.latest_step_id !== step.id) {
        await aiTaskWfProcUpdate(proc.id, step.id, user);
      }
      const procSteps = await aiTaskWfProcStepList(proc.id);
      const existing = procSteps.find((s) => s.wf_step_id === step.id);
      if (!existing) {
        await aiTaskWfProcStepCreate(user, {
          wf_proc_id: proc.id,
          wf_step_id: step.id,
          status: "in_progress",
        });
      } else if (existing.status !== "in_progress") {
        await aiTaskWfProcStepUpdate(existing.id, "in_progress", user);
      }
    }
  }

  /**
   * Ghép prompt truyền cho `claude` cho MỘT task: `/<skill-name> [CATEGORY] <task_cd>`.
   * VD: `/create-plan [SCREEN] SZTN_G_SC01`.
   */
  function buildTaskSkillPrompt(step: CoworkStep, task: AiTaskResult): string {
    return `/${step.skillName} [${task.category.toUpperCase()}] ${task.task_cd}`;
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
      // Nạp lại trạng thái bước hiện tại của các task đã xác nhận rồi mới quyết định.
      await resolveConfirmedTaskSteps();

      if (confirmedTasks.value.length > 0) {
        // Các task đã xác nhận phải cùng 1 workflow_proc_step.
        if (confirmedStepConflict.value) {
          showConfirmedStepConflictDialog();
          return;
        }
        // Task đã ở bước cuối cùng → không cho chạy terminal.
        if (confirmedStepIsLatest.value) {
          toast.error("Task đã ở bước cuối cùng của workflow — không thể mở terminal.");
          return;
        }
        // Task chưa có bước hiện tại (chưa bắt đầu workflow).
        if (confirmedStepId.value === null) {
          toast.error("Task chưa có bước hiện tại trong workflow — không thể mở terminal.");
          return;
        }
        // Chỉ mở được terminal ở skill của đúng bước hiện tại của task.
        if (step.id !== confirmedStepId.value) {
          toast.error("Chỉ mở terminal được ở skill của bước hiện tại của task.");
          return;
        }
      }

      // Đăng ký dữ liệu wf_proc / wf_proc_step rồi mở terminal kèm prompt skill.
      // `claude --dangerously-skip-permissions` (trong build_claude_command) đã tự bỏ qua
      // hộp thoại "trust folder" nên prompt được chạy thẳng.
      const tasks = confirmedTasks.value;
      if (tasks.length > 0) {
        await registerTaskWorkflowProgress(step);
        // Đồng bộ lại trạng thái bước hiện tại theo dữ liệu vừa ghi.
        await resolveConfirmedTaskSteps();
        // Mỗi task mở 1 terminal riêng (không gom chung) để tránh xung đột khi cùng xử lý 1 file.
        for (const task of tasks) {
          await aiUsageOpenTerminal(active.config_dir, dir, buildTaskSkillPrompt(step, task));
        }
      } else {
        await aiUsageOpenTerminal(active.config_dir, dir);
      }
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
    if (!canUseTauriRuntime()) return;
    try {
      const state = await aiCoworkGetState();
      if (state.project_dir) {
        projectDir.value = state.project_dir;
        await loadDirectory();
      }
    } catch {
      // Không có lịch sử hoặc lỗi đọc file — bỏ qua, giữ màn hình ở trạng thái mặc định.
    }
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
    canOpenStepTerminal,
    stepTerminalTitle,
    isCurrentTaskStep,
    confirmedStepConflict,
    confirmedStepIsLatest,
    confirmedStepIsNonSkill,
    openStepTerminal,
    skillFolders,
    showSkillListDialog,
    isLoadingSkillList,
    openSkillListDialog,
    showTaskStepConflict,
    taskStepConflictGroups,
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
