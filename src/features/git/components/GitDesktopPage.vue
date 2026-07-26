<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Textarea from "primevue/textarea";

import { useGit } from "../composables/useGit";
import type { GitBranch, GitFileChange, GitRepo } from "@/_/types/git";

const git = useGit();

onMounted(() => {
  git.loadRepos();
});

// === Popovers (repo picker / branch picker) ===
const repoMenuOpen = ref(false);
const branchMenuOpen = ref(false);
const branchFilter = ref("");

function closeMenus() {
  repoMenuOpen.value = false;
  branchMenuOpen.value = false;
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

// === Drag-to-resize (History tab) ===
const isResizing = ref(false);
const commitListWidth = ref(340); // cột danh sách commit
const commitListRef = ref<HTMLElement | null>(null);
const commitFilesWidth = ref(224); // cột file trong commit
const commitFilesRef = ref<HTMLElement | null>(null);
let activeMove: ((e: MouseEvent) => void) | null = null;

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
          </template>
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
        <div v-show="git.tab.value === 'changes'" class="flex min-h-0 flex-1 gap-3 overflow-hidden">
          <!-- Left: file list + commit box -->
          <div class="flex w-[340px] shrink-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
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
              <p class="text-sm font-semibold text-ink">{{ git.commitDetail.value.commit.subject }}</p>
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
    <div v-if="repoMenuOpen || branchMenuOpen" class="fixed inset-0 z-20" @click="closeMenus" />

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
  </section>
</template>
