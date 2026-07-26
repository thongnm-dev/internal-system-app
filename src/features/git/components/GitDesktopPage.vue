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
  <section class="relative flex min-h-0 flex-1 flex-col gap-3 overflow-hidden">
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
      <!-- ======================= TOOLBAR ======================= -->
      <div
        class="flex flex-wrap items-center gap-2 rounded-lg border border-divider bg-panel px-3 py-2 shadow-sm"
      >
        <!-- Repo picker -->
        <div class="relative">
          <button
            class="flex h-9 min-w-[180px] max-w-[280px] items-center gap-2 rounded-md border border-divider bg-canvas px-3 text-sm transition-colors hover:border-brand"
            @click="toggleRepoMenu"
          >
            <i class="pi pi-book shrink-0 text-brand" />
            <span class="truncate font-semibold text-ink">
              {{ git.activeRepo.value?.name ?? "Chọn repository" }}
            </span>
            <i class="pi pi-chevron-down ml-auto shrink-0 text-[10px] text-muted" />
          </button>

          <div
            v-if="repoMenuOpen"
            class="absolute left-0 top-11 z-30 w-[320px] rounded-lg border border-divider bg-panel p-1.5 shadow-float"
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

        <!-- Branch picker -->
        <div v-if="git.activeRepo.value" class="relative">
          <button
            class="flex h-9 min-w-[150px] max-w-[240px] items-center gap-2 rounded-md border border-divider bg-canvas px-3 text-sm transition-colors hover:border-brand"
            @click="toggleBranchMenu"
          >
            <i class="pi pi-sitemap shrink-0 text-brand" />
            <span class="truncate font-medium text-ink">
              {{ git.info.value?.detached ? "detached @ " + git.info.value?.current_branch : (git.info.value?.current_branch || "—") }}
            </span>
            <i class="pi pi-chevron-down ml-auto shrink-0 text-[10px] text-muted" />
          </button>

          <div
            v-if="branchMenuOpen"
            class="absolute left-0 top-11 z-30 w-[300px] rounded-lg border border-divider bg-panel p-1.5 shadow-float"
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

        <div class="ml-auto flex items-center gap-1.5">
          <template v-if="git.activeRepo.value">
            <Button
              size="small"
              outlined
              severity="secondary"
              :loading="git.syncing.value"
              @click="git.fetch()"
            >
              <i class="pi pi-refresh mr-1.5 text-xs" /> Fetch
            </Button>
            <Button
              size="small"
              outlined
              severity="secondary"
              :disabled="git.syncing.value"
              @click="git.pull()"
            >
              <i class="pi pi-arrow-down mr-1.5 text-xs" /> Pull
              <span v-if="git.info.value?.behind" class="ml-1.5 rounded-full bg-sky-100 px-1.5 text-[10px] font-bold text-sky-700">
                {{ git.info.value.behind }}
              </span>
            </Button>
            <Button
              size="small"
              :disabled="git.syncing.value"
              @click="git.push()"
            >
              <i class="pi pi-arrow-up mr-1.5 text-xs" /> Push
              <span v-if="git.info.value?.ahead" class="ml-1.5 rounded-full bg-white/25 px-1.5 text-[10px] font-bold">
                {{ git.info.value.ahead }}
              </span>
            </Button>

            <!-- More actions -->
            <div class="relative">
              <Button size="small" outlined severity="secondary" title="Thao tác khác" @click="toggleMoreMenu">
                <i class="pi pi-ellipsis-h" />
              </Button>
              <div
                v-if="moreMenuOpen"
                class="absolute right-0 top-11 z-30 w-60 rounded-lg border border-divider bg-panel p-1.5 shadow-float"
              >
                <button
                  class="flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm text-secondary transition-colors hover:bg-canvas hover:text-brand"
                  @click="openRebaseDialog"
                >
                  <i class="pi pi-arrows-v text-xs" /> Rebase branch hiện tại…
                </button>
                <button
                  class="flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm text-secondary transition-colors hover:bg-canvas hover:text-brand"
                  @click="openWorktreeCreate"
                >
                  <i class="pi pi-clone text-xs" /> Tạo worktree…
                </button>
                <button
                  class="flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm text-secondary transition-colors hover:bg-canvas hover:text-brand"
                  @click="openWorktreeList"
                >
                  <i class="pi pi-list text-xs" /> Quản lý worktree…
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
          <Button size="small" :disabled="!!git.busyMessage.value" @click="git.rebaseContinue()">
            <i class="pi pi-play mr-1.5 text-xs" /> Tiếp tục
          </Button>
          <Button size="small" outlined severity="danger" :disabled="!!git.busyMessage.value" @click="git.rebaseAbort()">
            <i class="pi pi-times mr-1.5 text-xs" /> Hủy rebase
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
      <div v-else class="flex min-h-0 flex-1 flex-col gap-3 overflow-hidden">
        <!-- Tabs -->
        <div class="flex items-center gap-1 rounded-lg border border-divider bg-panel p-1 shadow-sm">
          <button
            class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
            :class="git.tab.value === 'changes' ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
            @click="git.switchTab('changes')"
          >
            <i class="pi pi-pencil text-xs" /> Changes
            <span
              v-if="git.hasChanges.value"
              class="rounded-full px-1.5 text-[10px] font-bold"
              :class="git.tab.value === 'changes' ? 'bg-white/25' : 'bg-canvas text-secondary'"
            >
              {{ git.staged.value.length + git.unstaged.value.length }}
            </span>
          </button>
          <button
            class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
            :class="git.tab.value === 'history' ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
            @click="git.switchTab('history')"
          >
            <i class="pi pi-history text-xs" /> History
          </button>

          <div class="ml-auto flex items-center gap-2 px-2 text-xs text-muted">
            <i v-if="git.refreshing.value || git.loadingRepo.value" class="pi pi-spinner pi-spin" />
            <span v-if="git.busyMessage.value">{{ git.busyMessage.value }}</span>
            <button
              class="rounded p-1 transition-colors hover:bg-canvas hover:text-brand"
              title="Làm mới"
              @click="git.refreshStatusAndInfo()"
            >
              <i class="pi pi-sync text-xs" />
            </button>
          </div>
        </div>

        <!-- ================= CHANGES TAB ================= -->
        <div
          v-show="git.tab.value === 'changes'"
          class="flex min-h-0 flex-1 overflow-hidden"
          :class="isResizing ? 'select-none' : ''"
        >
          <!-- Left: file list + commit box -->
          <div
            ref="changesListRef"
            class="flex shrink-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm"
            :style="{ width: changesListWidth + 'px' }"
          >
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
            <div class="border-b border-divider px-3 py-2 text-xs font-bold uppercase tracking-wide text-muted">
              Commits ({{ git.commits.value.length }})
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
                <Button
                  size="small"
                  outlined
                  severity="secondary"
                  class="shrink-0"
                  title="Revert commit này"
                  @click="askRevert(git.commitDetail.value.commit)"
                >
                  <i class="pi pi-undo mr-1.5 text-xs" /> Revert
                </Button>
              </div>
              <p v-if="git.commitDetail.value.body" class="mt-1 whitespace-pre-wrap text-xs text-secondary">
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
      <div class="my-1 border-t border-divider" />
      <button :class="ctxItem" @click="closeCommitMenu(); askBranchFrom(commitMenu.commit)">
        <i class="pi pi-sitemap text-xs" /> Tạo branch từ đây…
      </button>
      <button :class="ctxItem" @click="closeCommitMenu(); git.checkoutCommit(commitMenu.commit.hash)">
        <i class="pi pi-arrow-right text-xs" /> Checkout commit (detached)
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
