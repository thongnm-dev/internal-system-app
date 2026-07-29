<script setup lang="ts">
import { ref, watch, computed, onUnmounted } from "vue";
import Button from "primevue/button";
import Column from "primevue/column";
import DataTable from "primevue/datatable";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import DatePicker from "primevue/datepicker";
import type { GitApi } from "../composables/useGit";
import type { GitCommit, GitCommitDetail } from "@/_/types/git";
import { gitLogSearch, gitCommitDetail } from "@/tauri/commands/git";
import { statusMeta, baseName } from "../utils/fileStatus";
import { useDataTablePagination } from "@/shared/composables/useDataTablePagination";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });
const { paginationCompact: pg } = useDataTablePagination();

const isMaximized = ref(false);

const WIDTH_KEY = "gitlog.width.commitList";
function loadWidth(def: number, min: number, max: number) {
  const raw = Number(localStorage.getItem(WIDTH_KEY) ?? "");
  return Number.isFinite(raw) && raw > 0 ? Math.max(min, Math.min(max, raw)) : def;
}
const isResizing = ref(false);
const commitListWidth = ref(loadWidth(420, 280, 700));
const commitListRef = ref<HTMLElement | null>(null);
let activeMove: ((e: MouseEvent) => void) | null = null;

function endResize() {
  isResizing.value = false;
  if (activeMove) document.removeEventListener("mousemove", activeMove);
  document.removeEventListener("mouseup", endResize);
  activeMove = null;
  localStorage.setItem(WIDTH_KEY, String(Math.round(commitListWidth.value)));
}

function startResize(e: MouseEvent) {
  e.preventDefault();
  isResizing.value = true;
  const move = (ev: MouseEvent) => {
    const left = commitListRef.value?.getBoundingClientRect().left ?? 0;
    commitListWidth.value = Math.max(280, Math.min(700, ev.clientX - left));
  };
  activeMove = move;
  document.addEventListener("mousemove", move);
  document.addEventListener("mouseup", endResize);
}

onUnmounted(() => {
  if (activeMove) document.removeEventListener("mousemove", activeMove);
  document.removeEventListener("mouseup", endResize);
});

const loading = ref(false);
const commits = ref<GitCommit[]>([]);
const first = ref(0);
const rows = ref(pg.rows);
const totalRecords = ref(0);

const filterFrom = ref<Date | null>(null);
const filterTo = ref<Date | null>(null);
const filterMessage = ref("");
const filterAuthor = ref("");
const filterFile = ref("");

const selectedCommit = ref<GitCommit | null>(null);
const detail = ref<GitCommitDetail | null>(null);
const detailLoading = ref(false);

function fmtDate(d: Date | null): string {
  if (!d) return "";
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${dd}`;
}

const repoPath = computed(() => props.git.activeRepo.value?.path ?? "");

async function search(resetPage = true) {
  if (!repoPath.value) return;
  if (resetPage) first.value = 0;
  loading.value = true;
  try {
    const result = await gitLogSearch(
      repoPath.value,
      fmtDate(filterFrom.value),
      fmtDate(filterTo.value),
      filterAuthor.value.trim(),
      filterMessage.value.trim(),
      filterFile.value.trim(),
      first.value,
      rows.value + 1,
    );
    if (!result) {
      commits.value = [];
      totalRecords.value = 0;
      return;
    }
    const hasMore = result.length > rows.value;
    commits.value = hasMore ? result.slice(0, rows.value) : result;
    totalRecords.value = hasMore
      ? first.value + rows.value + 1
      : first.value + result.length;
  } finally {
    loading.value = false;
  }
}

function onPage(e: { first: number; rows: number }) {
  first.value = e.first;
  rows.value = e.rows;
  search(false);
}

function clearFilters() {
  filterFrom.value = null;
  filterTo.value = null;
  filterMessage.value = "";
  filterAuthor.value = "";
  filterFile.value = "";
  search();
}

async function selectCommit(c: GitCommit) {
  selectedCommit.value = c;
  detail.value = null;
  if (!repoPath.value) return;
  detailLoading.value = true;
  try {
    const d = await gitCommitDetail(repoPath.value, c.hash);
    if (d) detail.value = d;
  } finally {
    detailLoading.value = false;
  }
}

watch(visible, (v) => {
  if (!v) return;
  isMaximized.value = false;
  selectedCommit.value = null;
  detail.value = null;
  commits.value = [];
  first.value = 0;
  rows.value = pg.rows;
  totalRecords.value = 0;
  filterFrom.value = null;
  filterTo.value = null;
  filterMessage.value = "";
  filterAuthor.value = "";
  filterFile.value = "";
  search();
});
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    maximizable
    header="Git Log"
    :style="{ width: '95vw'}"
    @maximize="isMaximized = true"
    @unmaximize="isMaximized = false"
  >
    <div class="flex flex-col gap-3">
      <!-- Search filters -->
      <div class="flex flex-wrap items-end gap-2 rounded-lg border border-divider bg-canvas p-3">
        <div class="flex flex-col gap-1">
          <label class="text-[11px] font-medium text-muted">From</label>
          <DatePicker
            v-model="filterFrom"
            date-format="yy-mm-dd"
            placeholder="From date"
            show-icon
            show-button-bar
            class="w-40"
            input-class="text-sm h-8"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] font-medium text-muted">To</label>
          <DatePicker
            v-model="filterTo"
            date-format="yy-mm-dd"
            placeholder="To date"
            show-icon
            show-button-bar
            class="w-40"
            input-class="text-sm h-8"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] font-medium text-muted">Message</label>
          <InputText
            v-model="filterMessage"
            placeholder="Commit message…"
            class="h-8 w-48 text-sm"
            @keydown.enter="search()"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] font-medium text-muted">Author</label>
          <InputText
            v-model="filterAuthor"
            placeholder="Author…"
            class="h-8 w-36 text-sm"
            @keydown.enter="search()"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] font-medium text-muted">File</label>
          <InputText
            v-model="filterFile"
            placeholder="File path…"
            class="h-8 w-48 text-sm"
            @keydown.enter="search()"
          />
        </div>
        <Button size="small" :loading="loading" @click="search()">
          <i class="pi pi-search mr-1.5" /> Tìm
        </Button>
        <Button size="small" outlined severity="secondary" @click="clearFilters">
          <i class="pi pi-filter-slash mr-1.5" /> Xóa bộ lọc
        </Button>
      </div>

      <!-- Content area -->
      <div
        class="flex"
        :class="[
          isMaximized ? 'h-[calc(100vh-320px)]' : 'h-[60vh]',
          isResizing ? 'select-none' : '',
        ]"
      >
        <!-- Commit list -->
        <div
          ref="commitListRef"
          class="flex shrink-0 flex-col overflow-hidden rounded-md border border-divider"
          :style="{ width: commitListWidth + 'px' }"
        >
          <DataTable
            :value="commits"
            :loading="loading"
            lazy
            paginator
            :first="first"
            :rows="rows"
            :rows-per-page-options="pg.rowsPerPageOptions"
            :total-records="totalRecords"
            selection-mode="single"
            :meta-key-selection="false"
            :selection="selectedCommit"
            data-key="hash"
            scrollable
            scroll-height="flex"
            class="gitlog-table flex-1"
            @page="onPage"
            @row-select="(e: any) => selectCommit(e.data)"
          >
            <template #empty>
              <div class="p-4 text-center text-sm text-muted">Không tìm thấy commit nào.</div>
            </template>
            <Column field="subject" header="Commit">
              <template #body="{ data }">
                <div class="flex flex-col gap-0.5">
                  <span class="truncate text-sm font-medium text-ink">{{ data.subject }}</span>
                  <span class="flex items-center gap-2 text-[10px] text-muted">
                    <span class="truncate">{{ data.author_name }}</span>
                    <span>·</span>
                    <span class="shrink-0">{{ data.date?.slice(0, 10) }}</span>
                    <span class="ml-auto shrink-0 font-mono">{{ data.short_hash }}</span>
                  </span>
                </div>
              </template>
            </Column>
          </DataTable>
        </div>

        <!-- Resize handle -->
        <div
          class="flex w-2 shrink-0 cursor-col-resize items-center justify-center hover:bg-brand/10"
          :class="isResizing ? 'bg-brand/20' : ''"
          @mousedown="startResize"
        >
          <div class="h-8 w-0.5 rounded-full bg-divider" :class="isResizing ? 'bg-brand' : ''" />
        </div>

        <!-- Commit detail -->
        <div class="flex min-w-0 flex-1 flex-col overflow-hidden rounded-md border border-divider">
          <template v-if="selectedCommit">
            <!-- Header -->
            <div class="border-b border-divider bg-canvas px-4 py-2.5">
              <p class="text-sm font-semibold text-ink">{{ selectedCommit.subject }}</p>
              <p v-if="detail?.body" class="mt-1 whitespace-pre-wrap text-xs text-secondary">{{ detail.body }}</p>
              <p class="mt-1.5 flex flex-wrap items-center gap-2 text-[11px] text-muted">
                <span>{{ selectedCommit.author_name }}</span>
                <span>&lt;{{ selectedCommit.author_email }}&gt;</span>
                <span>·</span>
                <span>{{ selectedCommit.date?.slice(0, 10) }}</span>
                <span class="font-mono">{{ selectedCommit.short_hash }}</span>
                <button
                  class="rounded px-1.5 py-0.5 text-[10px] font-medium text-secondary transition-colors hover:bg-panel hover:text-brand"
                  title="Copy SHA"
                  @click="git.copyText(selectedCommit!.hash, 'SHA')"
                >
                  <i class="pi pi-copy text-[10px]" /> Copy SHA
                </button>
              </p>
            </div>
            <!-- Files -->
            <div v-if="detailLoading" class="p-4 text-sm text-muted">
              <i class="pi pi-spinner pi-spin mr-1.5" /> Đang tải chi tiết…
            </div>
            <div v-else-if="detail" class="min-h-0 flex-1 overflow-y-auto">
              <div class="border-b border-divider bg-canvas px-3 py-1 text-[11px] font-bold uppercase tracking-wide text-muted">
                Files thay đổi ({{ detail.files.length }})
              </div>
              <div
                v-for="f in detail.files"
                :key="f.path"
                class="flex items-center gap-2 border-b border-divider-light px-3 py-1.5 text-sm"
              >
                <span class="shrink-0 text-xs font-bold" :class="statusMeta(f.status).cls">{{ f.status }}</span>
                <span class="min-w-0 flex-1 truncate text-xs text-ink" :title="f.path">{{ f.path }}</span>
              </div>
              <div v-if="!detail.files.length" class="p-4 text-center text-xs text-muted">—</div>
            </div>
          </template>
          <div v-else class="flex h-full items-center justify-center p-8 text-center text-sm text-muted">
            Chọn một commit để xem chi tiết.
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Đóng</Button>
    </template>
  </Dialog>
</template>

<style scoped>
.gitlog-table :deep(.p-datatable-thead > tr > th) {
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.gitlog-table :deep(.p-datatable-tbody > tr > td) {
  padding: 6px 12px;
}
.gitlog-table :deep(.p-datatable-tbody > tr) {
  cursor: pointer;
}
.gitlog-table :deep(.p-datatable-paginator-bottom) {
  padding: 4px 8px;
  min-height: unset;
}
</style>
