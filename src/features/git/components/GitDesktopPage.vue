<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Select from "primevue/select";
import Textarea from "primevue/textarea";
import { open } from "@tauri-apps/plugin-dialog";

import { useGit } from "../composables/useGit";
import type { GitBranch, GitCommit, GitFileChange, GitRepo } from "@/_/types/git";

const git = useGit();

onMounted(() => {
  git.loadRepos();
});

// === Popovers (repo picker / branch picker) ===
const repoMenuOpen = ref(false);
const branchMenuOpen = ref(false);
const moreMenuOpen = ref(false);
const branchFilter = ref("");

function closeMenus() {
  repoMenuOpen.value = false;
  branchMenuOpen.value = false;
  moreMenuOpen.value = false;
}

function toggleMoreMenu() {
  repoMenuOpen.value = false;
  branchMenuOpen.value = false;
  moreMenuOpen.value = !moreMenuOpen.value;
}

function toggleRepoMenu() {
  branchMenuOpen.value = false;
  repoMenuOpen.value = !repoMenuOpen.value;
}
function toggleBranchMenu() {
  repoMenuOpen.value = false;
  branchMenuOpen.value = !branchMenuOpen.value;
  branchFilter.value = "";
}

async function pickRepo(repo: GitRepo) {
  closeMenus();
  if (repo.id !== git.activeRepo.value?.id) await git.openRepo(repo);
}

const filteredBranches = computed(() => {
  const q = branchFilter.value.trim().toLowerCase();
  const list = git.branches.value;
  if (!q) return list;
  return list.filter((b) => b.name.toLowerCase().includes(q));
});

async function pickBranch(b: GitBranch) {
  closeMenus();
  const name = b.is_remote ? b.name.replace(/^[^/]+\//, "") : b.name;
  await git.checkoutBranch(name);
}

// === Dialogs ===
const cloneDialog = ref(false);
const cloneUrl = ref("");
const newBranchDialog = ref(false);
const newBranchName = ref("");
const discardDialog = ref(false);
const discardTarget = ref<{ files: string[]; label: string } | null>(null);

async function doClone() {
  const ok = await git.cloneRepo(cloneUrl.value);
  if (ok) {
    cloneDialog.value = false;
    cloneUrl.value = "";
  }
}

async function doCreateBranch() {
  const name = newBranchName.value.trim();
  if (!name) return;
  await git.createBranch(name);
  newBranchDialog.value = false;
  newBranchName.value = "";
  branchMenuOpen.value = false;
}

function askDiscard(files: string[], label: string) {
  discardTarget.value = { files, label };
  discardDialog.value = true;
}
async function confirmDiscard() {
  if (discardTarget.value) await git.discardFiles(discardTarget.value.files);
  discardDialog.value = false;
  discardTarget.value = null;
}

// === Helpers: file status presentation ===
const STATUS_META: Record<string, { label: string; cls: string; badge: string }> = {
  M: { label: "Modified", cls: "text-amber-600", badge: "bg-amber-100 text-amber-700" },
  A: { label: "Added", cls: "text-emerald-600", badge: "bg-emerald-100 text-emerald-700" },
  D: { label: "Deleted", cls: "text-red-600", badge: "bg-red-100 text-red-700" },
  R: { label: "Renamed", cls: "text-sky-600", badge: "bg-sky-100 text-sky-700" },
  C: { label: "Copied", cls: "text-sky-600", badge: "bg-sky-100 text-sky-700" },
  U: { label: "Conflict", cls: "text-red-600", badge: "bg-red-100 text-red-700" },
  "?": { label: "New", cls: "text-emerald-600", badge: "bg-emerald-100 text-emerald-700" },
};
function statusMeta(code: string) {
  return STATUS_META[code] ?? { label: code, cls: "text-muted", badge: "bg-slate-100 text-slate-600" };
}
function baseName(path: string) {
  const parts = path.split("/");
  return parts[parts.length - 1] || path;
}
function dirName(path: string) {
  const idx = path.lastIndexOf("/");
  return idx > 0 ? path.slice(0, idx) : "";
}

// === Staging selection ===
function isFileSelected(path: string, staged: boolean) {
  const s = git.selectedFile.value;
  return !!s && s.path === path && s.staged === staged;
}

const displayDiff = computed(() =>
  git.tab.value === "history" ? git.commitFileDiff.value : git.diff.value,
);

// Thu gọn/mở rộng vùng commit detail (collapse chỉ hiện tiêu đề + metadata author).
const detailExpanded = ref(true);

// === Drag-to-resize (nhớ độ rộng vào localStorage) ===
const WIDTH_KEYS = {
  changesList: "git.width.changesList",
  commitList: "git.width.commitList",
  commitFiles: "git.width.commitFiles",
} as const;

function loadWidth(key: string, def: number, min: number, max: number) {
  const raw = Number(localStorage.getItem(key) ?? "");
  return Number.isFinite(raw) && raw > 0 ? Math.max(min, Math.min(max, raw)) : def;
}

const isResizing = ref(false);
const changesListWidth = ref(loadWidth(WIDTH_KEYS.changesList, 340, 240, 600)); // cột file (tab Changes)
const changesListRef = ref<HTMLElement | null>(null);
const commitListWidth = ref(loadWidth(WIDTH_KEYS.commitList, 340, 220, 600)); // cột danh sách commit
const commitListRef = ref<HTMLElement | null>(null);
const commitFilesWidth = ref(loadWidth(WIDTH_KEYS.commitFiles, 224, 140, 500)); // cột file trong commit
const commitFilesRef = ref<HTMLElement | null>(null);
let activeMove: ((e: MouseEvent) => void) | null = null;

function persistWidths() {
  localStorage.setItem(WIDTH_KEYS.changesList, String(Math.round(changesListWidth.value)));
  localStorage.setItem(WIDTH_KEYS.commitList, String(Math.round(commitListWidth.value)));
  localStorage.setItem(WIDTH_KEYS.commitFiles, String(Math.round(commitFilesWidth.value)));
}

function beginResize(move: (e: MouseEvent) => void, e: MouseEvent) {
  e.preventDefault();
  isResizing.value = true;
  activeMove = move;
  document.addEventListener("mousemove", move);
  document.addEventListener("mouseup", endResize);
}

function endResize() {
  isResizing.value = false;
  if (activeMove) document.removeEventListener("mousemove", activeMove);
  document.removeEventListener("mouseup", endResize);
  activeMove = null;
  persistWidths();
}

function startResizeChangesList(e: MouseEvent) {
  beginResize((ev) => {
    const left = changesListRef.value?.getBoundingClientRect().left ?? 0;
    changesListWidth.value = Math.max(240, Math.min(600, ev.clientX - left));
  }, e);
}

function startResizeCommitList(e: MouseEvent) {
  beginResize((ev) => {
    const left = commitListRef.value?.getBoundingClientRect().left ?? 0;
    commitListWidth.value = Math.max(220, Math.min(600, ev.clientX - left));
  }, e);
}

function startResizeCommitFiles(e: MouseEvent) {
  beginResize((ev) => {
    const left = commitFilesRef.value?.getBoundingClientRect().left ?? 0;
    commitFilesWidth.value = Math.max(140, Math.min(500, ev.clientX - left));
  }, e);
}

onUnmounted(() => {
  if (activeMove) document.removeEventListener("mousemove", activeMove);
  document.removeEventListener("mouseup", endResize);
});

// === Rebase ===
const rebaseDialog = ref(false);
const rebaseTarget = ref("");
const rebaseOptions = computed(() =>
  git.branches.value
    .filter((b) => !b.is_current)
    .map((b) => ({ label: b.is_remote ? `${b.name} (remote)` : b.name, value: b.name })),
);
function openRebaseDialog() {
  closeMenus();
  rebaseTarget.value = "";
  rebaseDialog.value = true;
}
async function doRebase() {
  if (!rebaseTarget.value) return;
  await git.rebaseOnto(rebaseTarget.value);
  rebaseDialog.value = false;
  rebaseTarget.value = "";
}

// === Revert ===
const revertDialog = ref(false);
const revertTarget = ref<GitCommit | null>(null);
function askRevert(c: GitCommit) {
  revertTarget.value = c;
  revertDialog.value = true;
}
async function doRevert() {
  if (!revertTarget.value) return;
  await git.revert(revertTarget.value.hash);
  revertDialog.value = false;
  revertTarget.value = null;
}

// === Worktree ===
const worktreeDialog = ref(false);
const worktreeListDialog = ref(false);
const wtParent = ref("");
const wtFolder = ref("");
const wtCreateNewBranch = ref(false);
const wtExistingBranch = ref("");
const wtNewBranch = ref("");
const wtOpenAfter = ref(true);

const worktreeBranchOptions = computed(() =>
  git.localBranches.value.map((b) => ({ label: b.name, value: b.name })),
);

function resetWorktreeForm() {
  wtParent.value = "";
  wtFolder.value = "";
  wtCreateNewBranch.value = false;
  wtExistingBranch.value = git.info.value?.current_branch ?? "";
  wtNewBranch.value = "";
  wtOpenAfter.value = true;
}
function openWorktreeCreate() {
  closeMenus();
  resetWorktreeForm();
  worktreeDialog.value = true;
}
function openWorktreeList() {
  closeMenus();
  git.loadWorktrees();
  worktreeListDialog.value = true;
}
async function pickWorktreeParent() {
  const picked = await open({ directory: true, title: "Chọn thư mục cha cho worktree" });
  if (picked && typeof picked === "string") wtParent.value = picked;
}
function joinPath(parent: string, name: string) {
  const sep = parent.includes("\\") ? "\\" : "/";
  return `${parent.replace(/[/\\]+$/, "")}${sep}${name}`;
}
const worktreeCanCreate = computed(() => {
  if (!wtParent.value.trim()) return false;
  return wtCreateNewBranch.value ? !!wtNewBranch.value.trim() : !!wtExistingBranch.value;
});
async function doWorktreeCreate() {
  if (!worktreeCanCreate.value) return;
  const branchRef = wtCreateNewBranch.value ? wtNewBranch.value.trim() : wtExistingBranch.value;
  const defaultFolder = (branchRef.split(/[/\\]/).pop() || "worktree").trim();
  const folder = (wtFolder.value.trim() || defaultFolder) || "worktree";
  const fullPath = joinPath(wtParent.value, folder);
  const created = await git.worktreeAdd(
    fullPath,
    wtCreateNewBranch.value ? "" : wtExistingBranch.value,
    wtCreateNewBranch.value ? wtNewBranch.value.trim() : "",
  );
  if (created) {
    worktreeDialog.value = false;
    if (wtOpenAfter.value) await git.openPathAsRepo(created);
  }
}

// === Tag ===
const tagDialog = ref(false);
const tagName = ref("");
const tagMessage = ref("");
const tagAnnotated = ref(true);
const tagPush = ref(false);
const tagTarget = ref<{ hash: string; label: string }>({ hash: "", label: "HEAD" });
function openTagDialog(target?: { hash: string; label: string }) {
  closeMenus();
  closeCommitMenu();
  tagTarget.value = target ?? { hash: "", label: "HEAD (branch hiện tại)" };
  tagName.value = "";
  tagMessage.value = "";
  tagAnnotated.value = true;
  tagPush.value = false;
  git.loadTags();
  tagDialog.value = true;
}
async function doCreateTag() {
  const ok = await git.createTag(
    tagName.value,
    tagTarget.value.hash,
    tagMessage.value,
    tagAnnotated.value,
    tagPush.value,
  );
  if (ok) {
    tagName.value = "";
    tagMessage.value = "";
  }
}

// === Merge ===
const mergeDialog = ref(false);
const mergeBranchSel = ref("");
const mergeSquash = ref(true);
const mergeMessage = ref("");
const mergeableBranches = computed(() =>
  git.branches.value
    .filter((b) => !b.is_current && !b.name.endsWith("/HEAD"))
    .map((b) => ({ label: b.is_remote ? `${b.name} (remote)` : b.name, value: b.name })),
);
function openMergeDialog() {
  closeMenus();
  mergeBranchSel.value = "";
  mergeSquash.value = true;
  mergeMessage.value = "";
  mergeDialog.value = true;
}
async function doMerge() {
  if (!mergeBranchSel.value) return;
  const ok = await git.mergeBranch(mergeBranchSel.value, mergeSquash.value, mergeMessage.value);
  if (ok) mergeDialog.value = false;
}

// === Compare / Pull Request ===
const compareDialog = ref(false);
const cmpBase = ref("");
const cmpHead = ref("");
const allBranchRefs = computed(() =>
  git.branches.value
    .filter((b) => !b.name.endsWith("/HEAD"))
    .map((b) => ({ label: b.is_remote ? `${b.name} (remote)` : b.name, value: b.name })),
);
function guessBase(head: string): string {
  const names = git.branches.value.map((b) => b.name);
  for (const cand of ["origin/main", "origin/master", "main", "master", "develop"]) {
    if (cand !== head && names.includes(cand)) return cand;
  }
  return git.info.value?.upstream || names.find((n) => n !== head) || "";
}
function openCompareDialog() {
  closeMenus();
  cmpHead.value = git.info.value?.current_branch || "";
  cmpBase.value = guessBase(cmpHead.value);
  git.comparison.value = null;
  git.comparisonDiff.value = null;
  compareDialog.value = true;
  void runCompare();
}
async function runCompare() {
  if (cmpBase.value && cmpHead.value && cmpBase.value !== cmpHead.value) {
    await git.compareBranches(cmpBase.value, cmpHead.value);
  }
}
async function doCreatePR() {
  await git.createPullRequest(cmpBase.value, cmpHead.value);
}

// === Danh sách Pull Request ===
const prDialog = ref(false);
const prStateFilter = ref<"open" | "closed" | "all">("open");
function openPrDialog() {
  closeMenus();
  prStateFilter.value = "open";
  prDialog.value = true;
  git.loadPullRequests(prStateFilter.value);
}
function changePrState(state: "open" | "closed" | "all") {
  prStateFilter.value = state;
  git.loadPullRequests(state);
}
function prStateBadge(s: string) {
  if (s === "merged") return "bg-purple-100 text-purple-700";
  if (s === "closed") return "bg-red-100 text-red-700";
  if (s === "draft") return "bg-slate-100 text-slate-600";
  return "bg-emerald-100 text-emerald-700";
}

// Xem diff của một PR: mở lại dialog Compare với base/head của PR.
function resolveRef(name: string): string {
  const refs = git.branches.value;
  if (refs.some((b) => b.is_remote && b.name === `origin/${name}`)) return `origin/${name}`;
  if (refs.some((b) => !b.is_remote && b.name === name)) return name;
  return `origin/${name}`;
}
function openCompareForPR(pr: { base: string; head: string }) {
  prDialog.value = false;
  cmpBase.value = resolveRef(pr.base);
  cmpHead.value = resolveRef(pr.head);
  git.comparison.value = null;
  git.comparisonDiff.value = null;
  compareDialog.value = true;
  void runCompare();
}

// === Reset HEAD ===
const resetHeadDialog = ref(false);
const resetTarget = ref("HEAD");
const resetMode = ref<"soft" | "mixed" | "hard">("mixed");
function openResetHeadDialog() {
  closeMenus();
  resetTarget.value = "HEAD";
  resetMode.value = "mixed";
  resetHeadDialog.value = true;
}
async function doResetHead() {
  await git.resetTo(resetTarget.value.trim() || "HEAD", resetMode.value);
  resetHeadDialog.value = false;
}

// === Cleanup branch đã merge ===
const cleanupDialog = ref(false);
const cleanupList = ref<string[]>([]);
const cleanupSelected = ref<Set<string>>(new Set());
const cleanupScanning = ref(false);
async function openCleanupDialog() {
  closeMenus();
  cleanupDialog.value = true;
  cleanupScanning.value = true;
  cleanupList.value = [];
  cleanupSelected.value = new Set();
  cleanupList.value = await git.cleanupScan();
  cleanupSelected.value = new Set(cleanupList.value);
  cleanupScanning.value = false;
}
function toggleCleanup(name: string) {
  const s = new Set(cleanupSelected.value);
  if (s.has(name)) s.delete(name);
  else s.add(name);
  cleanupSelected.value = s;
}
async function doCleanup() {
  await git.cleanupDelete([...cleanupSelected.value]);
  cleanupDialog.value = false;
}

// === Resolve conflict ===
const conflictDialog = ref(false);
function openConflictDialog() {
  closeMenus();
  git.loadConflicts();
  conflictDialog.value = true;
}
async function doFinishConflict() {
  await git.finishConflict();
  if (!git.conflicts.value.length) conflictDialog.value = false;
}

// === Update from main/master ===
const updateDialog = ref(false);
const updateBranchSel = ref("");
function openUpdateDialog() {
  closeMenus();
  updateBranchSel.value = guessBase(git.info.value?.current_branch || "");
  updateDialog.value = true;
}
async function doUpdateFromMain() {
  if (!updateBranchSel.value) return;
  const ok = await git.mergeBranch(updateBranchSel.value, false, "");
  if (ok) updateDialog.value = false;
}

// === File actions: copy path / show in folder (context menu) ===
function absPath(rel: string): string {
  const root = git.info.value?.path || git.activeRepo.value?.path || "";
  if (!root) return rel;
  const sep = root.includes("\\") ? "\\" : "/";
  const relNorm = sep === "\\" ? rel.replace(/\//g, "\\") : rel;
  return `${root.replace(/[/\\]+$/, "")}${sep}${relNorm}`;
}
const fileMenu = ref<{ x: number; y: number; rel: string } | null>(null);
function openFileMenu(e: MouseEvent, rel: string) {
  e.preventDefault();
  e.stopPropagation();
  const x = Math.min(e.clientX, window.innerWidth - 220);
  const y = Math.min(e.clientY, window.innerHeight - 160);
  fileMenu.value = { x: Math.max(8, x), y: Math.max(8, y), rel };
  requestAnimationFrame(() => {
    window.addEventListener("click", closeFileMenu);
    window.addEventListener("contextmenu", closeFileMenu);
    window.addEventListener("scroll", closeFileMenu, true);
  });
}
function closeFileMenu() {
  fileMenu.value = null;
  window.removeEventListener("click", closeFileMenu);
  window.removeEventListener("contextmenu", closeFileMenu);
  window.removeEventListener("scroll", closeFileMenu, true);
}
onUnmounted(closeFileMenu);

// === Commit browser (duyệt commit + copy nhiều SHA) ===
const browserDialog = ref(false);
const browserSelected = ref<Set<string>>(new Set());
const browserFocusedHash = ref("");
const browserFileSel = ref("");
async function openCommitBrowser() {
  closeMenus();
  browserDialog.value = true;
  browserSelected.value = new Set();
  browserFocusedHash.value = "";
  browserFileSel.value = "";
  await git.loadBrowserCommits();
  if (git.browserCommits.value.length) focusBrowser(git.browserCommits.value[0]);
}
function focusBrowser(c: GitCommit) {
  browserFocusedHash.value = c.hash;
  browserFileSel.value = "";
  void git.focusBrowserCommit(c.hash);
}
function selectBrowserFile(f: GitFileChange) {
  browserFileSel.value = f.path;
  void git.selectBrowserFile(browserFocusedHash.value, f.path);
}
function toggleBrowserSel(hash: string) {
  const s = new Set(browserSelected.value);
  if (s.has(hash)) s.delete(hash);
  else s.add(hash);
  browserSelected.value = s;
}
function copySelectedShas() {
  const shas = git.browserCommits.value
    .filter((c) => browserSelected.value.has(c.hash))
    .map((c) => c.hash);
  if (shas.length) git.copyText(shas.join("\n"), `${shas.length} SHA`);
}

// === Context menu trên history ===
const commitMenu = ref<{ x: number; y: number; commit: GitCommit } | null>(null);
const ctxItem =
  "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-sm text-secondary transition-colors hover:bg-canvas hover:text-brand";
const ctxDanger =
  "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-sm text-red-600 transition-colors hover:bg-red-50";

function openCommitMenu(e: MouseEvent, commit: GitCommit) {
  e.preventDefault();
  e.stopPropagation();
  git.selectCommit(commit);
  // Kẹp vị trí để menu không tràn khỏi cửa sổ.
  const x = Math.min(e.clientX, window.innerWidth - 240);
  const y = Math.min(e.clientY, window.innerHeight - 320);
  commitMenu.value = { x: Math.max(8, x), y: Math.max(8, y), commit };
  requestAnimationFrame(() => {
    window.addEventListener("click", closeCommitMenu);
    window.addEventListener("contextmenu", closeCommitMenu);
    window.addEventListener("scroll", closeCommitMenu, true);
  });
}

function closeCommitMenu() {
  commitMenu.value = null;
  window.removeEventListener("click", closeCommitMenu);
  window.removeEventListener("contextmenu", closeCommitMenu);
  window.removeEventListener("scroll", closeCommitMenu, true);
}

function isTopCommit(c: GitCommit) {
  return git.commits.value[0]?.hash === c.hash;
}

// Reset (hard) — cần xác nhận vì mất dữ liệu.
const resetHardDialog = ref(false);
const resetHardCommit = ref<GitCommit | null>(null);
function askResetHard(c: GitCommit) {
  resetHardCommit.value = c;
  resetHardDialog.value = true;
}
async function doResetHard() {
  if (resetHardCommit.value) await git.resetTo(resetHardCommit.value.hash, "hard");
  resetHardDialog.value = false;
  resetHardCommit.value = null;
}

// Tạo branch từ commit.
const branchFromDialog = ref(false);
const branchFromName = ref("");
const branchFromCommit = ref<GitCommit | null>(null);
function askBranchFrom(c: GitCommit) {
  branchFromCommit.value = c;
  branchFromName.value = "";
  branchFromDialog.value = true;
}
async function doBranchFrom() {
  if (!branchFromCommit.value || !branchFromName.value.trim()) return;
  await git.createBranchAt(branchFromName.value, branchFromCommit.value.hash);
  branchFromDialog.value = false;
  branchFromName.value = "";
  branchFromCommit.value = null;
}

onUnmounted(closeCommitMenu);
</script>

<template>
  <section class="relative flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
    <!-- Runtime unavailable (browser preview) -->
    <div
      v-if="!git.runtimeAvailable.value"
      class="flex flex-1 items-center justify-center rounded-lg border border-divider bg-panel p-8 text-center text-sm text-muted"
    >
      <div>
        <i class="pi pi-desktop mb-3 block text-3xl text-muted" />
        Màn hình Git chỉ hoạt động trong ứng dụng desktop (Tauri).
      </div>
    </div>

    <template v-else>
      <!-- ======================= BOTTOM ACTION BAR (VSCode-style) ======================= -->
      <div
        class="order-last flex shrink-0 flex-wrap items-center gap-1 rounded-lg border border-divider bg-panel px-2 py-1 text-xs shadow-sm"
      >
        <!-- Repo picker -->
        <div class="relative">
          <button
            class="flex h-7 min-w-[140px] max-w-[240px] items-center gap-1.5 rounded-md px-2 transition-colors hover:bg-canvas"
            @click="toggleRepoMenu"
          >
            <i class="pi pi-book shrink-0 text-[11px] text-brand" />
            <span class="truncate font-semibold text-ink">
              {{ git.activeRepo.value?.name ?? "Chọn repository" }}
            </span>
            <i class="pi pi-chevron-up ml-auto shrink-0 text-[9px] text-muted" />
          </button>

          <div
            v-if="repoMenuOpen"
            class="absolute bottom-full left-0 z-30 mb-2 w-[320px] rounded-lg border border-divider bg-panel p-1.5 shadow-float"
          >
            <div class="max-h-64 overflow-y-auto">
              <div
                v-for="repo in git.repos.value"
                :key="repo.id"
                class="group flex items-center gap-2 rounded-md px-2.5 py-2 text-sm transition-colors hover:bg-canvas"
                :class="repo.id === git.activeRepo.value?.id ? 'bg-canvas' : ''"
              >
                <button class="flex min-w-0 flex-1 items-center gap-2 text-left" @click="pickRepo(repo)">
                  <i
                    class="pi shrink-0 text-xs"
                    :class="repo.id === git.activeRepo.value?.id ? 'pi-check text-brand' : 'pi-book text-muted'"
                  />
                  <span class="min-w-0">
                    <span class="block truncate font-medium text-ink">{{ repo.name }}</span>
                    <span class="block truncate text-[11px] text-muted">{{ repo.path }}</span>
                  </span>
                </button>
                <button
                  class="invisible shrink-0 rounded p-1 text-muted transition-colors hover:bg-red-50 hover:text-red-600 group-hover:visible"
                  title="Gỡ khỏi danh sách"
                  @click.stop="git.removeRepo(repo)"
                >
                  <i class="pi pi-times text-xs" />
                </button>
              </div>
              <div v-if="!git.repos.value.length" class="px-2.5 py-3 text-center text-xs text-muted">
                Chưa có repository nào.
              </div>
            </div>
            <div class="mt-1 flex gap-1 border-t border-divider pt-1.5">
              <button
                class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-2 text-xs font-medium text-secondary transition-colors hover:bg-canvas hover:text-brand"
                @click="closeMenus(); git.addRepoFromDialog()"
              >
                <i class="pi pi-folder-open text-xs" /> Thêm local
              </button>
              <button
                class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-2 text-xs font-medium text-secondary transition-colors hover:bg-canvas hover:text-brand"
                @click="closeMenus(); cloneDialog = true"
              >
                <i class="pi pi-cloud-download text-xs" /> Clone URL
              </button>
            </div>
          </div>
        </div>

        <!-- Branch picker (button → dropdown, giống VSCode) -->
        <div v-if="git.activeRepo.value" class="relative">
          <button
            class="flex h-7 min-w-[110px] max-w-[220px] items-center gap-1.5 rounded-md px-2 transition-colors hover:bg-canvas"
            title="Đổi branch"
            @click="toggleBranchMenu"
          >
            <i class="pi pi-sitemap shrink-0 text-[11px] text-brand" />
            <span class="truncate font-medium text-ink">
              {{ git.info.value?.detached ? "detached @ " + git.info.value?.current_branch : (git.info.value?.current_branch || "—") }}
            </span>
            <span v-if="git.info.value?.behind" class="shrink-0 text-[10px] font-semibold text-sky-600">↓{{ git.info.value.behind }}</span>
            <span v-if="git.info.value?.ahead" class="shrink-0 text-[10px] font-semibold text-emerald-600">↑{{ git.info.value.ahead }}</span>
            <i class="pi pi-chevron-up ml-auto shrink-0 text-[9px] text-muted" />
          </button>

          <div
            v-if="branchMenuOpen"
            class="absolute bottom-full left-0 z-30 mb-2 w-[300px] rounded-lg border border-divider bg-panel p-1.5 shadow-float"
          >
            <div class="relative mb-1">
              <i class="pi pi-search pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-xs text-muted" />
              <InputText
                v-model="branchFilter"
                placeholder="Lọc branch…"
                class="h-8 w-full !pl-8 text-sm"
              />
            </div>
            <div class="max-h-64 overflow-y-auto">
              <button
                v-for="b in filteredBranches"
                :key="(b.is_remote ? 'r:' : 'l:') + b.name"
                class="group flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-sm transition-colors hover:bg-canvas"
                @click="pickBranch(b)"
              >
                <i
                  class="pi shrink-0 text-xs"
                  :class="b.is_current ? 'pi-check text-brand' : b.is_remote ? 'pi-cloud text-muted' : 'pi-sitemap text-muted'"
                />
                <span class="min-w-0 flex-1 truncate" :class="b.is_current ? 'font-semibold text-brand' : 'text-ink'">
                  {{ b.name }}
                </span>
                <i
                  v-if="!b.is_remote && !b.is_current"
                  class="pi pi-trash invisible shrink-0 rounded p-1 text-muted transition-colors hover:text-red-600 group-hover:visible"
                  title="Xóa branch"
                  @click.stop="git.deleteBranch(b.name, false)"
                />
              </button>
              <div v-if="!filteredBranches.length" class="px-2.5 py-3 text-center text-xs text-muted">
                Không có branch phù hợp.
              </div>
            </div>
            <div class="mt-1 border-t border-divider pt-1.5">
              <button
                class="flex w-full items-center justify-center gap-1.5 rounded-md px-2 py-2 text-xs font-medium text-secondary transition-colors hover:bg-canvas hover:text-brand"
                @click="branchMenuOpen = false; newBranchDialog = true"
              >
                <i class="pi pi-plus text-xs" /> Tạo branch mới
              </button>
            </div>
          </div>
        </div>

        <!-- Trạng thái bận / tiến trình -->
        <div v-if="git.busyMessage.value" class="ml-2 flex items-center gap-2 text-[11px] text-muted">
          <i class="pi pi-spinner pi-spin text-[11px]" />
          <template v-if="git.syncProgress.value">
            <span class="whitespace-nowrap">
              {{ git.syncProgress.value.phase }} {{ git.syncProgress.value.percent }}%
            </span>
            <span class="h-1.5 w-24 overflow-hidden rounded-full bg-canvas">
              <span
                class="block h-full rounded-full bg-brand transition-[width] duration-150"
                :style="{ width: git.syncProgress.value.percent + '%' }"
              />
            </span>
          </template>
          <span v-else class="whitespace-nowrap">{{ git.busyMessage.value }}</span>
        </div>

        <div class="ml-auto flex items-center gap-0.5">
          <template v-if="git.activeRepo.value">
            <button
              class="flex h-7 items-center rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand"
              title="Mở terminal tại repo"
              @click="git.openTerminal()"
            >
              <i class="pi pi-desktop text-[11px]" />
            </button>
            <button
              class="flex h-7 items-center rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand"
              title="Mở thư mục repo"
              @click="git.showInFolder(git.info.value?.path || git.activeRepo.value.path)"
            >
              <i class="pi pi-folder-open text-[11px]" />
            </button>
            <button
              class="flex h-7 items-center gap-1 rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand disabled:opacity-50"
              :disabled="git.syncing.value"
              @click="git.fetch()"
            >
              <i class="pi text-[11px]" :class="git.syncing.value ? 'pi-spinner pi-spin' : 'pi-refresh'" /> Fetch
            </button>
            <button
              class="flex h-7 items-center gap-1 rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand disabled:opacity-50"
              :disabled="git.syncing.value"
              @click="git.pull()"
            >
              <i class="pi pi-arrow-down text-[11px]" /> Pull
              <span v-if="git.info.value?.behind" class="rounded-full bg-sky-100 px-1 text-[9px] font-bold text-sky-700">{{ git.info.value.behind }}</span>
            </button>
            <button
              class="flex h-7 items-center gap-1 rounded-md bg-brand px-2.5 font-medium text-white transition-colors hover:brightness-110 disabled:opacity-50"
              :disabled="git.syncing.value"
              @click="git.push()"
            >
              <i class="pi pi-arrow-up text-[11px]" /> Push
              <span v-if="git.info.value?.ahead" class="rounded-full bg-white/25 px-1 text-[9px] font-bold">{{ git.info.value.ahead }}</span>
            </button>

            <!-- More actions -->
            <div class="relative">
              <button
                class="flex h-7 items-center rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand"
                title="Thao tác khác"
                @click="toggleMoreMenu"
              >
                <i class="pi pi-ellipsis-h text-[11px]" />
              </button>
              <div
                v-if="moreMenuOpen"
                class="absolute bottom-full right-0 z-30 mb-2 w-60 rounded-lg border border-divider bg-panel p-1.5 shadow-float"
              >
                <button :class="ctxItem" @click="openRebaseDialog">
                  <i class="pi pi-arrows-v text-xs" /> Rebase branch hiện tại…
                </button>
                <button :class="ctxItem" @click="openWorktreeCreate">
                  <i class="pi pi-clone text-xs" /> Tạo worktree…
                </button>
                <button :class="ctxItem" @click="openWorktreeList">
                  <i class="pi pi-list text-xs" /> Quản lý worktree…
                </button>
                <div class="my-1 border-t border-divider" />
                <button :class="ctxItem" @click="openTagDialog()">
                  <i class="pi pi-tag text-xs" /> Tạo tag…
                </button>
                <button :class="ctxItem" @click="openMergeDialog">
                  <i class="pi pi-code-branch text-xs" /> Merge branch…
                </button>
                <button :class="ctxItem" @click="openCompareDialog">
                  <i class="pi pi-arrows-h text-xs" /> So sánh / Pull Request…
                </button>
                <button :class="ctxItem" @click="openPrDialog">
                  <i class="pi pi-flag text-xs" /> Xem Pull Requests…
                </button>
                <div class="my-1 border-t border-divider" />
                <button :class="ctxItem" @click="openUpdateDialog">
                  <i class="pi pi-arrow-circle-down text-xs" /> Cập nhật từ main/master…
                </button>
                <button :class="ctxItem" @click="openResetHeadDialog">
                  <i class="pi pi-backward text-xs" /> Reset HEAD…
                </button>
                <button :class="ctxItem" @click="openCleanupDialog">
                  <i class="pi pi-eraser text-xs" /> Cleanup branch đã merge…
                </button>
                <button :class="ctxItem" @click="openCommitBrowser">
                  <i class="pi pi-copy text-xs" /> Duyệt commit / copy SHA…
                </button>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- Rebase in progress banner -->
      <div
        v-if="git.info.value?.rebase_in_progress"
        class="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-amber-300 bg-amber-50 px-4 py-2.5 text-sm"
      >
        <i class="pi pi-exclamation-triangle text-amber-600" />
        <span class="font-semibold text-amber-800">Đang có một rebase dở dang.</span>
        <span class="text-amber-700">Giải quyết xung đột (stage file) rồi Tiếp tục, hoặc Hủy để quay lại.</span>
        <div class="ml-auto flex gap-2">
          <Button size="small" outlined severity="secondary" @click="openConflictDialog">
            <i class="pi pi-wrench mr-1.5 text-xs" /> Xử lý xung đột
          </Button>
          <Button size="small" :disabled="!!git.busyMessage.value" @click="git.rebaseContinue()">
            <i class="pi pi-play mr-1.5 text-xs" /> Tiếp tục
          </Button>
          <Button size="small" outlined severity="danger" :disabled="!!git.busyMessage.value" @click="git.rebaseAbort()">
            <i class="pi pi-times mr-1.5 text-xs" /> Hủy rebase
          </Button>
        </div>
      </div>

      <!-- Cherry-pick in progress banner -->
      <div
        v-if="git.info.value?.cherry_pick_in_progress"
        class="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-amber-300 bg-amber-50 px-4 py-2.5 text-sm"
      >
        <i class="pi pi-exclamation-triangle text-amber-600" />
        <span class="font-semibold text-amber-800">Đang có một cherry-pick dở dang.</span>
        <span class="text-amber-700">Giải quyết xung đột (stage file) rồi Tiếp tục, hoặc Hủy để quay lại.</span>
        <div class="ml-auto flex gap-2">
          <Button size="small" outlined severity="secondary" @click="openConflictDialog">
            <i class="pi pi-wrench mr-1.5 text-xs" /> Xử lý xung đột
          </Button>
          <Button size="small" :disabled="!!git.busyMessage.value" @click="git.cherryPickContinue()">
            <i class="pi pi-play mr-1.5 text-xs" /> Tiếp tục
          </Button>
          <Button size="small" outlined severity="danger" :disabled="!!git.busyMessage.value" @click="git.cherryPickAbort()">
            <i class="pi pi-times mr-1.5 text-xs" /> Hủy cherry-pick
          </Button>
        </div>
      </div>

      <!-- Merge in progress banner -->
      <div
        v-if="git.info.value?.merge_in_progress"
        class="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-amber-300 bg-amber-50 px-4 py-2.5 text-sm"
      >
        <i class="pi pi-exclamation-triangle text-amber-600" />
        <span class="font-semibold text-amber-800">Đang có một merge dở dang.</span>
        <span class="text-amber-700">Giải quyết xung đột (stage file) rồi Commit để hoàn tất, hoặc Hủy.</span>
        <div class="ml-auto flex gap-2">
          <Button size="small" outlined severity="secondary" @click="openConflictDialog">
            <i class="pi pi-wrench mr-1.5 text-xs" /> Xử lý xung đột
          </Button>
          <Button size="small" outlined severity="danger" :disabled="!!git.busyMessage.value" @click="git.mergeAbort()">
            <i class="pi pi-times mr-1.5 text-xs" /> Hủy merge
          </Button>
        </div>
      </div>

      <!-- ======================= EMPTY STATE ======================= -->
      <div
        v-if="!git.activeRepo.value"
        class="flex flex-1 flex-col items-center justify-center gap-4 rounded-lg border border-dashed border-divider bg-panel p-8 text-center"
      >
        <i class="pi pi-github text-5xl text-muted" />
        <div>
          <p class="text-base font-semibold text-ink">Chưa mở repository nào</p>
          <p class="mt-1 text-sm text-muted">Thêm một repo local có sẵn hoặc clone từ URL để bắt đầu.</p>
        </div>
        <div class="flex gap-2">
          <Button size="small" @click="git.addRepoFromDialog()">
            <i class="pi pi-folder-open mr-1.5" /> Thêm repo local
          </Button>
          <Button size="small" outlined severity="secondary" @click="cloneDialog = true">
            <i class="pi pi-cloud-download mr-1.5" /> Clone từ URL
          </Button>
        </div>
      </div>

      <!-- ======================= MAIN ======================= -->
      <div v-else class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
        <!-- ================= CHANGES TAB ================= -->
        <div
          v-show="git.tab.value === 'changes'"
          class="flex min-h-0 flex-1 overflow-hidden"
          :class="isResizing ? 'select-none' : ''"
        >
          <!-- Left: tabs + file list + commit box -->
          <div
            ref="changesListRef"
            class="flex shrink-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm"
            :style="{ width: changesListWidth + 'px' }"
          >
            <!-- Tabs (trong cột 1) -->
            <div class="flex items-center gap-1 border-b border-divider p-1">
              <button
                class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors"
                :class="git.tab.value === 'changes' ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
                @click="git.switchTab('changes')"
              >
                <i class="pi pi-pencil text-[11px]" /> Changes
                <span
                  v-if="git.hasChanges.value"
                  class="rounded-full px-1.5 text-[10px] font-bold"
                  :class="git.tab.value === 'changes' ? 'bg-white/25' : 'bg-canvas text-secondary'"
                >
                  {{ git.staged.value.length + git.unstaged.value.length }}
                </span>
              </button>
              <button
                class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors"
                :class="git.tab.value === 'history' ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
                @click="git.switchTab('history')"
              >
                <i class="pi pi-history text-[11px]" /> History
              </button>
              <button
                class="rounded p-1 text-muted transition-colors hover:bg-canvas hover:text-brand"
                title="Làm mới"
                @click="git.refreshStatusAndInfo()"
              >
                <i v-if="git.refreshing.value || git.loadingRepo.value" class="pi pi-spinner pi-spin text-[11px]" />
                <i v-else class="pi pi-sync text-[11px]" />
              </button>
            </div>

            <div class="min-h-0 flex-1 overflow-y-auto">
              <!-- Staged -->
              <div v-if="git.staged.value.length" class="border-b border-divider">
                <div class="sticky top-0 flex items-center gap-2 bg-canvas px-3 py-1.5">
                  <Checkbox :model-value="true" binary @change="git.unstageAll()" />
                  <span class="text-xs font-bold uppercase tracking-wide text-muted">Staged</span>
                  <span class="text-[11px] text-muted">({{ git.staged.value.length }})</span>
                  <button
                    class="ml-auto text-[11px] font-medium text-secondary hover:text-brand"
                    @click="git.unstageAll()"
                  >
                    Unstage all
                  </button>
                </div>
                <button
                  v-for="f in git.staged.value"
                  :key="'s:' + f.path"
                  class="group flex w-full items-center gap-2 px-3 py-1.5 text-left transition-colors hover:bg-canvas"
                  :class="isFileSelected(f.path, true) ? 'bg-canvas' : ''"
                  @click="git.selectFile(f, true)"
                  @contextmenu="openFileMenu($event, f.path)"
                >
                  <Checkbox :model-value="true" binary @click.stop @change="git.unstageFiles([f.path])" />
                  <i class="pi pi-file shrink-0 text-xs" :class="statusMeta(f.status).cls" />
                  <span class="min-w-0 flex-1 truncate text-sm">
                    <span class="text-ink">{{ baseName(f.path) }}</span>
                    <span v-if="dirName(f.path)" class="ml-1 text-[11px] text-muted">{{ dirName(f.path) }}</span>
                  </span>
                  <span class="shrink-0 text-xs font-bold" :class="statusMeta(f.status).cls">{{ f.status }}</span>
                </button>
              </div>

              <!-- Unstaged -->
              <div>
                <div class="sticky top-0 flex items-center gap-2 bg-canvas px-3 py-1.5">
                  <Checkbox
                    :model-value="false"
                    binary
                    :disabled="!git.unstaged.value.length"
                    @change="git.stageAll()"
                  />
                  <span class="text-xs font-bold uppercase tracking-wide text-muted">Changes</span>
                  <span class="text-[11px] text-muted">({{ git.unstaged.value.length }})</span>
                  <div class="ml-auto flex items-center gap-2">
                    <button
                      v-if="git.unstaged.value.length"
                      class="text-[11px] font-medium text-secondary hover:text-red-600"
                      @click="askDiscard(git.unstaged.value.map((x: GitFileChange) => x.path), 'tất cả thay đổi chưa stage')"
                    >
                      Discard all
                    </button>
                    <button
                      v-if="git.unstaged.value.length"
                      class="text-[11px] font-medium text-secondary hover:text-brand"
                      @click="git.stageAll()"
                    >
                      Stage all
                    </button>
                  </div>
                </div>
                <button
                  v-for="f in git.unstaged.value"
                  :key="'u:' + f.path"
                  class="group flex w-full items-center gap-2 px-3 py-1.5 text-left transition-colors hover:bg-canvas"
                  :class="isFileSelected(f.path, false) ? 'bg-canvas' : ''"
                  @click="git.selectFile(f, false)"
                  @contextmenu="openFileMenu($event, f.path)"
                >
                  <Checkbox :model-value="false" binary @click.stop @change="git.stageFiles([f.path])" />
                  <i class="pi pi-file shrink-0 text-xs" :class="statusMeta(f.status).cls" />
                  <span class="min-w-0 flex-1 truncate text-sm">
                    <span class="text-ink">{{ baseName(f.path) }}</span>
                    <span v-if="dirName(f.path)" class="ml-1 text-[11px] text-muted">{{ dirName(f.path) }}</span>
                  </span>
                  <i
                    class="pi pi-replay invisible shrink-0 cursor-pointer rounded p-1 text-muted transition-colors hover:text-red-600 group-hover:visible"
                    title="Discard thay đổi"
                    @click.stop="askDiscard([f.path], baseName(f.path))"
                  />
                  <span class="shrink-0 text-xs font-bold" :class="statusMeta(f.status).cls">{{ f.status }}</span>
                </button>
              </div>

              <div
                v-if="!git.hasChanges.value"
                class="flex flex-col items-center justify-center gap-2 p-8 text-center text-sm text-muted"
              >
                <i class="pi pi-check-circle text-2xl text-emerald-500" />
                Không có thay đổi nào. Working tree sạch.
              </div>
            </div>

            <!-- Commit box -->
            <div class="border-t border-divider bg-canvas p-3">
              <Textarea
                v-model="git.commitMessage.value"
                :rows="3"
                auto-resize
                placeholder="Mô tả thay đổi (commit message)…"
                class="w-full resize-none text-sm"
              />
              <div class="mt-2 flex items-center gap-2">
                <Button
                  class="flex-1"
                  size="small"
                  :disabled="!git.canCommit.value"
                  :loading="git.committing.value"
                  @click="git.commit()"
                >
                  <i class="pi pi-check mr-1.5" />
                  Commit vào {{ git.info.value?.current_branch || "branch" }}
                </Button>
                <Button
                  size="small"
                  outlined
                  severity="secondary"
                  :disabled="!git.hasChanges.value"
                  title="Cất thay đổi vào stash"
                  @click="git.stashSave('')"
                >
                  <i class="pi pi-inbox" />
                </Button>
              </div>
              <div v-if="git.staged.value.length" class="mt-1.5 text-[11px] text-muted">
                {{ git.staged.value.length }} file được stage sẽ được commit.
              </div>
            </div>
          </div>

          <!-- Resize handle: file list | diff -->
          <div
            class="flex w-2 shrink-0 cursor-col-resize items-center justify-center hover:bg-brand/10"
            :class="isResizing ? 'bg-brand/20' : ''"
            @mousedown="startResizeChangesList"
          >
            <div class="h-8 w-0.5 rounded-full bg-divider" :class="isResizing ? 'bg-brand' : ''" />
          </div>

          <!-- Right: diff -->
          <div class="flex min-w-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
            <div v-if="git.selectedFile.value" class="flex items-center gap-2 border-b border-divider bg-canvas px-3 py-2">
              <i class="pi pi-file text-xs text-muted" />
              <span class="truncate font-mono text-xs text-ink">{{ git.selectedFile.value.path }}</span>
            </div>
            <div class="min-h-0 flex-1 overflow-auto">
              <div v-if="git.diffLoading.value" class="p-4 text-sm text-muted">
                <i class="pi pi-spinner pi-spin mr-1.5" /> Đang tải diff…
              </div>
              <div
                v-else-if="!git.selectedFile.value"
                class="flex h-full items-center justify-center p-8 text-center text-sm text-muted"
              >
                Chọn một file để xem diff.
              </div>
              <div v-else-if="displayDiff && displayDiff.is_binary" class="p-4 text-sm text-muted">
                File nhị phân — không hiển thị diff.
              </div>
              <div v-else-if="displayDiff && !displayDiff.lines.length" class="p-4 text-sm text-muted">
                Không có thay đổi để hiển thị.
              </div>
              <table v-else-if="displayDiff" class="w-full border-collapse font-mono text-xs leading-5">
                <tbody>
                  <tr
                    v-for="(line, i) in displayDiff.lines"
                    :key="i"
                    :class="{
                      'bg-emerald-50': line.kind === 'add',
                      'bg-red-50': line.kind === 'del',
                      'bg-slate-100': line.kind === 'hunk',
                    }"
                  >
                    <td class="w-12 select-none border-r border-divider px-2 text-right text-[10px] text-muted">
                      {{ line.old_line || "" }}
                    </td>
                    <td class="w-12 select-none border-r border-divider px-2 text-right text-[10px] text-muted">
                      {{ line.new_line || "" }}
                    </td>
                    <td
                      class="whitespace-pre-wrap break-all px-2"
                      :class="{
                        'text-emerald-700': line.kind === 'add',
                        'text-red-700': line.kind === 'del',
                        'font-semibold text-sky-700': line.kind === 'hunk',
                        'text-secondary': line.kind === 'context',
                      }"
                    ><span class="select-none text-muted">{{ line.kind === 'add' ? '+' : line.kind === 'del' ? '-' : ' ' }}</span>{{ line.content }}</td>
                  </tr>
                </tbody>
              </table>
              <div v-if="displayDiff?.truncated" class="border-t border-divider p-2 text-center text-[11px] text-amber-600">
                Diff quá lớn — đã cắt bớt phần hiển thị.
              </div>
            </div>
          </div>
        </div>

        <!-- ================= HISTORY TAB ================= -->
        <div
          v-show="git.tab.value === 'history'"
          class="flex min-h-0 flex-1 overflow-hidden"
          :class="isResizing ? 'select-none' : ''"
        >
          <!-- Commit list -->
          <div
            ref="commitListRef"
            class="flex shrink-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm"
            :style="{ width: commitListWidth + 'px' }"
          >
            <!-- Tabs (trong cột 1) -->
            <div class="flex items-center gap-1 border-b border-divider p-1">
              <button
                class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors"
                :class="git.tab.value === 'changes' ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
                @click="git.switchTab('changes')"
              >
                <i class="pi pi-pencil text-[11px]" /> Changes
                <span
                  v-if="git.hasChanges.value"
                  class="rounded-full px-1.5 text-[10px] font-bold"
                  :class="git.tab.value === 'changes' ? 'bg-white/25' : 'bg-canvas text-secondary'"
                >
                  {{ git.staged.value.length + git.unstaged.value.length }}
                </span>
              </button>
              <button
                class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors"
                :class="git.tab.value === 'history' ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
                @click="git.switchTab('history')"
              >
                <i class="pi pi-history text-[11px]" /> History
                <span
                  v-if="git.commits.value.length"
                  class="rounded-full px-1.5 text-[10px] font-bold"
                  :class="git.tab.value === 'history' ? 'bg-white/25' : 'bg-canvas text-secondary'"
                >
                  {{ git.commits.value.length }}
                </span>
              </button>
              <button
                class="rounded p-1 text-muted transition-colors hover:bg-canvas hover:text-brand"
                title="Làm mới"
                @click="git.refreshStatusAndInfo()"
              >
                <i v-if="git.refreshing.value || git.loadingRepo.value" class="pi pi-spinner pi-spin text-[11px]" />
                <i v-else class="pi pi-sync text-[11px]" />
              </button>
            </div>

            <div class="min-h-0 flex-1 overflow-y-auto">
              <button
                v-for="c in git.commits.value"
                :key="c.hash"
                class="flex w-full flex-col gap-0.5 border-b border-divider-light px-3 py-2 text-left transition-colors hover:bg-canvas"
                :class="git.selectedCommit.value?.hash === c.hash ? 'bg-canvas' : ''"
                @click="git.selectCommit(c)"
                @contextmenu="openCommitMenu($event, c)"
              >
                <span class="truncate text-sm font-medium text-ink">{{ c.subject }}</span>
                <span class="flex items-center gap-2 text-[11px] text-muted">
                  <span class="truncate">{{ c.author_name }}</span>
                  <span>·</span>
                  <span class="shrink-0">{{ c.relative_date }}</span>
                  <span class="ml-auto shrink-0 font-mono">{{ c.short_hash }}</span>
                </span>
              </button>
              <div v-if="!git.commits.value.length" class="p-6 text-center text-sm text-muted">
                Chưa có commit nào.
              </div>
            </div>
          </div>

          <!-- Resize handle: commit list | detail -->
          <div
            class="flex w-2 shrink-0 cursor-col-resize items-center justify-center hover:bg-brand/10"
            :class="isResizing ? 'bg-brand/20' : ''"
            @mousedown="startResizeCommitList"
          >
            <div class="h-8 w-0.5 rounded-full bg-divider" :class="isResizing ? 'bg-brand' : ''" />
          </div>

          <!-- Commit detail + file diff -->
          <div class="flex min-w-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
            <div v-if="git.commitDetail.value" class="border-b border-divider bg-canvas px-4 py-2.5">
              <div class="flex items-start gap-2">
                <p class="min-w-0 flex-1 text-sm font-semibold text-ink">{{ git.commitDetail.value.commit.subject }}</p>
                <button
                  v-if="git.commitDetail.value.body"
                  class="shrink-0 rounded p-1 text-muted transition-colors hover:bg-panel hover:text-brand"
                  :title="detailExpanded ? 'Thu gọn' : 'Mở rộng'"
                  @click="detailExpanded = !detailExpanded"
                >
                  <i class="pi text-xs" :class="detailExpanded ? 'pi-chevron-up' : 'pi-chevron-down'" />
                </button>
              </div>
              <p
                v-if="git.commitDetail.value.body && detailExpanded"
                class="mt-1 whitespace-pre-wrap text-xs text-secondary"
              >
                {{ git.commitDetail.value.body }}
              </p>
              <p class="mt-1.5 flex flex-wrap items-center gap-2 text-[11px] text-muted">
                <span>{{ git.commitDetail.value.commit.author_name }}</span>
                <span>&lt;{{ git.commitDetail.value.commit.author_email }}&gt;</span>
                <span>·</span>
                <span>{{ git.commitDetail.value.commit.relative_date }}</span>
                <span class="font-mono">{{ git.commitDetail.value.commit.short_hash }}</span>
              </p>
            </div>
            <div class="flex min-h-0 flex-1 overflow-hidden" :class="isResizing ? 'select-none' : ''">
              <!-- files in commit -->
              <div
                ref="commitFilesRef"
                class="shrink-0 overflow-y-auto"
                :style="{ width: commitFilesWidth + 'px' }"
              >
                <button
                  v-for="f in git.commitDetail.value?.files ?? []"
                  :key="f.path"
                  class="flex w-full items-center gap-2 px-3 py-1.5 text-left transition-colors hover:bg-canvas"
                  :class="git.commitFileDiff.value?.path === f.path ? 'bg-canvas' : ''"
                  @click="git.selectCommitFile(f)"
                  @contextmenu="openFileMenu($event, f.path)"
                >
                  <span class="shrink-0 text-xs font-bold" :class="statusMeta(f.status).cls">{{ f.status }}</span>
                  <span class="min-w-0 flex-1 truncate text-xs text-ink" :title="f.path">{{ baseName(f.path) }}</span>
                </button>
                <div v-if="!(git.commitDetail.value?.files ?? []).length" class="p-4 text-center text-xs text-muted">
                  —
                </div>
              </div>

              <!-- Resize handle: files | diff -->
              <div
                class="flex w-2 shrink-0 cursor-col-resize items-center justify-center border-l border-divider hover:bg-brand/10"
                :class="isResizing ? 'bg-brand/20' : ''"
                @mousedown="startResizeCommitFiles"
              >
                <div class="h-8 w-0.5 rounded-full bg-divider" :class="isResizing ? 'bg-brand' : ''" />
              </div>

              <!-- diff of selected file -->
              <div class="min-h-0 flex-1 overflow-auto">
                <div v-if="git.diffLoading.value" class="p-4 text-sm text-muted">
                  <i class="pi pi-spinner pi-spin mr-1.5" /> Đang tải diff…
                </div>
                <div v-else-if="!git.commitFileDiff.value" class="flex h-full items-center justify-center p-8 text-center text-sm text-muted">
                  Chọn một file để xem diff.
                </div>
                <div v-else-if="git.commitFileDiff.value.is_binary" class="p-4 text-sm text-muted">
                  File nhị phân — không hiển thị diff.
                </div>
                <table v-else class="w-full border-collapse font-mono text-xs leading-5">
                  <tbody>
                    <tr
                      v-for="(line, i) in git.commitFileDiff.value.lines"
                      :key="i"
                      :class="{
                        'bg-emerald-50': line.kind === 'add',
                        'bg-red-50': line.kind === 'del',
                        'bg-slate-100': line.kind === 'hunk',
                      }"
                    >
                      <td class="w-12 select-none border-r border-divider px-2 text-right text-[10px] text-muted">
                        {{ line.old_line || "" }}
                      </td>
                      <td class="w-12 select-none border-r border-divider px-2 text-right text-[10px] text-muted">
                        {{ line.new_line || "" }}
                      </td>
                      <td
                        class="whitespace-pre-wrap break-all px-2"
                        :class="{
                          'text-emerald-700': line.kind === 'add',
                          'text-red-700': line.kind === 'del',
                          'font-semibold text-sky-700': line.kind === 'hunk',
                          'text-secondary': line.kind === 'context',
                        }"
                      ><span class="select-none text-muted">{{ line.kind === 'add' ? '+' : line.kind === 'del' ? '-' : ' ' }}</span>{{ line.content }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Click-away backdrop for popovers -->
    <div v-if="repoMenuOpen || branchMenuOpen || moreMenuOpen" class="fixed inset-0 z-20" @click="closeMenus" />

    <!-- ======================= DIALOGS ======================= -->
    <Dialog v-model:visible="cloneDialog" modal header="Clone repository" :style="{ width: '460px' }">
      <div class="flex flex-col gap-3">
        <label class="text-sm font-medium text-ink">URL repository</label>
        <InputText
          v-model="cloneUrl"
          placeholder="https://github.com/user/repo.git"
          class="w-full"
          @keydown.enter="doClone"
        />
        <p class="text-xs text-muted">Sau khi nhập URL, bạn sẽ chọn thư mục để clone vào.</p>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="cloneDialog = false">Hủy</Button>
        <Button size="small" :loading="git.syncing.value" :disabled="!cloneUrl.trim()" @click="doClone">
          <i class="pi pi-cloud-download mr-1.5" /> Clone
        </Button>
      </template>
    </Dialog>

    <Dialog v-model:visible="newBranchDialog" modal header="Tạo branch mới" :style="{ width: '420px' }">
      <div class="flex flex-col gap-3">
        <label class="text-sm font-medium text-ink">Tên branch</label>
        <InputText
          v-model="newBranchName"
          placeholder="feature/ten-branch"
          class="w-full"
          @keydown.enter="doCreateBranch"
        />
        <p class="text-xs text-muted">
          Tạo từ branch hiện tại (<strong>{{ git.info.value?.current_branch }}</strong>) và tự động chuyển sang.
        </p>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="newBranchDialog = false">Hủy</Button>
        <Button size="small" :disabled="!newBranchName.trim()" @click="doCreateBranch">
          <i class="pi pi-plus mr-1.5" /> Tạo branch
        </Button>
      </template>
    </Dialog>

    <Dialog v-model:visible="discardDialog" modal header="Xác nhận discard" :style="{ width: '420px' }">
      <p class="text-sm text-secondary">
        Bỏ thay đổi của <strong class="text-ink">{{ discardTarget?.label }}</strong>?
        Thao tác này không thể hoàn tác.
      </p>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="discardDialog = false">Hủy</Button>
        <Button size="small" severity="danger" @click="confirmDiscard">
          <i class="pi pi-trash mr-1.5" /> Discard
        </Button>
      </template>
    </Dialog>

    <!-- Rebase dialog -->
    <Dialog v-model:visible="rebaseDialog" modal header="Rebase branch hiện tại" :style="{ width: '460px' }">
      <div class="flex flex-col gap-3">
        <p class="text-sm text-secondary">
          Rebase <strong class="text-ink">{{ git.info.value?.current_branch }}</strong> lên trên branch:
        </p>
        <Select
          v-model="rebaseTarget"
          :options="rebaseOptions"
          option-label="label"
          option-value="value"
          placeholder="Chọn branch đích…"
          filter
          class="w-full"
        />
        <p class="text-xs text-muted">
          Nếu có xung đột, rebase sẽ tạm dừng — bạn giải quyết ở tab Changes rồi bấm "Tiếp tục" trên thanh cảnh báo.
        </p>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="rebaseDialog = false">Hủy</Button>
        <Button size="small" :disabled="!rebaseTarget || !!git.busyMessage.value" @click="doRebase">
          <i class="pi pi-arrows-v mr-1.5" /> Rebase
        </Button>
      </template>
    </Dialog>

    <!-- Revert dialog -->
    <Dialog v-model:visible="revertDialog" modal header="Revert commit" :style="{ width: '460px' }">
      <p class="text-sm text-secondary">
        Tạo một commit mới đảo ngược thay đổi của:
      </p>
      <div v-if="revertTarget" class="mt-2 rounded-md border border-divider bg-canvas p-2.5">
        <p class="text-sm font-medium text-ink">{{ revertTarget.subject }}</p>
        <p class="mt-0.5 font-mono text-[11px] text-muted">{{ revertTarget.short_hash }} · {{ revertTarget.author_name }}</p>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="revertDialog = false">Hủy</Button>
        <Button size="small" :disabled="!!git.busyMessage.value" @click="doRevert">
          <i class="pi pi-undo mr-1.5" /> Revert
        </Button>
      </template>
    </Dialog>

    <!-- Worktree create dialog -->
    <Dialog v-model:visible="worktreeDialog" modal header="Tạo worktree" :style="{ width: '520px' }">
      <div class="flex flex-col gap-3">
        <div>
          <label class="mb-1 block text-sm font-medium text-ink">Thư mục cha</label>
          <div class="flex gap-2">
            <InputText :model-value="wtParent" readonly placeholder="Chọn thư mục…" class="min-w-0 flex-1" />
            <Button size="small" outlined severity="secondary" @click="pickWorktreeParent">
              <i class="pi pi-folder-open mr-1.5" /> Chọn
            </Button>
          </div>
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium text-ink">Tên thư mục worktree</label>
          <InputText v-model="wtFolder" placeholder="(mặc định theo tên branch)" class="w-full" />
        </div>
        <div class="flex items-center gap-2">
          <Checkbox v-model="wtCreateNewBranch" binary input-id="wt-new-branch" />
          <label for="wt-new-branch" class="text-sm text-ink">Tạo branch mới (từ HEAD hiện tại)</label>
        </div>
        <div v-if="wtCreateNewBranch">
          <label class="mb-1 block text-sm font-medium text-ink">Tên branch mới</label>
          <InputText v-model="wtNewBranch" placeholder="feature/ten-branch" class="w-full" />
        </div>
        <div v-else>
          <label class="mb-1 block text-sm font-medium text-ink">Branch (đã có)</label>
          <Select
            v-model="wtExistingBranch"
            :options="worktreeBranchOptions"
            option-label="label"
            option-value="value"
            placeholder="Chọn branch…"
            filter
            class="w-full"
          />
        </div>
        <div class="flex items-center gap-2">
          <Checkbox v-model="wtOpenAfter" binary input-id="wt-open-after" />
          <label for="wt-open-after" class="text-sm text-ink">Mở worktree sau khi tạo</label>
        </div>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="worktreeDialog = false">Hủy</Button>
        <Button size="small" :disabled="!worktreeCanCreate || !!git.busyMessage.value" @click="doWorktreeCreate">
          <i class="pi pi-clone mr-1.5" /> Tạo worktree
        </Button>
      </template>
    </Dialog>

    <!-- Worktree list dialog -->
    <Dialog v-model:visible="worktreeListDialog" modal header="Worktrees" :style="{ width: '620px' }">
      <div class="flex flex-col gap-2">
        <div
          v-for="w in git.worktrees.value"
          :key="w.path"
          class="flex items-center gap-2 rounded-md border border-divider px-3 py-2"
        >
          <i class="pi shrink-0 text-sm" :class="w.is_current ? 'pi-check-circle text-brand' : 'pi-folder text-muted'" />
          <div class="min-w-0 flex-1">
            <p class="truncate font-mono text-xs text-ink">{{ w.path }}</p>
            <p class="text-[11px] text-muted">
              <span v-if="w.is_bare">bare</span>
              <span v-else-if="w.is_detached">detached @ {{ w.head.slice(0, 7) }}</span>
              <span v-else>{{ w.branch }}</span>
              <span v-if="w.is_current" class="ml-1 text-brand">· đang mở</span>
            </p>
          </div>
          <Button
            v-if="!w.is_current"
            size="small"
            outlined
            severity="secondary"
            class="shrink-0"
            title="Mở worktree này"
            @click="worktreeListDialog = false; git.openPathAsRepo(w.path)"
          >
            <i class="pi pi-external-link" />
          </Button>
          <Button
            v-if="!w.is_current && !w.is_bare"
            size="small"
            outlined
            severity="danger"
            class="shrink-0"
            title="Gỡ worktree"
            @click="git.worktreeRemove(w.path, false)"
          >
            <i class="pi pi-trash" />
          </Button>
        </div>
        <div v-if="!git.worktrees.value.length" class="p-4 text-center text-sm text-muted">
          Chưa có worktree nào.
        </div>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="worktreeListDialog = false">Đóng</Button>
        <Button size="small" @click="worktreeListDialog = false; openWorktreeCreate()">
          <i class="pi pi-plus mr-1.5" /> Tạo worktree
        </Button>
      </template>
    </Dialog>

    <!-- Tag dialog -->
    <Dialog v-model:visible="tagDialog" modal header="Tạo tag" :style="{ width: '520px' }">
      <div class="flex flex-col gap-3">
        <p class="text-xs text-muted">Tạo tag tại: <strong class="text-ink">{{ tagTarget.label }}</strong></p>
        <div>
          <label class="mb-1 block text-sm font-medium text-ink">Tên tag</label>
          <InputText v-model="tagName" placeholder="v1.0.0" class="w-full" @keydown.enter="doCreateTag" />
        </div>
        <div class="flex items-center gap-2">
          <Checkbox v-model="tagAnnotated" binary input-id="tag-annotated" />
          <label for="tag-annotated" class="text-sm text-ink">Annotated (kèm message)</label>
        </div>
        <div v-if="tagAnnotated">
          <label class="mb-1 block text-sm font-medium text-ink">Message</label>
          <InputText v-model="tagMessage" placeholder="Mô tả cho tag" class="w-full" />
        </div>
        <div class="flex items-center gap-2">
          <Checkbox v-model="tagPush" binary input-id="tag-push" />
          <label for="tag-push" class="text-sm text-ink">Push tag lên origin sau khi tạo</label>
        </div>
        <div v-if="git.tags.value.length" class="mt-1">
          <p class="mb-1 text-xs font-bold uppercase tracking-wide text-muted">Tag hiện có</p>
          <div class="max-h-40 overflow-y-auto rounded-md border border-divider">
            <div
              v-for="t in git.tags.value"
              :key="t.name"
              class="flex items-center gap-2 border-b border-divider-light px-2.5 py-1.5 last:border-0"
            >
              <i class="pi pi-tag shrink-0 text-xs text-brand" />
              <span class="min-w-0 flex-1 truncate text-sm text-ink" :title="t.subject">{{ t.name }}</span>
              <span class="shrink-0 font-mono text-[11px] text-muted">{{ t.target }}</span>
              <button
                class="shrink-0 rounded p-1 text-muted transition-colors hover:text-red-600"
                title="Xóa tag (local)"
                @click="git.deleteTag(t.name, false)"
              >
                <i class="pi pi-trash text-xs" />
              </button>
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="tagDialog = false">Đóng</Button>
        <Button size="small" :disabled="!tagName.trim() || !!git.busyMessage.value" @click="doCreateTag">
          <i class="pi pi-tag mr-1.5" /> Tạo tag
        </Button>
      </template>
    </Dialog>

    <!-- Merge dialog -->
    <Dialog v-model:visible="mergeDialog" modal header="Merge branch" :style="{ width: '480px' }">
      <div class="flex flex-col gap-3">
        <p class="text-sm text-secondary">
          Merge một branch vào <strong class="text-ink">{{ git.info.value?.current_branch }}</strong>:
        </p>
        <Select
          v-model="mergeBranchSel"
          :options="mergeableBranches"
          option-label="label"
          option-value="value"
          placeholder="Chọn branch nguồn…"
          filter
          class="w-full"
        />
        <div class="flex items-center gap-2">
          <Checkbox v-model="mergeSquash" binary input-id="merge-squash" />
          <label for="merge-squash" class="text-sm text-ink">Squash &amp; merge (gộp thành 1 commit)</label>
        </div>
        <div v-if="mergeSquash">
          <label class="mb-1 block text-sm font-medium text-ink">Commit message (tùy chọn)</label>
          <InputText
            v-model="mergeMessage"
            :placeholder="`Squash merge branch '${mergeBranchSel || '...'}'`"
            class="w-full"
          />
        </div>
        <p class="text-xs text-muted">
          Nếu có xung đột, merge sẽ tạm dừng — giải quyết ở tab Changes rồi commit để hoàn tất.
        </p>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="mergeDialog = false">Hủy</Button>
        <Button size="small" :disabled="!mergeBranchSel || !!git.busyMessage.value" @click="doMerge">
          <i class="pi pi-code-branch mr-1.5" /> {{ mergeSquash ? "Squash & merge" : "Merge" }}
        </Button>
      </template>
    </Dialog>

    <!-- Pull Requests list dialog -->
    <Dialog v-model:visible="prDialog" modal header="Pull Requests" :style="{ width: '720px' }">
      <div class="flex flex-col gap-3">
        <div class="flex items-center gap-2">
          <div class="flex overflow-hidden rounded-md border border-divider">
            <button
              v-for="opt in (['open','closed','all'] as const)"
              :key="opt"
              class="px-3 py-1 text-xs font-medium transition-colors"
              :class="prStateFilter === opt ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
              @click="changePrState(opt)"
            >
              {{ opt === 'open' ? 'Đang mở' : opt === 'closed' ? 'Đã đóng' : 'Tất cả' }}
            </button>
          </div>
          <button
            class="rounded p-1.5 text-muted transition-colors hover:bg-canvas hover:text-brand"
            title="Làm mới"
            @click="git.loadPullRequests(prStateFilter)"
          >
            <i class="pi text-xs" :class="git.pullRequestsLoading.value ? 'pi-spinner pi-spin' : 'pi-refresh'" />
          </button>
          <span class="ml-auto text-xs text-muted">Dùng credential git đã lưu để truy cập repo riêng tư.</span>
        </div>

        <div class="min-h-[280px] max-h-[440px] overflow-y-auto rounded-md border border-divider">
          <div v-if="git.pullRequestsLoading.value" class="p-8 text-center text-sm text-muted">
            <i class="pi pi-spinner pi-spin mr-1.5" /> Đang tải…
          </div>
          <div v-else-if="!git.pullRequests.value.length" class="p-8 text-center text-sm text-muted">
            Không có Pull Request nào.
          </div>
          <div
            v-for="pr in git.pullRequests.value"
            v-else
            :key="pr.number"
            class="group flex items-start gap-3 border-b border-divider-light px-3 py-2.5 transition-colors last:border-0 hover:bg-canvas"
          >
            <span class="mt-0.5 shrink-0 rounded-full px-2 py-0.5 text-[10px] font-bold uppercase" :class="prStateBadge(pr.state)">
              {{ pr.state }}
            </span>
            <span class="min-w-0 flex-1">
              <span class="block truncate text-sm font-medium text-ink">
                #{{ pr.number }} {{ pr.title }}
              </span>
              <span class="mt-0.5 flex flex-wrap items-center gap-x-2 text-[11px] text-muted">
                <span>{{ pr.author }}</span>
                <span class="font-mono">{{ pr.head }} → {{ pr.base }}</span>
              </span>
            </span>
            <button
              class="shrink-0 rounded p-1 text-muted transition-colors hover:bg-panel hover:text-brand"
              title="Xem diff của PR"
              @click="openCompareForPR(pr)"
            >
              <i class="pi pi-file-edit text-xs" />
            </button>
            <button
              class="shrink-0 rounded p-1 text-muted transition-colors hover:bg-panel hover:text-brand"
              title="Mở trên trình duyệt"
              @click="git.openUrl(pr.url)"
            >
              <i class="pi pi-external-link text-xs" />
            </button>
          </div>
        </div>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="prDialog = false">Đóng</Button>
        <Button size="small" @click="prDialog = false; openCompareDialog()">
          <i class="pi pi-plus mr-1.5" /> Tạo Pull Request
        </Button>
      </template>
    </Dialog>

    <!-- Compare / Pull Request dialog -->
    <Dialog v-model:visible="compareDialog" modal header="So sánh branch / Pull Request" :style="{ width: '840px' }">
      <div class="flex flex-col gap-3">
        <div class="flex items-end gap-2">
          <div class="min-w-0 flex-1">
            <label class="mb-1 block text-xs font-medium text-muted">Base (đích merge vào)</label>
            <Select
              v-model="cmpBase"
              :options="allBranchRefs"
              option-label="label"
              option-value="value"
              filter
              class="w-full"
              @change="runCompare"
            />
          </div>
          <i class="pi pi-arrow-left mb-2 shrink-0 text-muted" />
          <div class="min-w-0 flex-1">
            <label class="mb-1 block text-xs font-medium text-muted">Head (nguồn)</label>
            <Select
              v-model="cmpHead"
              :options="allBranchRefs"
              option-label="label"
              option-value="value"
              filter
              class="w-full"
              @change="runCompare"
            />
          </div>
        </div>

        <p v-if="cmpBase === cmpHead" class="text-xs text-amber-600">
          Base và head đang trùng nhau — hãy chọn hai branch khác nhau.
        </p>

        <div v-if="git.comparison.value" class="flex flex-wrap items-center gap-2 text-xs">
          <span class="rounded-full bg-emerald-100 px-2 py-0.5 font-semibold text-emerald-700">
            {{ git.comparison.value.ahead }} commit sẽ vào PR
          </span>
          <span v-if="git.comparison.value.behind" class="rounded-full bg-sky-100 px-2 py-0.5 font-semibold text-sky-700">
            base đi trước {{ git.comparison.value.behind }}
          </span>
          <span class="text-muted">{{ git.comparison.value.files.length }} file thay đổi</span>
        </div>

        <div v-if="git.comparison.value" class="flex h-[380px] gap-2">
          <!-- commits + files -->
          <div class="flex w-64 shrink-0 flex-col overflow-hidden rounded-md border border-divider">
            <div class="border-b border-divider bg-canvas px-2 py-1 text-[11px] font-bold uppercase tracking-wide text-muted">
              Commits ({{ git.comparison.value.commits.length }})
            </div>
            <div class="max-h-36 overflow-y-auto">
              <div
                v-for="c in git.comparison.value.commits"
                :key="c.hash"
                class="border-b border-divider-light px-2 py-1"
              >
                <p class="truncate text-xs text-ink">{{ c.subject }}</p>
                <p class="text-[10px] text-muted">{{ c.author_name }} · {{ c.short_hash }}</p>
              </div>
              <div v-if="!git.comparison.value.commits.length" class="p-3 text-center text-xs text-muted">
                Không có commit chênh lệch.
              </div>
            </div>
            <div class="border-y border-divider bg-canvas px-2 py-1 text-[11px] font-bold uppercase tracking-wide text-muted">
              Files ({{ git.comparison.value.files.length }})
            </div>
            <div class="min-h-0 flex-1 overflow-y-auto">
              <button
                v-for="f in git.comparison.value.files"
                :key="f.path"
                class="flex w-full items-center gap-2 px-2 py-1 text-left transition-colors hover:bg-canvas"
                :class="git.comparisonDiff.value?.path === f.path ? 'bg-canvas' : ''"
                @click="git.compareSelectFile(f)"
                @contextmenu="openFileMenu($event, f.path)"
              >
                <span class="shrink-0 text-xs font-bold" :class="statusMeta(f.status).cls">{{ f.status }}</span>
                <span class="min-w-0 flex-1 truncate text-xs text-ink" :title="f.path">{{ baseName(f.path) }}</span>
              </button>
            </div>
          </div>
          <!-- diff -->
          <div class="min-h-0 flex-1 overflow-auto rounded-md border border-divider">
            <div v-if="!git.comparisonDiff.value" class="flex h-full items-center justify-center p-6 text-center text-xs text-muted">
              Chọn một file để xem diff.
            </div>
            <div v-else-if="git.comparisonDiff.value.is_binary" class="p-4 text-xs text-muted">
              File nhị phân — không hiển thị diff.
            </div>
            <table v-else class="w-full border-collapse font-mono text-xs leading-5">
              <tbody>
                <tr
                  v-for="(line, i) in git.comparisonDiff.value.lines"
                  :key="i"
                  :class="{
                    'bg-emerald-50': line.kind === 'add',
                    'bg-red-50': line.kind === 'del',
                    'bg-slate-100': line.kind === 'hunk',
                  }"
                >
                  <td class="w-10 select-none border-r border-divider px-2 text-right text-[10px] text-muted">
                    {{ line.old_line || "" }}
                  </td>
                  <td class="w-10 select-none border-r border-divider px-2 text-right text-[10px] text-muted">
                    {{ line.new_line || "" }}
                  </td>
                  <td
                    class="whitespace-pre-wrap break-all px-2"
                    :class="{
                      'text-emerald-700': line.kind === 'add',
                      'text-red-700': line.kind === 'del',
                      'font-semibold text-sky-700': line.kind === 'hunk',
                      'text-secondary': line.kind === 'context',
                    }"
                  ><span class="select-none text-muted">{{ line.kind === 'add' ? '+' : line.kind === 'del' ? '-' : ' ' }}</span>{{ line.content }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
      <template #footer>
        <span
          v-if="git.comparison.value && !git.comparison.value.web_url"
          class="mr-auto text-xs text-amber-600"
        >
          Repo không có remote origin — không tạo được Pull Request.
        </span>
        <Button size="small" outlined severity="secondary" @click="compareDialog = false">Đóng</Button>
        <Button
          size="small"
          :disabled="!git.comparison.value?.web_url || !git.comparison.value?.ahead"
          @click="doCreatePR"
        >
          <i class="pi pi-external-link mr-1.5" /> Tạo Pull Request
        </Button>
      </template>
    </Dialog>

    <!-- Reset (hard) confirm -->
    <Dialog v-model:visible="resetHardDialog" modal header="Reset (hard)" :style="{ width: '470px' }">
      <p class="text-sm text-secondary">
        Reset branch về <strong class="text-ink">{{ resetHardCommit?.short_hash }}</strong> và
        <strong class="text-red-600">xóa toàn bộ thay đổi</strong> sau commit này (kể cả file đang sửa).
        Thao tác này không thể hoàn tác.
      </p>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="resetHardDialog = false">Hủy</Button>
        <Button size="small" severity="danger" :disabled="!!git.busyMessage.value" @click="doResetHard">
          <i class="pi pi-exclamation-triangle mr-1.5" /> Reset hard
        </Button>
      </template>
    </Dialog>

    <!-- Create branch from commit -->
    <Dialog v-model:visible="branchFromDialog" modal header="Tạo branch từ commit" :style="{ width: '440px' }">
      <div class="flex flex-col gap-3">
        <div v-if="branchFromCommit" class="rounded-md border border-divider bg-canvas p-2.5">
          <p class="text-sm font-medium text-ink">{{ branchFromCommit.subject }}</p>
          <p class="mt-0.5 font-mono text-[11px] text-muted">{{ branchFromCommit.short_hash }}</p>
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium text-ink">Tên branch</label>
          <InputText
            v-model="branchFromName"
            placeholder="feature/ten-branch"
            class="w-full"
            @keydown.enter="doBranchFrom"
          />
        </div>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="branchFromDialog = false">Hủy</Button>
        <Button size="small" :disabled="!branchFromName.trim()" @click="doBranchFrom">
          <i class="pi pi-sitemap mr-1.5" /> Tạo branch
        </Button>
      </template>
    </Dialog>

    <!-- Update from main/master dialog -->
    <Dialog v-model:visible="updateDialog" modal header="Cập nhật từ main/master" :style="{ width: '460px' }">
      <div class="flex flex-col gap-3">
        <p class="text-sm text-secondary">
          Merge nhánh mặc định vào <strong class="text-ink">{{ git.info.value?.current_branch }}</strong>:
        </p>
        <Select
          v-model="updateBranchSel"
          :options="allBranchRefs"
          option-label="label"
          option-value="value"
          filter
          class="w-full"
        />
        <p class="text-xs text-muted">
          Nếu có xung đột, dùng nút "Xử lý xung đột" trên thanh cảnh báo để giải quyết.
        </p>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="updateDialog = false">Hủy</Button>
        <Button size="small" :disabled="!updateBranchSel || !!git.busyMessage.value" @click="doUpdateFromMain">
          <i class="pi pi-arrow-circle-down mr-1.5" /> Cập nhật
        </Button>
      </template>
    </Dialog>

    <!-- Reset HEAD dialog -->
    <Dialog v-model:visible="resetHeadDialog" modal header="Reset HEAD" :style="{ width: '460px' }">
      <div class="flex flex-col gap-3">
        <div>
          <label class="mb-1 block text-sm font-medium text-ink">Reset về (ref/commit)</label>
          <InputText v-model="resetTarget" placeholder="HEAD, HEAD~1, origin/main…" class="w-full" />
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium text-ink">Chế độ</label>
          <div class="flex overflow-hidden rounded-md border border-divider text-xs">
            <button
              v-for="m in (['soft','mixed','hard'] as const)"
              :key="m"
              class="flex-1 px-2 py-1.5 font-medium transition-colors"
              :class="resetMode === m ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
              @click="resetMode = m"
            >
              {{ m }}
            </button>
          </div>
          <p class="mt-1 text-xs text-muted">
            <template v-if="resetMode === 'soft'">Giữ nguyên index và working tree (chỉ dời HEAD).</template>
            <template v-else-if="resetMode === 'mixed'">Giữ working tree, bỏ stage (mặc định).</template>
            <template v-else class="text-red-600">Xóa toàn bộ thay đổi working tree — không hoàn tác được.</template>
          </p>
        </div>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="resetHeadDialog = false">Hủy</Button>
        <Button
          size="small"
          :severity="resetMode === 'hard' ? 'danger' : undefined"
          :disabled="!!git.busyMessage.value"
          @click="doResetHead"
        >
          <i class="pi pi-backward mr-1.5" /> Reset ({{ resetMode }})
        </Button>
      </template>
    </Dialog>

    <!-- Cleanup dialog -->
    <Dialog v-model:visible="cleanupDialog" modal header="Cleanup branch đã merge" :style="{ width: '520px' }">
      <div class="flex flex-col gap-2">
        <p class="text-xs text-muted">
          Đã <strong>fetch --prune</strong>. Các branch local có remote đã bị xóa (thường sau khi PR đã merge &amp; xóa nhánh):
        </p>
        <div v-if="cleanupScanning" class="p-6 text-center text-sm text-muted">
          <i class="pi pi-spinner pi-spin mr-1.5" /> Đang quét…
        </div>
        <div v-else-if="!cleanupList.length" class="p-6 text-center text-sm text-muted">
          Không có branch nào cần dọn. 🎉
        </div>
        <div v-else class="max-h-64 overflow-y-auto rounded-md border border-divider">
          <label
            v-for="b in cleanupList"
            :key="b"
            class="flex cursor-pointer items-center gap-2 border-b border-divider-light px-2.5 py-1.5 last:border-0 hover:bg-canvas"
          >
            <Checkbox :model-value="cleanupSelected.has(b)" binary @change="toggleCleanup(b)" />
            <i class="pi pi-sitemap text-xs text-muted" />
            <span class="min-w-0 flex-1 truncate text-sm text-ink">{{ b }}</span>
            <span class="shrink-0 rounded-full bg-red-100 px-1.5 text-[10px] font-bold text-red-700">gone</span>
          </label>
        </div>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="cleanupDialog = false">Đóng</Button>
        <Button
          size="small"
          severity="danger"
          :disabled="!cleanupSelected.size"
          @click="doCleanup"
        >
          <i class="pi pi-trash mr-1.5" /> Xóa {{ cleanupSelected.size }} branch
        </Button>
      </template>
    </Dialog>

    <!-- Resolve conflict dialog -->
    <Dialog v-model:visible="conflictDialog" modal header="Xử lý xung đột" :style="{ width: '640px' }">
      <div class="flex flex-col gap-2">
        <p class="text-xs text-muted">
          Chọn phía giữ lại cho từng file, hoặc tự sửa file trong editor rồi bấm "Đã xử lý". Khi hết xung đột, bấm "Hoàn tất".
        </p>
        <div v-if="!git.conflicts.value.length" class="p-5 text-center text-sm text-muted">
          <i class="pi pi-check-circle mr-1.5 text-emerald-500" /> Không còn file xung đột. Bấm "Hoàn tất" để kết thúc.
        </div>
        <div v-else class="max-h-80 overflow-y-auto rounded-md border border-divider">
          <div
            v-for="f in git.conflicts.value"
            :key="f"
            class="flex items-center gap-2 border-b border-divider-light px-2.5 py-2 last:border-0"
          >
            <i class="pi pi-exclamation-triangle shrink-0 text-xs text-red-500" />
            <span class="min-w-0 flex-1 truncate font-mono text-xs text-ink" :title="f">{{ f }}</span>
            <button
              class="shrink-0 rounded border border-divider px-2 py-0.5 text-[11px] text-secondary transition-colors hover:border-brand hover:text-brand"
              title="Giữ bản HEAD (ours)"
              @click="git.resolveConflict(f, 'ours')"
            >
              Giữ HEAD
            </button>
            <button
              class="shrink-0 rounded border border-divider px-2 py-0.5 text-[11px] text-secondary transition-colors hover:border-brand hover:text-brand"
              title="Giữ bản đến (theirs)"
              @click="git.resolveConflict(f, 'theirs')"
            >
              Giữ bản đến
            </button>
            <button
              class="shrink-0 rounded border border-divider px-2 py-0.5 text-[11px] text-secondary transition-colors hover:border-brand hover:text-brand"
              title="Đã tự sửa xong (stage file)"
              @click="git.markResolved(f)"
            >
              Đã xử lý
            </button>
          </div>
        </div>
      </div>
      <template #footer>
        <Button size="small" outlined severity="secondary" @click="conflictDialog = false">Đóng</Button>
        <Button
          size="small"
          :disabled="!!git.conflicts.value.length || !!git.busyMessage.value"
          @click="doFinishConflict"
        >
          <i class="pi pi-check mr-1.5" /> Hoàn tất
        </Button>
      </template>
    </Dialog>

    <!-- Commit browser dialog -->
    <Dialog v-model:visible="browserDialog" modal header="Duyệt commit" :style="{ width: '900px' }">
      <div class="flex h-[460px] gap-2">
        <!-- commit list (multi-select) -->
        <div class="flex w-80 shrink-0 flex-col overflow-hidden rounded-md border border-divider">
          <div class="flex items-center gap-2 border-b border-divider bg-canvas px-2 py-1 text-[11px] text-muted">
            <span class="font-bold uppercase tracking-wide">Commits</span>
            <span v-if="browserSelected.size" class="ml-auto text-brand">{{ browserSelected.size }} chọn</span>
          </div>
          <div class="min-h-0 flex-1 overflow-y-auto">
            <div v-if="git.browserLoading.value" class="p-6 text-center text-sm text-muted">
              <i class="pi pi-spinner pi-spin mr-1.5" /> Đang tải…
            </div>
            <div
              v-for="c in git.browserCommits.value"
              v-else
              :key="c.hash"
              class="flex items-start gap-2 border-b border-divider-light px-2 py-1.5 transition-colors hover:bg-canvas"
              :class="browserFocusedHash === c.hash ? 'bg-canvas' : ''"
            >
              <Checkbox
                :model-value="browserSelected.has(c.hash)"
                binary
                class="mt-0.5"
                @click.stop
                @change="toggleBrowserSel(c.hash)"
              />
              <button class="min-w-0 flex-1 text-left" @click="focusBrowser(c)">
                <span class="block truncate text-sm text-ink">{{ c.subject }}</span>
                <span class="flex items-center gap-2 text-[10px] text-muted">
                  <span class="font-mono">{{ c.short_hash }}</span>
                  <span class="truncate">{{ c.author_name }}</span>
                  <span class="ml-auto shrink-0">{{ c.relative_date }}</span>
                </span>
              </button>
            </div>
          </div>
        </div>
        <!-- files of focused commit -->
        <div class="flex w-56 shrink-0 flex-col overflow-hidden rounded-md border border-divider">
          <div class="border-b border-divider bg-canvas px-2 py-1 text-[11px] font-bold uppercase tracking-wide text-muted">
            Files ({{ git.browserFiles.value.length }})
          </div>
          <div class="min-h-0 flex-1 overflow-y-auto">
            <button
              v-for="f in git.browserFiles.value"
              :key="f.path"
              class="flex w-full items-center gap-2 px-2 py-1 text-left transition-colors hover:bg-canvas"
              :class="browserFileSel === f.path ? 'bg-canvas' : ''"
              @click="selectBrowserFile(f)"
              @contextmenu="openFileMenu($event, f.path)"
            >
              <span class="shrink-0 text-xs font-bold" :class="statusMeta(f.status).cls">{{ f.status }}</span>
              <span class="min-w-0 flex-1 truncate text-xs text-ink" :title="f.path">{{ baseName(f.path) }}</span>
            </button>
            <div v-if="!git.browserFiles.value.length" class="p-4 text-center text-xs text-muted">—</div>
          </div>
        </div>
        <!-- diff -->
        <div class="min-h-0 flex-1 overflow-auto rounded-md border border-divider">
          <div v-if="!git.browserDiff.value" class="flex h-full items-center justify-center p-6 text-center text-xs text-muted">
            Chọn một file để xem diff.
          </div>
          <div v-else-if="git.browserDiff.value.is_binary" class="p-4 text-xs text-muted">File nhị phân.</div>
          <table v-else class="w-full border-collapse font-mono text-xs leading-5">
            <tbody>
              <tr
                v-for="(line, i) in git.browserDiff.value.lines"
                :key="i"
                :class="{
                  'bg-emerald-50': line.kind === 'add',
                  'bg-red-50': line.kind === 'del',
                  'bg-slate-100': line.kind === 'hunk',
                }"
              >
                <td class="w-10 select-none border-r border-divider px-2 text-right text-[10px] text-muted">{{ line.old_line || "" }}</td>
                <td class="w-10 select-none border-r border-divider px-2 text-right text-[10px] text-muted">{{ line.new_line || "" }}</td>
                <td
                  class="whitespace-pre-wrap break-all px-2"
                  :class="{
                    'text-emerald-700': line.kind === 'add',
                    'text-red-700': line.kind === 'del',
                    'font-semibold text-sky-700': line.kind === 'hunk',
                    'text-secondary': line.kind === 'context',
                  }"
                ><span class="select-none text-muted">{{ line.kind === 'add' ? '+' : line.kind === 'del' ? '-' : ' ' }}</span>{{ line.content }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
      <template #footer>
        <span class="mr-auto text-xs text-muted">Tick chọn nhiều commit rồi copy SHA.</span>
        <Button size="small" outlined severity="secondary" @click="browserDialog = false">Đóng</Button>
        <Button size="small" :disabled="!browserSelected.size" @click="copySelectedShas">
          <i class="pi pi-copy mr-1.5" /> Copy {{ browserSelected.size }} SHA
        </Button>
      </template>
    </Dialog>

    <!-- File context menu (copy path / show in folder) -->
    <div
      v-if="fileMenu"
      class="fixed z-40 w-52 rounded-lg border border-divider bg-panel p-1 shadow-float"
      :style="{ left: fileMenu.x + 'px', top: fileMenu.y + 'px' }"
      @click.stop
    >
      <button :class="ctxItem" @click="closeFileMenu(); git.copyText(absPath(fileMenu.rel), 'đường dẫn')">
        <i class="pi pi-copy text-xs" /> Copy path
      </button>
      <button :class="ctxItem" @click="closeFileMenu(); git.copyText(fileMenu.rel, 'đường dẫn tương đối')">
        <i class="pi pi-copy text-xs" /> Copy relative path
      </button>
      <button :class="ctxItem" @click="closeFileMenu(); git.showInFolder(absPath(fileMenu.rel))">
        <i class="pi pi-folder-open text-xs" /> Show in folder
      </button>
    </div>

    <!-- Context menu: history commit -->
    <div
      v-if="commitMenu"
      class="fixed z-40 w-56 rounded-lg border border-divider bg-panel p-1 shadow-float"
      :style="{ left: commitMenu.x + 'px', top: commitMenu.y + 'px' }"
      @click.stop
    >
      <button v-if="isTopCommit(commitMenu.commit)" :class="ctxItem" @click="closeCommitMenu(); git.undoLastCommit()">
        <i class="pi pi-replay text-xs" /> Undo commit này
      </button>
      <button :class="ctxItem" @click="closeCommitMenu(); askRevert(commitMenu.commit)">
        <i class="pi pi-undo text-xs" /> Revert commit…
      </button>
      <button :class="ctxItem" @click="closeCommitMenu(); git.cherryPick(commitMenu.commit.hash)">
        <i class="pi pi-share-alt text-xs" /> Cherry-pick vào branch hiện tại
      </button>
      <div class="my-1 border-t border-divider" />
      <button :class="ctxItem" @click="closeCommitMenu(); askBranchFrom(commitMenu.commit)">
        <i class="pi pi-sitemap text-xs" /> Tạo branch từ đây…
      </button>
      <button :class="ctxItem" @click="closeCommitMenu(); git.checkoutCommit(commitMenu.commit.hash)">
        <i class="pi pi-arrow-right text-xs" /> Checkout commit (detached)
      </button>
      <button :class="ctxItem" @click="openTagDialog({ hash: commitMenu.commit.hash, label: commitMenu.commit.short_hash + ' — ' + commitMenu.commit.subject })">
        <i class="pi pi-tag text-xs" /> Tạo tag tại đây…
      </button>
      <div class="my-1 border-t border-divider" />
      <button :class="ctxItem" @click="closeCommitMenu(); git.resetTo(commitMenu.commit.hash, 'mixed')">
        <i class="pi pi-history text-xs" /> Reset về đây (giữ thay đổi)
      </button>
      <button :class="ctxDanger" @click="closeCommitMenu(); askResetHard(commitMenu.commit)">
        <i class="pi pi-exclamation-triangle text-xs" /> Reset về đây (xóa thay đổi)…
      </button>
      <div class="my-1 border-t border-divider" />
      <button :class="ctxItem" @click="closeCommitMenu(); git.copyText(commitMenu.commit.hash, 'SHA')">
        <i class="pi pi-copy text-xs" /> Copy SHA
      </button>
      <button :class="ctxItem" @click="closeCommitMenu(); git.copyText(commitMenu.commit.subject, 'commit message')">
        <i class="pi pi-copy text-xs" /> Copy message
      </button>
    </div>
  </section>
</template>
