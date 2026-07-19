<script setup lang="ts">
import { ref, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import Button from "primevue/button";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import { useAuthStore } from "@/app/stores/auth";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { getProjectDetail } from "@/tauri/commands/project";
import {
  backlogGetProjectLookup,
  backlogListStatuses,
  backlogListProjectUsers,
  backlogListIssueTypes,
  backlogListCategories,
  backlogListIssues,
  type BacklogStatus,
  type BacklogUser,
  type BacklogIssue,
  type BacklogIssueType as BacklogIssueTypeModel,
  type BacklogCategory as BacklogCategoryModel,
} from "@/tauri/commands/backlog";
import {
  useProjectTasks,
  type ProjectTask,
  type ProjectTaskInput,
} from "../composables/useProjectTasks";
import { useToast } from "@/shared/composables/useToast";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();
const projectId = (route.params.id as string) || "";

const ctrl = useProjectTasks(projectId);
const toast = useToast();
const globalLoading = useGlobalLoading();

// Chỉ cho phép liên kết Backlog khi dự án đã thiết lập Backlog key.
const hasBacklog = computed(() => Boolean(ctrl.project.value?.backlog_key));

// --- Backlog import dialog ---
type BacklogRow = {
  issueKey: string;
  summary: string;
  assignee: string;
  status: string;
  issueType: string;
  estimatedHours: number | null;
  dueDate: string;
  _selected: boolean;
};

const backlogDialogVisible = ref(false);
const backlogLoading = ref(false);
const backlogSearching = ref(false);
const backlogError = ref("");
const backlogRows = ref<BacklogRow[]>([]);
const backlogSelectedCount = computed(() => backlogRows.value.filter((r) => r._selected).length);
const backlogProjectName = ref("");

const blStatusOptions = ref<BacklogStatus[]>([]);
const blIssueTypeOptions = ref<BacklogIssueTypeModel[]>([]);
const blCategoryOptions = ref<BacklogCategoryModel[]>([]);
const blUserOptions = ref<BacklogUser[]>([]);
const blProjectId = ref("");
const blMatchedUser = ref<BacklogUser | null>(null);

const blSelectedStatuses = ref<number[]>([]);
const blSelectedIssueType = ref("");
const blSelectedCategory = ref("");
const blNotClosed = ref(false);

function isClosedStatus(name: string) {
  const n = name.toLowerCase();
  return n.includes("closed") || n.includes("close") || n === "完了";
}

function formatDate(dateStr: string | null | undefined): string {
  if (!dateStr) return "";
  return dateStr.substring(0, 10);
}

function mapBacklogIssue(issue: BacklogIssue): BacklogRow {
  return {
    issueKey: issue.issueKey,
    summary: issue.summary,
    assignee: issue.assignee?.name ?? "",
    status: issue.status?.name ?? "",
    issueType: issue.issueType?.name ?? "",
    estimatedHours: issue.estimatedHours,
    dueDate: formatDate(issue.dueDate),
    _selected: true,
  };
}

function toggleBlStatus(id: number) {
  blNotClosed.value = false;
  const idx = blSelectedStatuses.value.indexOf(id);
  if (idx >= 0) {
    blSelectedStatuses.value = blSelectedStatuses.value.filter((v) => v !== id);
  } else {
    blSelectedStatuses.value = [...blSelectedStatuses.value, id];
  }
}

function selectBlNotClosed() {
  blSelectedStatuses.value = [];
  blNotClosed.value = true;
}

async function openBacklogDialog() {
  if (!canUseTauriRuntime()) return;

  backlogDialogVisible.value = true;
  backlogLoading.value = true;
  backlogError.value = "";
  backlogRows.value = [];
  backlogProjectName.value = "";

  await globalLoading.run(async () => {
    try {
      const numericId = Number(projectId);
      if (Number.isNaN(numericId)) throw new Error("Invalid project ID");

      const detail = await getProjectDetail(numericId);
      const backlogKey = detail.backlog_key;
      if (!backlogKey) throw new Error("Project has no Backlog key configured.");

      backlogProjectName.value = detail.name || detail.code;

      const [statuses, issueTypesList, cats, users, lookup] = await Promise.all([
        backlogListStatuses(backlogKey),
        backlogListIssueTypes(backlogKey),
        backlogListCategories(backlogKey),
        backlogListProjectUsers(backlogKey),
        backlogGetProjectLookup(backlogKey),
      ]);

      blStatusOptions.value = statuses;
      blIssueTypeOptions.value = issueTypesList;
      blCategoryOptions.value = cats;
      blUserOptions.value = users;
      blProjectId.value = lookup.projectId.toString();

      const username = auth.user?.username ?? "";
      const matched = users.find(
        (u: BacklogUser) =>
          u.userId === username ||
          u.name.toLowerCase() === username.toLowerCase() ||
          u.mailAddress?.split("@")[0]?.toLowerCase() === username.toLowerCase(),
      );
      blMatchedUser.value = matched ?? null;

      blSelectedStatuses.value = [];
      blNotClosed.value = true;
      blSelectedIssueType.value = "";
      blSelectedCategory.value = "";

      await searchBacklog();
    } catch (e) {
      backlogError.value = String(e);
    } finally {
      backlogLoading.value = false;
    }
  });
}

async function searchBacklog() {
  backlogSearching.value = true;
  backlogError.value = "";
  backlogRows.value = [];

  globalLoading.start();
  try {
    const issueTypeIds = blSelectedIssueType.value
      ? [blIssueTypeOptions.value.find((t) => t.name === blSelectedIssueType.value)?.id].filter((id): id is number => id !== undefined)
      : undefined;

    const categoryIds = blSelectedCategory.value
      ? [blCategoryOptions.value.find((c) => c.name === blSelectedCategory.value)?.id].filter((id): id is number => id !== undefined)
      : undefined;

    const assigneeIds = blMatchedUser.value ? [blMatchedUser.value.id] : undefined;

    const statusIds = blNotClosed.value
      ? blStatusOptions.value.filter((s) => !isClosedStatus(s.name)).map((s) => s.id)
      : blSelectedStatuses.value;

    const result = await backlogListIssues({
      projectKey: blProjectId.value,
      count: 100,
      offset: 0,
      statusIds: statusIds.length ? statusIds : undefined,
      issueTypeIds: issueTypeIds?.length ? issueTypeIds : undefined,
      categoryIds: categoryIds?.length ? categoryIds : undefined,
      assigneeIds,
      sort: "created",
      order: "desc",
    });

    backlogRows.value = result.issues.map(mapBacklogIssue);
  } catch (e) {
    backlogError.value = String(e);
  } finally {
    backlogSearching.value = false;
    globalLoading.stop();
  }
}

function toggleBacklogAll(checked: boolean) {
  backlogRows.value.forEach((r) => (r._selected = checked));
}

function confirmBacklogSelection() {
  const selected = backlogRows.value.filter((r) => r._selected);
  parsed.value = selected.map((row, i) => ({
    _selected: true,
    _row: i + 1,
    shortName: row.summary,
    description: "",
    categories: [] as ProjectTask["categories"],
    assignee: row.assignee,
    estimateHour: row.estimatedHours != null ? String(row.estimatedHours) : "",
    dueDate: row.dueDate,
    issueKey: row.issueKey,
  }));
  updateSelectedCount();
  backlogDialogVisible.value = false;
}

type ParsedRow = ProjectTaskInput & { _selected: boolean; _row: number };

const parsed = ref<ParsedRow[]>([]);
const importError = ref("");
const fileInput = ref<HTMLInputElement | null>(null);

const selectedCount = ref(0);

function updateSelectedCount() {
  selectedCount.value = parsed.value.filter((r) => r._selected).length;
}

function toggleAll(checked: boolean) {
  parsed.value.forEach((r) => (r._selected = checked));
  updateSelectedCount();
}

function goBack() {
  router.push(`/projects/${encodeURIComponent(projectId)}/tasks`);
}

function triggerFileInput() {
  fileInput.value?.click();
}

function handleFileChange(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  importError.value = "";
  parsed.value = [];

  const reader = new FileReader();
  reader.onload = () => {
    try {
      const text = reader.result as string;
      const rows = parseCsv(text);
      if (rows.length === 0) {
        importError.value = "No valid rows found in the CSV file.";
        return;
      }
      parsed.value = rows;
      updateSelectedCount();
    } catch (e) {
      importError.value = String(e);
    }
  };
  reader.onerror = () => {
    importError.value = "Failed to read the file.";
  };
  reader.readAsText(file);
}

function parseCsv(text: string): ParsedRow[] {
  const lines = text.split(/\r?\n/).filter((l) => l.trim());
  if (lines.length < 2) return [];

  const headers = lines[0].split(",").map((h) => h.trim().toLowerCase());

  const nameIdx = headers.findIndex((h) => h === "short name" || h === "shortname" || h === "name" || h === "task");
  const descIdx = headers.findIndex((h) => h === "description" || h === "desc");
  const catIdx = headers.findIndex((h) => h === "category" || h === "categories");
  const assigneeIdx = headers.findIndex((h) => h === "assignee");
  const estIdx = headers.findIndex((h) => h === "estimate" || h === "estimate hour" || h === "estimatehour" || h === "hours");
  const dueIdx = headers.findIndex((h) => h === "due date" || h === "duedate" || h === "due");
  const issueIdx = headers.findIndex((h) => h === "issue" || h === "issue key" || h === "issuekey");

  if (nameIdx < 0) {
    throw new Error("CSV must have a 'short name' or 'name' column in the header row.");
  }

  const rows: ParsedRow[] = [];
  for (let i = 1; i < lines.length; i++) {
    const cols = splitCsvLine(lines[i]);
    const shortName = (cols[nameIdx] ?? "").trim();
    if (!shortName) continue;

    const catRaw = catIdx >= 0 ? (cols[catIdx] ?? "").trim() : "";
    const categories = catRaw ? catRaw.split(/[;|]/).map((c) => c.trim()).filter(Boolean) : [];

    rows.push({
      _selected: true,
      _row: i + 1,
      shortName,
      description: descIdx >= 0 ? (cols[descIdx] ?? "").trim() : "",
      categories: categories as ProjectTask["categories"],
      assignee: assigneeIdx >= 0 ? (cols[assigneeIdx] ?? "").trim() : "",
      estimateHour: estIdx >= 0 ? (cols[estIdx] ?? "").trim() : "",
      dueDate: dueIdx >= 0 ? (cols[dueIdx] ?? "").trim() : "",
      issueKey: issueIdx >= 0 ? (cols[issueIdx] ?? "").trim() : "",
    });
  }

  return rows;
}

function splitCsvLine(line: string): string[] {
  const result: string[] = [];
  let current = "";
  let inQuote = false;

  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (inQuote) {
      if (ch === '"') {
        if (i + 1 < line.length && line[i + 1] === '"') {
          current += '"';
          i++;
        } else {
          inQuote = false;
        }
      } else {
        current += ch;
      }
    } else if (ch === '"') {
      inQuote = true;
    } else if (ch === ",") {
      result.push(current);
      current = "";
    } else {
      current += ch;
    }
  }
  result.push(current);
  return result;
}

async function importSelected() {
  const selected = parsed.value.filter((r) => r._selected);
  await globalLoading.run(async () => {
    for (const row of selected) {
      await ctrl.addTask({
        shortName: row.shortName,
        description: row.description,
        categories: row.categories,
        assignee: row.assignee,
        estimateHour: row.estimateHour,
        dueDate: row.dueDate,
        issueKey: row.issueKey,
      });
    }
  });
  toast.success(`Imported ${selected.length} task${selected.length !== 1 ? "s" : ""} successfully.`);
  goBack();
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Header -->
    <section class="flex items-center justify-between gap-4 rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="min-w-0">
        <h3 class="truncate font-bold text-ink">Import Tasks</h3>
        <p class="mt-1 truncate text-sm text-muted">
          {{ ctrl.projectLabel.value }}
        </p>
      </div>
        <Button
          icon="pi pi-check"
          :label="`Import ${selectedCount} task${selectedCount !== 1 ? 's' : ''}`"
          :disabled="selectedCount === 0"
          @click="importSelected"
        />
    </section>

    <!-- File upload area -->
    <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="flex items-center gap-3">
        <input
          ref="fileInput"
          type="file"
          accept=".csv"
          class="hidden"
          @change="handleFileChange"
        />
        <Button icon="pi pi-upload" label="Choose CSV file" @click="triggerFileInput" />
        <Button
          icon="pi pi-list-check"
          label="Import from Backlog"
          outlined
          :disabled="!hasBacklog || ctrl.projectLoading.value"
          :title="hasBacklog ? 'Import issues from Backlog' : 'Project has no Backlog configured'"
          @click="openBacklogDialog"
        />
        <span class="text-sm text-muted">
          CSV columns: <code class="rounded bg-canvas px-1 text-xs">short name</code> (required),
          <code class="rounded bg-canvas px-1 text-xs">description</code>,
          <code class="rounded bg-canvas px-1 text-xs">category</code>,
          <code class="rounded bg-canvas px-1 text-xs">assignee</code>,
          <code class="rounded bg-canvas px-1 text-xs">estimate hour</code>,
          <code class="rounded bg-canvas px-1 text-xs">due date</code>,
          <code class="rounded bg-canvas px-1 text-xs">issue key</code>
        </span>
      </div>
      <p v-if="importError" class="mt-3 rounded-md border border-red-200 bg-red-50 p-2 text-sm text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300">
        {{ importError }}
      </p>
    </section>

    <!-- Preview table -->
    <template v-if="parsed.length > 0">
      <section class="min-h-0 flex-1 overflow-auto rounded-lg border border-divider bg-panel shadow-sm">
        <DataTable
          class="app-data-table min-h-0"
          scrollable
          scroll-height="flex"
          :table-style="{ minWidth: '860px' }"
          :value="parsed"
        >
          <Column header="" header-class="w-12" body-class="text-center w-12">
            <template #body="{ data }">
              <Checkbox
                v-model="data._selected"
                :binary="true"
                class="h-4 w-4 accent-brand"
                @update:model-value="updateSelectedCount()"
              />
            </template>
          </Column>
          <Column field="_row" header="Row" header-class="w-16" body-class="font-mono text-xs text-muted w-16" />
          <Column field="shortName" header="Short name" body-class="font-bold text-ink" />
          <Column header="Category">
            <template #body="{ data }">
              <div class="flex flex-wrap gap-1">
                <span
                  v-for="cat in data.categories"
                  :key="cat"
                  class="rounded-md bg-emerald-50 px-2 py-0.5 text-xs font-bold text-brand"
                >{{ cat }}</span>
                <span v-if="!data.categories.length" class="text-muted">-</span>
              </div>
            </template>
          </Column>
          <Column field="assignee" header="Assignee">
            <template #body="{ data }">{{ data.assignee || '-' }}</template>
          </Column>
          <Column header="Estimate" body-class="num" header-class="num">
            <template #body="{ data }">{{ data.estimateHour ? `${data.estimateHour}h` : '-' }}</template>
          </Column>
          <Column field="dueDate" header="Due date">
            <template #body="{ data }">{{ data.dueDate || '-' }}</template>
          </Column>
          <Column field="issueKey" header="Issue">
            <template #body="{ data }">{{ data.issueKey || '-' }}</template>
          </Column>
        </DataTable>
      </section>
    </template>

    <!-- Empty state -->
    <div v-if="parsed.length === 0 && !importError" class="flex flex-1 items-center justify-center rounded-lg border border-dashed border-divider bg-panel/50 p-12">
      <p class="text-sm text-muted">Upload a CSV file to preview and import tasks.</p>
    </div>

    <!-- Backlog import dialog -->
    <Dialog
      v-model:visible="backlogDialogVisible"
      :header="`Backlog - ${backlogProjectName || '...'}`"
      modal
      maximizable
      :style="{ width: '920px' }"
      :content-style="{ padding: '0', display: 'flex', flexDirection: 'column', overflow: 'hidden' }"
    >

      <!-- Loading -->
      <div v-if="backlogLoading" class="flex items-center justify-center p-12">
        <i class="pi pi-spin pi-spinner mr-2 text-lg text-brand" />
        <span class="text-sm text-muted">Loading issues from Backlog...</span>
      </div>

      <template v-else>
        <!-- Error -->
        <div v-if="backlogError && !backlogSearching" class="px-4 pt-4">
          <p class="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300">
            {{ backlogError }}
          </p>
        </div>

        <!-- Search fieldset -->
        <div class="border-b border-divider px-4 py-3">
          <div class="grid gap-3">
            <div class="block min-w-0">
              <span class="text-xs font-bold text-muted">Status</span>
              <div class="mt-1 flex min-w-0 flex-wrap items-center gap-1 rounded-md border border-divider bg-panel p-1 text-sm leading-none">
                <Button
                  :class="[
                    'flex min-w-0 items-center justify-center truncate rounded px-2 py-1 text-xs font-normal transition',
                    blSelectedStatuses.length === 0 && !blNotClosed ? 'bg-brand text-white' : 'hover:bg-canvas',
                  ]"
                  unstyled
                  @click="blSelectedStatuses = []; blNotClosed = false"
                >All</Button>
                <Button
                  v-for="s in blStatusOptions"
                  :key="s.id"
                  :class="[
                    'flex min-w-0 items-center justify-center truncate rounded px-2 py-1 text-xs font-normal transition',
                    !blNotClosed && blSelectedStatuses.includes(s.id) ? 'bg-brand text-white' : 'hover:bg-canvas',
                  ]"
                  unstyled
                  @click="toggleBlStatus(s.id)"
                >{{ s.name }}</Button>
                <Button
                  :class="[
                    'flex min-w-0 items-center justify-center truncate rounded px-2 py-1 text-xs font-normal transition',
                    blNotClosed ? 'bg-brand text-white' : 'hover:bg-canvas',
                  ]"
                  unstyled
                  :disabled="blStatusOptions.length === 0"
                  @click="selectBlNotClosed()"
                >Not Closed</Button>
              </div>
            </div>

            <div class="grid gap-3 md:grid-cols-3">
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Issue Type</span>
                <select
                  v-model="blSelectedIssueType"
                  class="mt-1 h-9 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                >
                  <option value="">All types</option>
                  <option v-for="t in blIssueTypeOptions" :key="t.id" :value="t.name">{{ t.name }}</option>
                </select>
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Category</span>
                <select
                  v-model="blSelectedCategory"
                  class="mt-1 h-9 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                >
                  <option value="">All categories</option>
                  <option v-for="c in blCategoryOptions" :key="c.id" :value="c.name">{{ c.name }}</option>
                </select>
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Assignee</span>
                <select
                  disabled
                  class="mt-1 h-9 w-full cursor-not-allowed rounded-md border border-divider bg-canvas px-3 text-sm text-muted outline-none"
                >
                  <option>{{ blMatchedUser?.name || auth.user?.username || '-' }}</option>
                </select>
              </label>
            </div>

            <div class="flex items-center justify-end">
              <Button
                :icon="backlogSearching ? 'pi pi-spinner pi-spin' : 'pi pi-search'"
                :label="backlogSearching ? 'Searching...' : 'Search'"
                size="small"
                :disabled="backlogSearching"
                @click="searchBacklog()"
              />
            </div>
          </div>
        </div>

        <!-- Issue list -->
        <div v-if="backlogRows.length === 0 && !backlogSearching" class="flex flex-1 items-center justify-center p-12">
          <p class="text-sm text-muted">No matching issues found.</p>
        </div>

        <template v-if="backlogRows.length > 0 || backlogSearching">
          <div class="flex items-center gap-3 border-b border-divider px-4 py-2">
            <Checkbox
              :model-value="backlogRows.length > 0 && backlogSelectedCount === backlogRows.length"
              :binary="true"
              class="h-4 w-4 accent-brand"
              @update:model-value="toggleBacklogAll($event as boolean)"
            />
            <span class="text-sm font-bold text-ink">{{ backlogSelectedCount }} / {{ backlogRows.length }} selected</span>
          </div>

          <div class="min-h-0 flex-1 overflow-auto">
            <DataTable
              class="app-data-table"
              :value="backlogRows"
              :table-style="{ minWidth: '780px' }"
              :loading="backlogSearching"
            >
              <Column header="" header-class="w-12" body-class="text-center w-12">
                <template #body="{ data }">
                  <Checkbox
                    v-model="data._selected"
                    :binary="true"
                    class="h-4 w-4 accent-brand"
                  />
                </template>
              </Column>
              <Column field="issueKey" header="Issue" body-class="font-mono text-xs whitespace-nowrap" header-class="w-28" />
              <Column field="summary" header="Summary" body-class="text-ink" />
              <Column field="issueType" header="Type" header-class="w-24" body-class="whitespace-nowrap text-xs" />
              <Column field="status" header="Status" header-class="w-28">
                <template #body="{ data }">
                  <span class="rounded-md bg-blue-50 px-2 py-0.5 text-xs font-bold text-blue-800 dark:bg-blue-900/30 dark:text-blue-300">{{ data.status }}</span>
                </template>
              </Column>
              <Column header="Est." header-class="w-16 num" body-class="num">
                <template #body="{ data }">{{ data.estimatedHours != null ? `${data.estimatedHours}h` : '-' }}</template>
              </Column>
              <Column field="dueDate" header="Due" header-class="w-24">
                <template #body="{ data }">{{ data.dueDate || '-' }}</template>
              </Column>
            </DataTable>
          </div>
        </template>
      </template>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" outlined @click="backlogDialogVisible = false" />
          <Button
            icon="pi pi-check"
            :label="`Select ${backlogSelectedCount} task${backlogSelectedCount !== 1 ? 's' : ''}`"
            :disabled="backlogSelectedCount === 0"
            @click="confirmBacklogSelection"
          />
        </div>
      </template>
    </Dialog>
  </section>
</template>
