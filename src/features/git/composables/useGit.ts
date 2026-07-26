import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";

import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { explorerOpen } from "@/tauri/commands/explorer";
import {
  gitAddRepo,
  gitBranches,
  gitCheckoutBranch,
  gitCherryPick,
  gitCherryPickAbort,
  gitCherryPickContinue,
  gitCleanupDelete,
  gitCleanupScan,
  gitClone,
  gitCommitNoEdit,
  gitCompare,
  gitCompareFileDiff,
  gitCreatePullRequest,
  gitListConflicts,
  gitListPullRequests,
  gitMerge,
  gitMergeAbort,
  gitOpenTerminal,
  gitOpenUrl,
  gitResolveConflict,
  gitTagCreate,
  gitTagDelete,
  gitTagList,
  gitCommit,
  gitCommitDetail,
  gitCommitFileDiff,
  gitCreateBranch,
  gitDeleteBranch,
  gitDiscard,
  gitFetch,
  gitFileDiff,
  gitListRepos,
  gitLog,
  gitPull,
  gitPush,
  gitRebase,
  gitRebaseAbort,
  gitRebaseContinue,
  gitRemoveRepo,
  gitRepoInfo,
  gitReset,
  gitRevert,
  gitRevertAbort,
  gitStage,
  gitStashApply,
  gitStashDrop,
  gitStashList,
  gitStashSave,
  gitStatus,
  gitTouchRepo,
  gitUndoLastCommit,
  gitUnstage,
  gitWorktreeAdd,
  gitWorktreeList,
  gitWorktreeRemove,
} from "@/tauri/commands/git";
import type {
  GitBranch,
  GitCommit,
  GitCommitDetail,
  GitComparison,
  GitDiff,
  GitFileChange,
  GitProgress,
  GitPullRequest,
  GitRepo,
  GitRepoInfo,
  GitStash,
  GitTag,
  GitWorktree,
} from "@/_/types/git";
import { useToast } from "@/shared/composables/useToast";

const HISTORY_LIMIT = 80;
const ACTIVE_REPO_KEY = "git.activeRepoId";

/** File đang được chọn để xem diff. */
type SelectedFile = {
  path: string;
  staged: boolean;
  untracked: boolean;
};

export type GitTab = "changes" | "history";

export function useGit() {
  const toast = useToast();

  const repos = ref<GitRepo[]>([]);
  const activeRepo = ref<GitRepo | null>(null);
  const info = ref<GitRepoInfo | null>(null);

  const staged = ref<GitFileChange[]>([]);
  const unstaged = ref<GitFileChange[]>([]);
  const branches = ref<GitBranch[]>([]);
  const stashes = ref<GitStash[]>([]);
  const worktrees = ref<GitWorktree[]>([]);
  const tags = ref<GitTag[]>([]);
  const commits = ref<GitCommit[]>([]);

  const comparison = ref<GitComparison | null>(null);
  const comparisonDiff = ref<GitDiff | null>(null);

  const pullRequests = ref<GitPullRequest[]>([]);
  const pullRequestsLoading = ref(false);

  const conflicts = ref<string[]>([]);

  // Commit browser (dialog duyệt commit + copy SHA).
  const browserCommits = ref<GitCommit[]>([]);
  const browserFiles = ref<GitFileChange[]>([]);
  const browserDiff = ref<GitDiff | null>(null);
  const browserLoading = ref(false);

  const selectedFile = ref<SelectedFile | null>(null);
  const diff = ref<GitDiff | null>(null);
  const diffLoading = ref(false);

  const selectedCommit = ref<GitCommit | null>(null);
  const commitDetail = ref<GitCommitDetail | null>(null);
  const commitFileDiff = ref<GitDiff | null>(null);

  const commitMessage = ref("");
  const tab = ref<GitTab>("changes");

  // Cờ trạng thái — dùng cho spinner cục bộ, không blank cả màn hình.
  const loadingRepo = ref(false);
  const refreshing = ref(false);
  const committing = ref(false);
  const syncing = ref(false);
  const busyMessage = ref("");
  const syncProgress = ref<GitProgress | null>(null);

  const runtimeAvailable = computed(() => canUseTauriRuntime());
  const hasChanges = computed(() => staged.value.length + unstaged.value.length > 0);
  const canCommit = computed(
    () => staged.value.length > 0 && commitMessage.value.trim().length > 0 && !committing.value,
  );

  const localBranches = computed(() => branches.value.filter((b) => !b.is_remote));
  const remoteBranches = computed(() => branches.value.filter((b) => b.is_remote));

  function reportError(prefix: string, e: unknown) {
    toast.error(`${prefix}: ${friendlyError(e)}`);
  }

  // === Danh sách repo ===

  async function loadRepos() {
    if (!runtimeAvailable.value) return;
    try {
      repos.value = await gitListRepos();
      const savedId = Number(localStorage.getItem(ACTIVE_REPO_KEY) ?? "");
      const target =
        repos.value.find((r) => r.id === savedId) ?? repos.value[0] ?? null;
      if (target) await openRepo(target);
    } catch (e) {
      reportError("Không tải được danh sách repo", e);
    }
  }

  async function addRepoFromDialog() {
    if (!runtimeAvailable.value) return;
    try {
      const picked = await open({ directory: true, title: "Chọn thư mục Git repository" });
      if (!picked || typeof picked !== "string") return;
      const repo = await gitAddRepo(picked);
      if (!repos.value.some((r) => r.id === repo.id)) {
        repos.value = [repo, ...repos.value];
      }
      await openRepo(repo);
      toast.success(`Đã thêm repo "${repo.name}".`);
    } catch (e) {
      reportError("Không thêm được repo", e);
    }
  }

  async function removeRepo(repo: GitRepo) {
    try {
      await gitRemoveRepo(repo.id);
      repos.value = repos.value.filter((r) => r.id !== repo.id);
      if (activeRepo.value?.id === repo.id) {
        activeRepo.value = null;
        resetRepoState();
        const next = repos.value[0];
        if (next) await openRepo(next);
      }
      toast.success(`Đã gỡ repo "${repo.name}" khỏi danh sách.`);
    } catch (e) {
      reportError("Không gỡ được repo", e);
    }
  }

  function resetRepoState() {
    info.value = null;
    staged.value = [];
    unstaged.value = [];
    branches.value = [];
    stashes.value = [];
    commits.value = [];
    selectedFile.value = null;
    diff.value = null;
    selectedCommit.value = null;
    commitDetail.value = null;
    commitFileDiff.value = null;
  }

  async function openRepo(repo: GitRepo) {
    activeRepo.value = repo;
    localStorage.setItem(ACTIVE_REPO_KEY, String(repo.id));
    loadingRepo.value = true;
    resetRepoState();
    try {
      await Promise.all([refreshStatusAndInfo(), refreshBranches(), refreshStashes()]);
      gitTouchRepo(repo.id).catch(() => {});
    } catch (e) {
      reportError("Không mở được repo", e);
    } finally {
      loadingRepo.value = false;
    }
  }

  const repoPath = () => activeRepo.value?.path ?? "";

  // === Refresh (giữ dữ liệu cũ trong lúc tải để tránh nháy màn hình) ===

  async function refreshStatusAndInfo() {
    const path = repoPath();
    if (!path) return;
    refreshing.value = true;
    try {
      const [st, nfo] = await Promise.all([gitStatus(path), gitRepoInfo(path)]);
      staged.value = st.staged;
      unstaged.value = st.unstaged;
      info.value = nfo;
      reconcileSelectedFile();
    } catch (e) {
      reportError("Không lấy được trạng thái", e);
    } finally {
      refreshing.value = false;
    }
  }

  async function refreshBranches() {
    const path = repoPath();
    if (!path) return;
    try {
      branches.value = await gitBranches(path);
    } catch (e) {
      reportError("Không lấy được branch", e);
    }
  }

  async function refreshStashes() {
    const path = repoPath();
    if (!path) return;
    try {
      stashes.value = await gitStashList(path);
    } catch (e) {
      reportError("Không lấy được stash", e);
    }
  }

  /** Nếu file đang chọn không còn trong danh sách → bỏ chọn; ngược lại nạp lại diff. */
  function reconcileSelectedFile() {
    const sel = selectedFile.value;
    if (!sel) return;
    const list = sel.staged ? staged.value : unstaged.value;
    const found = list.find((f) => f.path === sel.path);
    if (!found) {
      selectedFile.value = null;
      diff.value = null;
    } else {
      void loadDiff(found, sel.staged);
    }
  }

  // === Diff (Changes tab) ===

  async function selectFile(file: GitFileChange, isStaged: boolean) {
    selectedFile.value = { path: file.path, staged: isStaged, untracked: file.untracked };
    await loadDiff(file, isStaged);
  }

  async function loadDiff(file: GitFileChange, isStaged: boolean) {
    const path = repoPath();
    if (!path) return;
    diffLoading.value = true;
    try {
      diff.value = await gitFileDiff(path, file.path, isStaged, file.untracked && !isStaged);
    } catch (e) {
      reportError("Không đọc được diff", e);
      diff.value = null;
    } finally {
      diffLoading.value = false;
    }
  }

  // === Staging ===

  async function stageFiles(files: string[]) {
    await mutate(() => gitStage(repoPath(), files), "Không stage được file");
  }
  async function unstageFiles(files: string[]) {
    await mutate(() => gitUnstage(repoPath(), files), "Không unstage được file");
  }
  async function stageAll() {
    await mutate(() => gitStage(repoPath(), []), "Không stage được");
  }
  async function unstageAll() {
    await mutate(() => gitUnstage(repoPath(), []), "Không unstage được");
  }
  async function discardFiles(files: string[]) {
    await mutate(() => gitDiscard(repoPath(), files), "Không bỏ được thay đổi");
  }

  /** Chạy một mutation rồi refresh status — giữ UI mượt (không blank). */
  async function mutate(fn: () => Promise<unknown>, errPrefix: string) {
    if (!repoPath()) return;
    try {
      await fn();
      await refreshStatusAndInfo();
    } catch (e) {
      reportError(errPrefix, e);
    }
  }

  // === Commit ===

  async function commit() {
    const path = repoPath();
    if (!path || !canCommit.value) return;
    committing.value = true;
    try {
      await gitCommit(path, commitMessage.value.trim());
      commitMessage.value = "";
      selectedFile.value = null;
      diff.value = null;
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      if (tab.value === "history") await loadHistory();
      toast.success("Đã commit.");
    } catch (e) {
      reportError("Commit thất bại", e);
    } finally {
      committing.value = false;
    }
  }

  // === Sync: fetch / pull / push ===

  async function fetch() {
    await runSync(
      (onP) => gitFetch(repoPath(), onP),
      "Đang fetch…",
      "Fetch xong.",
      "Fetch thất bại",
    );
  }
  async function pull() {
    await runSync((onP) => gitPull(repoPath(), onP), "Đang pull…", "Pull xong.", "Pull thất bại");
  }
  async function push() {
    await runSync((onP) => gitPush(repoPath(), onP), "Đang push…", "Push xong.", "Push thất bại");
  }

  async function runSync(
    fn: (onProgress: (p: GitProgress) => void) => Promise<string>,
    busy: string,
    ok: string,
    errPrefix: string,
  ) {
    const path = repoPath();
    if (!path || syncing.value) return;
    syncing.value = true;
    busyMessage.value = busy;
    syncProgress.value = null;
    try {
      await fn((p) => {
        syncProgress.value = p;
      });
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      if (tab.value === "history") await loadHistory();
      toast.success(ok);
    } catch (e) {
      reportError(errPrefix, e);
    } finally {
      syncing.value = false;
      busyMessage.value = "";
      syncProgress.value = null;
    }
  }

  // === History tab ===

  async function loadHistory() {
    const path = repoPath();
    if (!path) return;
    try {
      commits.value = await gitLog(path, HISTORY_LIMIT);
      if (commits.value.length && !selectedCommit.value) {
        await selectCommit(commits.value[0]);
      }
    } catch (e) {
      reportError("Không tải được lịch sử", e);
    }
  }

  /** Refresh sau khi lịch sử/HEAD thay đổi (undo, reset, checkout commit…). */
  async function refreshAfterHistoryChange() {
    selectedCommit.value = null;
    commits.value = [];
    selectedFile.value = null;
    diff.value = null;
    await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
    if (tab.value === "history") await loadHistory();
  }

  /** Undo commit gần nhất (giữ thay đổi ở staged). */
  async function undoLastCommit() {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang undo commit…";
    try {
      await gitUndoLastCommit(path);
      await refreshAfterHistoryChange();
      toast.success("Đã undo commit gần nhất (thay đổi giữ ở staged).");
    } catch (e) {
      reportError("Không undo được commit", e);
    } finally {
      busyMessage.value = "";
    }
  }

  /** Reset branch hiện tại về một commit. */
  async function resetTo(hash: string, mode: "soft" | "mixed" | "hard") {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang reset…";
    try {
      await gitReset(path, hash, mode);
      await refreshAfterHistoryChange();
      toast.success(`Đã reset (${mode}) về commit.`);
    } catch (e) {
      reportError("Reset thất bại", e);
    } finally {
      busyMessage.value = "";
    }
  }

  /** Checkout một commit (detached HEAD). */
  async function checkoutCommit(hash: string) {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang checkout commit…";
    try {
      await gitCheckoutBranch(path, hash);
      await refreshAfterHistoryChange();
      toast.success("Đã checkout commit (detached HEAD).");
    } catch (e) {
      reportError("Không checkout được commit", e);
    } finally {
      busyMessage.value = "";
    }
  }

  /** Tạo branch mới tại một commit cụ thể rồi checkout sang đó. */
  async function createBranchAt(name: string, from: string) {
    const path = repoPath();
    if (!path || !name.trim()) return;
    try {
      await gitCreateBranch(path, name.trim(), from);
      await refreshAfterHistoryChange();
      toast.success(`Đã tạo branch "${name.trim()}" tại commit.`);
    } catch (e) {
      reportError("Không tạo được branch", e);
    }
  }

  /** Copy một đoạn text vào clipboard. */
  async function copyText(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(`Đã copy ${label}.`);
    } catch (e) {
      reportError("Không copy được", e);
    }
  }

  async function selectCommit(c: GitCommit) {
    const path = repoPath();
    if (!path) return;
    selectedCommit.value = c;
    commitFileDiff.value = null;
    try {
      commitDetail.value = await gitCommitDetail(path, c.hash);
    } catch (e) {
      reportError("Không đọc được commit", e);
    }
  }

  async function selectCommitFile(file: GitFileChange) {
    const path = repoPath();
    const c = selectedCommit.value;
    if (!path || !c) return;
    diffLoading.value = true;
    try {
      commitFileDiff.value = await gitCommitFileDiff(path, c.hash, file.path);
    } catch (e) {
      reportError("Không đọc được diff", e);
    } finally {
      diffLoading.value = false;
    }
  }

  function switchTab(next: GitTab) {
    tab.value = next;
    if (next === "history" && commits.value.length === 0) void loadHistory();
  }

  // === Branch ===

  async function checkoutBranch(name: string) {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = `Đang chuyển sang ${name}…`;
    try {
      await gitCheckoutBranch(path, name);
      selectedFile.value = null;
      diff.value = null;
      selectedCommit.value = null;
      commits.value = [];
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      if (tab.value === "history") await loadHistory();
      toast.success(`Đã chuyển sang branch "${name}".`);
    } catch (e) {
      reportError("Không đổi được branch", e);
    } finally {
      busyMessage.value = "";
    }
  }

  async function createBranch(name: string) {
    const path = repoPath();
    if (!path || !name.trim()) return;
    try {
      await gitCreateBranch(path, name.trim());
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      toast.success(`Đã tạo và chuyển sang branch "${name.trim()}".`);
    } catch (e) {
      reportError("Không tạo được branch", e);
    }
  }

  async function deleteBranch(name: string, force: boolean) {
    const path = repoPath();
    if (!path) return;
    try {
      await gitDeleteBranch(path, name, force);
      await refreshBranches();
      toast.success(`Đã xóa branch "${name}".`);
    } catch (e) {
      reportError("Không xóa được branch", e);
    }
  }

  // === Stash ===

  async function stashSave(message: string) {
    const path = repoPath();
    if (!path) return;
    try {
      await gitStashSave(path, message);
      await Promise.all([refreshStatusAndInfo(), refreshStashes()]);
      toast.success("Đã cất thay đổi vào stash.");
    } catch (e) {
      reportError("Không stash được", e);
    }
  }

  async function stashApply(reference: string, pop: boolean) {
    const path = repoPath();
    if (!path) return;
    try {
      await gitStashApply(path, reference, pop);
      await Promise.all([refreshStatusAndInfo(), refreshStashes()]);
      toast.success(pop ? "Đã áp dụng và xóa stash." : "Đã áp dụng stash.");
    } catch (e) {
      reportError("Không áp dụng được stash", e);
    }
  }

  async function stashDrop(reference: string) {
    const path = repoPath();
    if (!path) return;
    try {
      await gitStashDrop(path, reference);
      await refreshStashes();
      toast.success("Đã xóa stash.");
    } catch (e) {
      reportError("Không xóa được stash", e);
    }
  }

  // === Revert ===

  async function revert(hash: string) {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang revert…";
    try {
      await gitRevert(path, hash);
      await Promise.all([refreshStatusAndInfo(), refreshBranches(), loadHistory()]);
      toast.success("Đã revert commit.");
    } catch (e) {
      reportError("Revert thất bại", e);
    } finally {
      busyMessage.value = "";
    }
  }

  async function revertAbort() {
    const path = repoPath();
    if (!path) return;
    try {
      await gitRevertAbort(path);
      await refreshStatusAndInfo();
      toast.success("Đã hủy revert.");
    } catch (e) {
      reportError("Không hủy được revert", e);
    }
  }

  // === Rebase ===

  async function rebaseOnto(onto: string) {
    const path = repoPath();
    if (!path || !onto.trim()) return;
    busyMessage.value = `Đang rebase lên ${onto}…`;
    try {
      await gitRebase(path, onto);
      selectedCommit.value = null;
      commits.value = [];
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      if (tab.value === "history") await loadHistory();
      toast.success(`Đã rebase lên "${onto}".`);
    } catch (e) {
      reportError("Rebase gặp lỗi (có thể do xung đột)", e);
      await refreshStatusAndInfo();
    } finally {
      busyMessage.value = "";
    }
  }

  async function rebaseAbort() {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang hủy rebase…";
    try {
      await gitRebaseAbort(path);
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      toast.success("Đã hủy rebase.");
    } catch (e) {
      reportError("Không hủy được rebase", e);
    } finally {
      busyMessage.value = "";
    }
  }

  async function rebaseContinue() {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang tiếp tục rebase…";
    try {
      await gitRebaseContinue(path);
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      if (tab.value === "history") await loadHistory();
      toast.success("Đã tiếp tục rebase.");
    } catch (e) {
      reportError("Không tiếp tục được rebase (còn xung đột chưa xử lý?)", e);
      await refreshStatusAndInfo();
    } finally {
      busyMessage.value = "";
    }
  }

  // === Cherry-pick ===

  async function cherryPick(hash: string) {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang cherry-pick…";
    try {
      await gitCherryPick(path, hash);
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      if (tab.value === "history") await loadHistory();
      toast.success("Đã cherry-pick commit.");
    } catch (e) {
      reportError("Cherry-pick gặp lỗi (có thể do xung đột)", e);
      await refreshStatusAndInfo();
    } finally {
      busyMessage.value = "";
    }
  }

  async function cherryPickAbort() {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang hủy cherry-pick…";
    try {
      await gitCherryPickAbort(path);
      await refreshStatusAndInfo();
      toast.success("Đã hủy cherry-pick.");
    } catch (e) {
      reportError("Không hủy được cherry-pick", e);
    } finally {
      busyMessage.value = "";
    }
  }

  async function cherryPickContinue() {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang tiếp tục cherry-pick…";
    try {
      await gitCherryPickContinue(path);
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      if (tab.value === "history") await loadHistory();
      toast.success("Đã tiếp tục cherry-pick.");
    } catch (e) {
      reportError("Không tiếp tục được cherry-pick (còn xung đột?)", e);
      await refreshStatusAndInfo();
    } finally {
      busyMessage.value = "";
    }
  }

  // === Worktree ===

  async function loadWorktrees() {
    const path = repoPath();
    if (!path) return;
    try {
      worktrees.value = await gitWorktreeList(path);
    } catch (e) {
      reportError("Không lấy được danh sách worktree", e);
    }
  }

  /** Tạo worktree. Trả về đường dẫn đã tạo (rỗng nếu thất bại). */
  async function worktreeAdd(
    worktreePath: string,
    branch: string,
    newBranch: string,
  ): Promise<string> {
    const path = repoPath();
    if (!path || !worktreePath.trim()) return "";
    busyMessage.value = "Đang tạo worktree…";
    try {
      const created = await gitWorktreeAdd(path, worktreePath, branch, newBranch);
      await Promise.all([loadWorktrees(), refreshBranches()]);
      toast.success("Đã tạo worktree.");
      return created;
    } catch (e) {
      reportError("Không tạo được worktree", e);
      return "";
    } finally {
      busyMessage.value = "";
    }
  }

  async function worktreeRemove(worktreePath: string, force: boolean) {
    const path = repoPath();
    if (!path) return;
    try {
      await gitWorktreeRemove(path, worktreePath, force);
      await loadWorktrees();
      toast.success("Đã gỡ worktree.");
    } catch (e) {
      reportError("Không gỡ được worktree", e);
    }
  }

  /** Thêm một đường dẫn (vd. worktree vừa tạo) vào danh sách repo và mở nó. */
  async function openPathAsRepo(targetPath: string) {
    if (!targetPath.trim()) return;
    try {
      const repo = await gitAddRepo(targetPath);
      if (!repos.value.some((r) => r.id === repo.id)) {
        repos.value = [repo, ...repos.value];
      }
      await openRepo(repo);
    } catch (e) {
      reportError("Không mở được thư mục", e);
    }
  }

  // === Tag ===

  async function loadTags() {
    const path = repoPath();
    if (!path) return;
    try {
      tags.value = await gitTagList(path);
    } catch (e) {
      reportError("Không lấy được tag", e);
    }
  }

  async function createTag(
    name: string,
    hash: string,
    message: string,
    annotated: boolean,
    push: boolean,
  ): Promise<boolean> {
    const path = repoPath();
    if (!path || !name.trim()) return false;
    busyMessage.value = "Đang tạo tag…";
    try {
      await gitTagCreate(path, name.trim(), hash, message, annotated, push);
      await loadTags();
      toast.success(`Đã tạo tag "${name.trim()}"${push ? " và push lên origin" : ""}.`);
      return true;
    } catch (e) {
      reportError("Không tạo được tag", e);
      return false;
    } finally {
      busyMessage.value = "";
    }
  }

  async function deleteTag(name: string, remote: boolean) {
    const path = repoPath();
    if (!path) return;
    try {
      await gitTagDelete(path, name, remote);
      await loadTags();
      toast.success(`Đã xóa tag "${name}".`);
    } catch (e) {
      reportError("Không xóa được tag", e);
    }
  }

  // === Merge ===

  async function mergeBranch(branch: string, squash: boolean, message: string): Promise<boolean> {
    const path = repoPath();
    if (!path || !branch.trim()) return false;
    busyMessage.value = squash ? "Đang squash & merge…" : "Đang merge…";
    try {
      await gitMerge(path, branch, squash, message);
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      if (tab.value === "history") await loadHistory();
      toast.success(squash ? `Đã squash & merge "${branch}".` : `Đã merge "${branch}".`);
      return true;
    } catch (e) {
      reportError("Merge gặp lỗi (có thể do xung đột)", e);
      await refreshStatusAndInfo();
      return false;
    } finally {
      busyMessage.value = "";
    }
  }

  async function mergeAbort() {
    const path = repoPath();
    if (!path) return;
    busyMessage.value = "Đang hủy merge…";
    try {
      await gitMergeAbort(path);
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      toast.success("Đã hủy merge.");
    } catch (e) {
      reportError("Không hủy được merge", e);
    } finally {
      busyMessage.value = "";
    }
  }

  // === Resolve conflict ===

  async function loadConflicts() {
    const path = repoPath();
    if (!path) return;
    try {
      conflicts.value = await gitListConflicts(path);
    } catch (e) {
      reportError("Không lấy được danh sách xung đột", e);
    }
  }

  async function resolveConflict(file: string, side: "ours" | "theirs") {
    const path = repoPath();
    if (!path) return;
    try {
      await gitResolveConflict(path, file, side);
      await Promise.all([loadConflicts(), refreshStatusAndInfo()]);
    } catch (e) {
      reportError("Không giải quyết được xung đột", e);
    }
  }

  /** Đánh dấu file đã tự xử lý (stage nó). */
  async function markResolved(file: string) {
    const path = repoPath();
    if (!path) return;
    try {
      await gitStage(path, [file]);
      await Promise.all([loadConflicts(), refreshStatusAndInfo()]);
    } catch (e) {
      reportError("Không stage được file", e);
    }
  }

  /** Hoàn tất sau khi hết xung đột: tùy trạng thái mà continue/commit. */
  async function finishConflict() {
    const path = repoPath();
    if (!path) return;
    if (info.value?.rebase_in_progress) return rebaseContinue();
    if (info.value?.cherry_pick_in_progress) return cherryPickContinue();
    // merge (kể cả pull dạng merge)
    busyMessage.value = "Đang hoàn tất merge…";
    try {
      await gitCommitNoEdit(path);
      await Promise.all([refreshStatusAndInfo(), refreshBranches()]);
      if (tab.value === "history") await loadHistory();
      toast.success("Đã hoàn tất merge.");
    } catch (e) {
      reportError("Không hoàn tất được merge", e);
      await refreshStatusAndInfo();
    } finally {
      busyMessage.value = "";
    }
  }

  // === Cleanup branch đã merge ===

  async function cleanupScan(): Promise<string[]> {
    const path = repoPath();
    if (!path) return [];
    busyMessage.value = "Đang quét branch đã merge…";
    try {
      return await gitCleanupScan(path);
    } catch (e) {
      reportError("Không quét được branch", e);
      return [];
    } finally {
      busyMessage.value = "";
    }
  }

  async function cleanupDelete(list: string[]) {
    const path = repoPath();
    if (!path || !list.length) return;
    try {
      const deleted = await gitCleanupDelete(path, list);
      await refreshBranches();
      toast.success(`Đã dọn ${deleted.length} branch.`);
    } catch (e) {
      reportError("Không dọn được branch", e);
    }
  }

  // === Compare / Pull Request ===

  async function compareBranches(base: string, head: string) {
    const path = repoPath();
    if (!path || !base.trim() || !head.trim()) return;
    comparisonDiff.value = null;
    busyMessage.value = "Đang so sánh…";
    try {
      comparison.value = await gitCompare(path, base, head);
    } catch (e) {
      reportError("Không so sánh được branch", e);
    } finally {
      busyMessage.value = "";
    }
  }

  async function compareSelectFile(file: GitFileChange) {
    const path = repoPath();
    const cmp = comparison.value;
    if (!path || !cmp) return;
    try {
      comparisonDiff.value = await gitCompareFileDiff(path, cmp.base, cmp.head, file.path);
    } catch (e) {
      reportError("Không đọc được diff", e);
    }
  }

  async function createPullRequest(base: string, head: string) {
    const path = repoPath();
    if (!path) return;
    try {
      const url = await gitCreatePullRequest(path, base, head);
      toast.success(`Đã mở trang tạo Pull Request: ${url}`);
    } catch (e) {
      reportError("Không tạo được Pull Request", e);
    }
  }

  /** Lấy danh sách Pull Request từ host (GitHub/GitLab), tận dụng credential đã lưu. */
  async function loadPullRequests(state: string) {
    const path = repoPath();
    if (!path) return;
    pullRequestsLoading.value = true;
    try {
      pullRequests.value = await gitListPullRequests(path, state);
    } catch (e) {
      pullRequests.value = [];
      reportError("Không lấy được danh sách Pull Request", e);
    } finally {
      pullRequestsLoading.value = false;
    }
  }

  /** Mở một URL bằng trình duyệt mặc định. */
  async function openUrl(url: string) {
    try {
      await gitOpenUrl(url);
    } catch (e) {
      reportError("Không mở được liên kết", e);
    }
  }

  /** Mở terminal tại thư mục repo hiện tại. */
  async function openTerminal() {
    const path = repoPath();
    if (!path) return;
    try {
      await gitOpenTerminal(path);
    } catch (e) {
      reportError("Không mở được terminal", e);
    }
  }

  /** Hiện một file/thư mục trong file explorer của hệ điều hành. */
  async function showInFolder(absolutePath: string) {
    try {
      await explorerOpen(absolutePath);
    } catch (e) {
      reportError("Không mở được thư mục", e);
    }
  }

  // === Commit browser ===

  async function loadBrowserCommits() {
    const path = repoPath();
    if (!path) return;
    browserLoading.value = true;
    browserFiles.value = [];
    browserDiff.value = null;
    try {
      browserCommits.value = await gitLog(path, 200);
    } catch (e) {
      reportError("Không tải được commit", e);
    } finally {
      browserLoading.value = false;
    }
  }

  async function focusBrowserCommit(hash: string) {
    const path = repoPath();
    if (!path) return;
    browserDiff.value = null;
    try {
      const detail = await gitCommitDetail(path, hash);
      browserFiles.value = detail.files;
    } catch (e) {
      reportError("Không đọc được commit", e);
    }
  }

  async function selectBrowserFile(hash: string, file: string) {
    const path = repoPath();
    if (!path) return;
    try {
      browserDiff.value = await gitCommitFileDiff(path, hash, file);
    } catch (e) {
      reportError("Không đọc được diff", e);
    }
  }

  // === Clone ===

  async function cloneRepo(url: string): Promise<boolean> {
    if (!runtimeAvailable.value || !url.trim()) return false;
    const parent = await open({ directory: true, title: "Chọn thư mục để clone vào" });
    if (!parent || typeof parent !== "string") return false;

    const name = repoNameFromUrl(url);
    const sep = parent.includes("\\") ? "\\" : "/";
    const dest = `${parent}${sep}${name}`;

    syncing.value = true;
    busyMessage.value = `Đang clone ${name}…`;
    syncProgress.value = null;
    try {
      await gitClone(url.trim(), dest, (p) => {
        syncProgress.value = p;
      });
      const repo = await gitAddRepo(dest);
      if (!repos.value.some((r) => r.id === repo.id)) {
        repos.value = [repo, ...repos.value];
      }
      await openRepo(repo);
      toast.success(`Đã clone "${name}".`);
      return true;
    } catch (e) {
      reportError("Clone thất bại", e);
      return false;
    } finally {
      syncing.value = false;
      busyMessage.value = "";
      syncProgress.value = null;
    }
  }

  function repoNameFromUrl(url: string): string {
    const trimmed = url.trim().replace(/\/+$/, "");
    const last = trimmed.split(/[/:]/).pop() ?? "repo";
    return last.replace(/\.git$/i, "") || "repo";
  }

  return {
    // state
    repos,
    activeRepo,
    info,
    staged,
    unstaged,
    branches,
    localBranches,
    remoteBranches,
    stashes,
    worktrees,
    tags,
    commits,
    comparison,
    comparisonDiff,
    pullRequests,
    pullRequestsLoading,
    conflicts,
    browserCommits,
    browserFiles,
    browserDiff,
    browserLoading,
    selectedFile,
    diff,
    diffLoading,
    selectedCommit,
    commitDetail,
    commitFileDiff,
    commitMessage,
    tab,
    loadingRepo,
    refreshing,
    committing,
    syncing,
    busyMessage,
    syncProgress,
    // computed
    runtimeAvailable,
    hasChanges,
    canCommit,
    // actions
    loadRepos,
    addRepoFromDialog,
    removeRepo,
    openRepo,
    refreshStatusAndInfo,
    refreshBranches,
    selectFile,
    stageFiles,
    unstageFiles,
    stageAll,
    unstageAll,
    discardFiles,
    commit,
    fetch,
    pull,
    push,
    loadHistory,
    selectCommit,
    selectCommitFile,
    switchTab,
    checkoutBranch,
    createBranch,
    deleteBranch,
    stashSave,
    stashApply,
    stashDrop,
    cloneRepo,
    undoLastCommit,
    resetTo,
    checkoutCommit,
    createBranchAt,
    copyText,
    revert,
    revertAbort,
    rebaseOnto,
    rebaseAbort,
    rebaseContinue,
    cherryPick,
    cherryPickAbort,
    cherryPickContinue,
    loadTags,
    createTag,
    deleteTag,
    mergeBranch,
    mergeAbort,
    compareBranches,
    compareSelectFile,
    createPullRequest,
    loadPullRequests,
    openUrl,
    openTerminal,
    showInFolder,
    loadBrowserCommits,
    focusBrowserCommit,
    selectBrowserFile,
    loadConflicts,
    resolveConflict,
    markResolved,
    finishConflict,
    cleanupScan,
    cleanupDelete,
    loadWorktrees,
    worktreeAdd,
    worktreeRemove,
    openPathAsRepo,
  };
}
