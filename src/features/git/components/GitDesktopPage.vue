<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import InputText from "primevue/inputtext";
import Textarea from "primevue/textarea";

import { useGit } from "../composables/useGit";
import type { GitBranch, GitCommit, GitFileChange, GitRepo } from "@/_/types/git";
import { statusMeta, baseName, dirName } from "../utils/fileStatus";
import { guessBase } from "../utils/gitRefs";

import GitCloneRepoDialog from "./GitCloneRepoDialog.vue";
import GitNewBranchDialog from "./GitNewBranchDialog.vue";
import GitDiscardConfirmDialog from "./GitDiscardConfirmDialog.vue";
import GitRebaseDialog from "./GitRebaseDialog.vue";
import GitRevertDialog from "./GitRevertDialog.vue";
import GitWorktreeCreateDialog from "./GitWorktreeCreateDialog.vue";
import GitWorktreeListDialog from "./GitWorktreeListDialog.vue";
import GitStashListDialog from "./GitStashListDialog.vue";
import GitStashSaveDialog from "./GitStashSaveDialog.vue";
import GitTagDialog from "./GitTagDialog.vue";
import GitTagListDialog from "./GitTagListDialog.vue";
import GitMergeDialog from "./GitMergeDialog.vue";
import GitPullRequestsDialog from "./GitPullRequestsDialog.vue";
import GitCompareDialog from "./GitCompareDialog.vue";
import GitResetHeadDialog from "./GitResetHeadDialog.vue";
import GitCleanupDialog from "./GitCleanupDialog.vue";
import GitConflictDialog from "./GitConflictDialog.vue";
import GitUpdateFromMainDialog from "./GitUpdateFromMainDialog.vue";
import GitResetHardDialog from "./GitResetHardDialog.vue";
import GitBranchFromCommitDialog from "./GitBranchFromCommitDialog.vue";
import GitGraphDialog from "./GitGraphDialog.vue";
import GitCommitBrowserDialog from "./GitCommitBrowserDialog.vue";
import GitBlameDialog from "./GitBlameDialog.vue";
import GitLogDialog from "./GitLogDialog.vue";

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
const cloneDialogVisible = ref(false);
const newBranchDialogVisible = ref(false);
const discardDialogVisible = ref(false);
const discardTarget = ref<{ files: string[]; label: string } | null>(null);

function askDiscard(files: string[], label: string) {
  discardTarget.value = { files, label };
  discardDialogVisible.value = true;
}

// === Staging selection ===
function isFileSelected(path: string, staged: boolean) {
  const s = git.selectedFile.value;
  return !!s && s.path === path && s.staged === staged;
}

const displayDiff = computed(() =>
  git.tab.value === "history" ? git.commitFileDiff.value : git.diff.value,
);

const isOnBaseBranch = computed(() => {
  const cur = git.info.value?.current_branch;
  if (!cur) return false;
  const base = guessBase(
    git.branches.value.map((b) => b.name),
    "",
    git.info.value?.upstream,
  );
  return cur === base || cur === base.replace(/^origin\//, "");
});

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
const rebaseDialogVisible = ref(false);
function openRebaseDialog() {
  rebaseDialogVisible.value = true;
}

// === Revert ===
const revertDialogVisible = ref(false);
const revertTarget = ref<GitCommit | null>(null);
function askRevert(c: GitCommit) {
  revertTarget.value = c;
  revertDialogVisible.value = true;
}

// === Worktree ===
const worktreeCreateDialogVisible = ref(false);
const worktreeListDialogVisible = ref(false);
function openWorktreeCreate() {
  worktreeCreateDialogVisible.value = true;
}
function openWorktreeList() {
  worktreeListDialogVisible.value = true;
}

// === Stash ===
const stashListDialogVisible = ref(false);
const stashSaveDialogVisible = ref(false);
function openStashList() {
  stashListDialogVisible.value = true;
}
function openStashSave() {
  stashSaveDialogVisible.value = true;
}

// === Tag ===
const tagDialogVisible = ref(false);
const tagListDialogVisible = ref(false);
const tagTarget = ref<{ hash: string; label: string }>({ hash: "", label: "HEAD" });
function openTagDialog(target?: { hash: string; label: string }) {
  closeCommitMenu();
  tagTarget.value = target ?? { hash: "", label: "HEAD (branch hiện tại)" };
  tagDialogVisible.value = true;
}
function openTagListDialog() {
  tagListDialogVisible.value = true;
}

// === Merge ===
const mergeDialogVisible = ref(false);
function openMergeDialog() {
  mergeDialogVisible.value = true;
}

// === Compare / Pull Request ===
const compareDialogVisible = ref(false);
const comparePr = ref<{ base: string; head: string } | null>(null);
function openCompareDialog() {
  comparePr.value = null;
  compareDialogVisible.value = true;
}
function handleOpenCompare(pr: { base: string; head: string } | null) {
  comparePr.value = pr;
  compareDialogVisible.value = true;
}

// === Danh sách Pull Request ===
const prDialogVisible = ref(false);
function openPrDialog() {
  prDialogVisible.value = true;
}

// === Reset HEAD ===
const resetHeadDialogVisible = ref(false);
function openResetHeadDialog() {
  resetHeadDialogVisible.value = true;
}

// === Cleanup branch đã merge ===
const cleanupDialogVisible = ref(false);
function openCleanupDialog() {
  cleanupDialogVisible.value = true;
}

// === Resolve conflict ===
const conflictDialogVisible = ref(false);
function openConflictDialog() {
  conflictDialogVisible.value = true;
}

// === Update from main/master ===
const updateDialogVisible = ref(false);
function openUpdateDialog() {
  updateDialogVisible.value = true;
}

// === File actions: copy path / show in folder (context menu) ===
// Git (kể cả trên Windows) luôn trả path với "/" — app chỉ chạy trên Windows
// nên chuẩn hóa về "\" để khớp định dạng đường dẫn Windows quen thuộc.
function winPath(p: string): string {
  return p.replace(/\//g, "\\");
}
function absPath(rel: string): string {
  const root = git.info.value?.path || git.activeRepo.value?.path || "";
  if (!root) return winPath(rel);
  return `${winPath(root).replace(/\\+$/, "")}\\${winPath(rel)}`;
}
function relPath(rel: string): string {
  return winPath(rel);
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

// === Git blame ===
const blameDialogVisible = ref(false);
const blameDialogFile = ref("");
function openBlameDialog(rel: string) {
  blameDialogFile.value = rel;
  blameDialogVisible.value = true;
  closeFileMenu();
}
async function openBlameFromHistory(hash: string) {
  git.switchTab("history");
  if (!git.commits.value.length) await git.loadHistory();
  const found = git.commits.value.find((c) => c.hash === hash);
  await git.selectCommit(
    found ?? {
      hash,
      short_hash: hash.slice(0, 7),
      subject: "",
      author_name: "",
      author_email: "",
      date: "",
      relative_date: "",
    },
  );
}

// === Commit browser (duyệt commit + copy nhiều SHA) ===
const browserDialogVisible = ref(false);
function openCommitBrowser() {
  browserDialogVisible.value = true;
}

// === Show Log ===
const logDialogVisible = ref(false);
function openLogDialog() {
  logDialogVisible.value = true;
}

// === Visualization (đồ thị commit) ===
const graphDialogVisible = ref(false);
function openGraphDialog() {
  graphDialogVisible.value = true;
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
const resetHardDialogVisible = ref(false);
const resetHardCommit = ref<GitCommit | null>(null);
function askResetHard(c: GitCommit) {
  resetHardCommit.value = c;
  resetHardDialogVisible.value = true;
}

// Tạo branch từ commit.
const branchFromDialogVisible = ref(false);
const branchFromCommit = ref<GitCommit | null>(null);
function askBranchFrom(c: GitCommit) {
  branchFromCommit.value = c;
  branchFromDialogVisible.value = true;
}

async function handleAmendCommit(c: GitCommit) {
  git.commitMessage.value = c.subject;
  await git.undoLastCommit();
  git.switchTab("changes");
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
                @click="closeMenus(); cloneDialogVisible = true"
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
                @click="branchMenuOpen = false; newBranchDialogVisible = true"
              >
                <i class="pi pi-plus text-xs" /> Tạo branch mới
              </button>
            </div>
          </div>
        </div>

        <!-- Fetch -->
        <button
          v-if="git.activeRepo.value"
          class="flex h-7 items-center gap-1 rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand disabled:opacity-50"
          :disabled="git.syncing.value"
          @click="git.fetch()"
        >
          <i class="pi text-[11px]" :class="git.syncing.value ? 'pi-spinner pi-spin' : 'pi-refresh'" />
        </button>

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
              class="flex h-7 items-center gap-1 rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand"
              title="Visualization — đồ thị commit"
              @click="openGraphDialog"
            >
              <i class="pi pi-sitemap text-[11px]" /> Graph
            </button>
            <button
              class="flex h-7 items-center rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand"
              title="Mở repo bằng VS Code"
              @click="git.openVscode()"
            >
              <i class="pi pi-code text-[11px]" />
            </button>
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
              @click="git.pull()"
            >
              <i class="pi pi-arrow-down text-[11px]" />
              <span v-if="git.info.value?.behind" class="badge-info">{{ git.info.value.behind }}</span>
            </button>
            <button
              class="flex h-7 items-center gap-1 rounded-md bg-brand px-2.5 font-medium text-white transition-colors hover:brightness-110 disabled:opacity-50"
              :disabled="git.syncing.value"
              @click="git.push()"
            >
              <i class="pi pi-arrow-up text-[11px]" />
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
                  <i class="pi pi-arrows-v text-xs" /> Rebase current branch…
                </button>
                <button :class="ctxItem" @click="openWorktreeCreate">
                  <i class="pi pi-clone text-xs" /> Tạo worktree…
                </button>
                <button :class="ctxItem" @click="openWorktreeList">
                  <i class="pi pi-list text-xs" /> Quản lý worktree…
                </button>
                <div class="my-1 border-t border-divider" />
                <button :class="ctxItem" @click="openStashList">
                  <i class="pi pi-inbox text-xs" /> Quản lý stash…
                  <span v-if="git.stashes.value.length" class="ml-auto rounded-full bg-canvas px-1.5 text-[10px] font-semibold text-muted">{{ git.stashes.value.length }}</span>
                </button>
                <div class="my-1 border-t border-divider" />
                <button :class="ctxItem" @click="openTagDialog()">
                  <i class="pi pi-tag text-xs" /> Tạo tag…
                </button>
                <button :class="ctxItem" @click="openTagListDialog">
                  <i class="pi pi-tags text-xs" /> Quản lý tags…
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
                <button v-if="!isOnBaseBranch" :class="ctxItem" @click="openUpdateDialog">
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
                <button :class="ctxItem" @click="logDialogVisible = true;">
                  <i class="pi pi-list-check text-xs" /> Show log…
                </button>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- Rebase in progress banner -->
      <div
        v-if="git.info.value?.rebase_in_progress"
        class="banner-warning flex-wrap"
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
        class="banner-warning flex-wrap"
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
        class="banner-warning flex-wrap"
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
          <Button size="small" outlined severity="secondary" @click="cloneDialogVisible = true">
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
                  @click="openStashSave"
                >
                  <i class="pi pi-inbox" />
                </Button>
                <Button
                  v-if="git.stashes.value.length"
                  size="small"
                  outlined
                  severity="secondary"
                  title="Xem danh sách stash"
                  @click="openStashList"
                >
                  <i class="pi pi-list" />
                  <span class="ml-1">{{ git.stashes.value.length }}</span>
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
    <GitCloneRepoDialog v-model:visible="cloneDialogVisible" :git="git" />
    <GitNewBranchDialog v-model:visible="newBranchDialogVisible" :git="git" />
    <GitDiscardConfirmDialog v-model:visible="discardDialogVisible" :git="git" :target="discardTarget" />
    <GitRebaseDialog v-model:visible="rebaseDialogVisible" :git="git" />
    <GitRevertDialog v-model:visible="revertDialogVisible" :git="git" :target="revertTarget" />
    <GitWorktreeCreateDialog v-model:visible="worktreeCreateDialogVisible" :git="git" />
    <GitWorktreeListDialog
      v-model:visible="worktreeListDialogVisible"
      :git="git"
      @create-worktree="worktreeCreateDialogVisible = true"
    />
    <GitStashListDialog v-model:visible="stashListDialogVisible" :git="git" />
    <GitStashSaveDialog v-model:visible="stashSaveDialogVisible" :git="git" />
    <GitTagDialog v-model:visible="tagDialogVisible" :git="git" :target="tagTarget" />
    <GitTagListDialog
      v-model:visible="tagListDialogVisible"
      :git="git"
      @create-tag="tagListDialogVisible = false; openTagDialog()"
    />
    <GitMergeDialog v-model:visible="mergeDialogVisible" :git="git" />
    <GitPullRequestsDialog
      v-model:visible="prDialogVisible"
      :git="git"
      @open-compare="handleOpenCompare"
    />
    <GitCompareDialog
      v-model:visible="compareDialogVisible"
      :git="git"
      :pr="comparePr"
      :on-file-context="openFileMenu"
    />
    <GitResetHeadDialog v-model:visible="resetHeadDialogVisible" :git="git" />
    <GitCleanupDialog v-model:visible="cleanupDialogVisible" :git="git" />
    <GitConflictDialog v-model:visible="conflictDialogVisible" :git="git" />
    <GitUpdateFromMainDialog v-model:visible="updateDialogVisible" :git="git" />
    <GitResetHardDialog v-model:visible="resetHardDialogVisible" :git="git" :target="resetHardCommit" />
    <GitBranchFromCommitDialog v-model:visible="branchFromDialogVisible" :git="git" :target="branchFromCommit" />
    <GitGraphDialog v-model:visible="graphDialogVisible" :git="git" :on-file-context="openFileMenu" />
    <GitCommitBrowserDialog v-model:visible="browserDialogVisible" :git="git" :on-file-context="openFileMenu" />
    <GitLogDialog v-model:visible="logDialogVisible" :git="git" />
    <GitBlameDialog
      v-model:visible="blameDialogVisible"
      :git="git"
      :file="blameDialogFile"
      @open-in-history="openBlameFromHistory"
    />

    <!-- File context menu (copy path / show in folder) -->
    <div
      v-if="fileMenu"
      class="fixed z-40 w-52 rounded-lg border border-divider bg-panel p-1 shadow-float"
      :style="{ left: fileMenu.x + 'px', top: fileMenu.y + 'px' }"
      @click.stop
    >
      <button :class="ctxItem" @click="git.copyText(absPath(fileMenu.rel), 'đường dẫn'); closeFileMenu()">
        <i class="pi pi-copy text-xs" /> Copy path
      </button>
      <button :class="ctxItem" @click="git.copyText(relPath(fileMenu.rel), 'đường dẫn tương đối'); closeFileMenu()">
        <i class="pi pi-copy text-xs" /> Copy relative path
      </button>
      <button :class="ctxItem" @click="git.showInFolder(absPath(fileMenu.rel)); closeFileMenu()">
        <i class="pi pi-folder-open text-xs" /> Show in folder
      </button>
      <div class="my-1 border-t border-divider" />
      <button :class="ctxItem" @click="openBlameDialog(fileMenu.rel)">
        <i class="pi pi-user-edit text-xs" /> Xem Git blame
      </button>
    </div>

    <!-- Context menu: history commit -->
    <div
      v-if="commitMenu"
      class="fixed z-40 w-56 rounded-lg border border-divider bg-panel p-1 shadow-float"
      :style="{ left: commitMenu.x + 'px', top: commitMenu.y + 'px' }"
      @click.stop
    >
      <button v-if="isTopCommit(commitMenu.commit)" :class="ctxItem" @click="handleAmendCommit(commitMenu.commit); closeCommitMenu()">
        <i class="pi pi-pencil text-xs" /> Amend commit
      </button>
      <button v-if="isTopCommit(commitMenu.commit) && (git.info.value?.ahead ?? 0) > 0" :class="ctxItem" @click="git.undoLastCommit(); closeCommitMenu()">
        <i class="pi pi-replay text-xs" /> Undo commit
      </button>
      <button :class="ctxItem" @click="askRevert(commitMenu.commit); closeCommitMenu()">
        <i class="pi pi-undo text-xs" /> Revert commit
      </button>
      <button :class="ctxItem" @click="git.cherryPick(commitMenu.commit.hash); closeCommitMenu()">
        <i class="pi pi-share-alt text-xs" /> Cherry-pick commit
      </button>
      <div class="my-1 border-t border-divider" />
      <button :class="ctxItem" @click="askBranchFrom(commitMenu.commit); closeCommitMenu()">
        <i class="pi pi-sitemap text-xs" /> Tạo branch from commit
      </button>
      <button :class="ctxItem" @click="git.checkoutCommit(commitMenu.commit.hash); closeCommitMenu()">
        <i class="pi pi-arrow-right text-xs" /> Checkout commit
      </button>
      <button :class="ctxItem" @click="openTagDialog({ hash: commitMenu.commit.hash, label: commitMenu.commit.short_hash + ' — ' + commitMenu.commit.subject })">
        <i class="pi pi-tag text-xs" /> Tạo tag
      </button>
      <div class="my-1 border-t border-divider" />
      <button :class="ctxItem" @click="git.resetTo(commitMenu.commit.hash, 'mixed'); closeCommitMenu()">
        <i class="pi pi-history text-xs" /> Reset commit
      </button>
      <button :class="ctxDanger" @click="askResetHard(commitMenu.commit); closeCommitMenu()">
        <i class="pi pi-exclamation-triangle text-xs" /> Reset hard (xóa commit)
      </button>
      <div class="my-1 border-t border-divider" />
      <button :class="ctxItem" @click="git.copyText(commitMenu.commit.hash, 'SHA'); closeCommitMenu()">
        <i class="pi pi-copy text-xs" /> Copy SHA
      </button>
      <button :class="ctxItem" @click="git.copyText(commitMenu.commit.subject, 'commit message'); closeCommitMenu()">
        <i class="pi pi-copy text-xs" /> Copy message
      </button>
    </div>
  </section>
</template>
