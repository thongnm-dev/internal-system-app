<script setup lang="ts">
import { ref, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import Button from "primevue/button";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Checkbox from "primevue/checkbox";
import Fieldset from "primevue/fieldset";
import Calendar from "primevue/calendar";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import {
  assignees,
  priorityOptions,
  categories,
  issueTypes,
  projects,
  statusOptions,
  uniqueCreateUsers,
  useIssueBacklog,
  statusTone,
  priorityTone,
} from "../composables/useIssueBacklog";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { getProjectDetail } from "@/tauri/commands/project";
import {
  backlogGetProjectLookup,
  backlogListIssueTypes,
  backlogListPriorities,
  backlogCreateIssue,
  type BacklogIssueType as BacklogIssueTypeModel,
  type BacklogPriority as BacklogPriorityModel,
} from "@/tauri/commands/backlog";
import { useToast } from "@/shared/composables/useToast";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";

const route = useRoute();
const router = useRouter();
const toast = useToast();
const globalLoading = useGlobalLoading();
const initialProject = (route.query.project as string) || "";
const ctrl = useIssueBacklog(initialProject);

function toggleStatus(s: string) {
  ctrl.setField("notClosed", false);
  const current = ctrl.criteria.value.status;
  const next = current.includes(s) ? current.filter((v) => v !== s) : [...current, s];
  ctrl.setField("status", next);
}

function selectAll() {
  ctrl.setField("notClosed", false);
  ctrl.setField("status", []);
}

function selectNotClosed() {
  ctrl.setField("status", []);
  ctrl.setField("notClosed", true);
}

// --- Import to Backlog dialog ---
type ImportRow = {
  subject: string;
  description: string;
  issueType: string;
  assignee: string;
  startDate: string;
  dueDate: string;
  estimatedHours: string;
  actualHours: string;
  categories: string;
  version: string;
  milestones: string;
  priority: string;
  parentIssue: string;
  _selected: boolean;
  _row: number;
};

const importDialogVisible = ref(false);
const importRows = ref<ImportRow[]>([]);
const importError = ref("");
const importing = ref(false);
const importProgress = ref({ done: 0, total: 0 });
const fileInput = ref<HTMLInputElement | null>(null);
const importSelectedCount = computed(() => importRows.value.filter((r) => r._selected).length);

const CSV_HEADERS = [
  "Subject", "Description", "Issue Type", "Assignee",
  "Start Date", "Due Date", "Estimated Hours", "Actual Hours",
  "Categories", "Version", "Milestones", "Priority",
  "Parent issue",
] as const;

function openImportDialog() {
  importDialogVisible.value = true;
  importRows.value = [];
  importError.value = "";
}

function triggerImportFile() {
  fileInput.value?.click();
}

function handleImportFile(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  importError.value = "";
  importRows.value = [];

  const reader = new FileReader();
  reader.onload = () => {
    try {
      const text = reader.result as string;
      const rows = parseImportCsv(text);
      if (rows.length === 0) {
        importError.value = "No valid rows found in the CSV file.";
        return;
      }
      importRows.value = rows;
    } catch (e) {
      importError.value = String(e);
    }
  };
  reader.onerror = () => { importError.value = "Failed to read the file."; };
  reader.readAsText(file);
  if (fileInput.value) fileInput.value.value = "";
}

function parseImportCsv(text: string): ImportRow[] {
  const records = parseCsvRecords(text);
  const [headers, ...dataRows] = records.filter((r) => r.some((c) => c.trim()));
  if (!headers) return [];

  const normalized = headers.map((h) => h.trim());
  const hasHeaders = CSV_HEADERS.every((h, i) => normalized[i] === h);
  if (!hasHeaders) {
    throw new Error(`CSV headers do not match the Backlog import template. Expected: ${CSV_HEADERS.join(", ")}`);
  }

  return dataRows.map((cols, i) => ({
    subject: (cols[0] ?? "").trim(),
    description: (cols[1] ?? "").trim(),
    issueType: (cols[2] ?? "").trim(),
    assignee: (cols[3] ?? "").trim(),
    startDate: (cols[4] ?? "").trim(),
    dueDate: (cols[5] ?? "").trim(),
    estimatedHours: (cols[6] ?? "").trim(),
    actualHours: (cols[7] ?? "").trim(),
    categories: (cols[8] ?? "").trim(),
    version: (cols[9] ?? "").trim(),
    milestones: (cols[10] ?? "").trim(),
    priority: (cols[11] ?? "").trim(),
    parentIssue: (cols[12] ?? "").trim(),
    _selected: true,
    _row: i + 2,
  })).filter((r) => r.subject);
}

function parseCsvRecords(content: string): string[][] {
  const records: string[][] = [];
  let cell = "";
  let record: string[] = [];
  let isQuoted = false;

  for (let i = 0; i < content.length; i++) {
    const ch = content[i];
    const next = content[i + 1];
    if (ch === '"') {
      if (isQuoted && next === '"') { cell += '"'; i++; }
      else { isQuoted = !isQuoted; }
      continue;
    }
    if (ch === "," && !isQuoted) { record.push(cell); cell = ""; continue; }
    if ((ch === "\n" || ch === "\r") && !isQuoted) {
      if (ch === "\r" && next === "\n") i++;
      record.push(cell); records.push(record); record = []; cell = "";
      continue;
    }
    cell += ch;
  }
  record.push(cell);
  records.push(record);
  return records;
}

function toggleImportAll(checked: boolean) {
  importRows.value.forEach((r) => (r._selected = checked));
}

async function executeImport() {
  if (!canUseTauriRuntime()) return;
  const projectCode = ctrl.criteria.value.project;
  if (!projectCode) { importError.value = "Please select a project first."; return; }

  const selected = importRows.value.filter((r) => r._selected);
  if (!selected.length) return;

  importing.value = true;
  importError.value = "";
  importProgress.value = { done: 0, total: selected.length };

  globalLoading.start();
  try {
    const proj = projects.value.find((p) => p.code === projectCode);
    if (!proj) throw new Error("Project not found.");

    const detail = await getProjectDetail(proj.id);
    const backlogKey = detail.backlog_key;
    if (!backlogKey) throw new Error("Project has no Backlog key configured.");

    const [lookup, typesList, prioList] = await Promise.all([
      backlogGetProjectLookup(backlogKey),
      backlogListIssueTypes(backlogKey),
      backlogListPriorities(),
    ]);

    const projectId = Number(lookup.projectId);
    const typeMap = new Map(typesList.map((t: BacklogIssueTypeModel) => [t.name, t.id]));
    const prioMap = new Map(prioList.map((p: BacklogPriorityModel) => [p.name, p.id]));

    const defaultTypeId = typesList[0]?.id;
    const defaultPrioId = prioList.find((p: BacklogPriorityModel) => p.name === "Normal")?.id ?? prioList[0]?.id;
    if (!defaultTypeId || !defaultPrioId) throw new Error("Could not resolve default issue type or priority.");

    let successCount = 0;
    const errors: string[] = [];

    for (const row of selected) {
      try {
        await backlogCreateIssue({
          projectId,
          summary: row.subject,
          issueTypeId: typeMap.get(row.issueType) ?? defaultTypeId,
          priorityId: prioMap.get(row.priority) ?? defaultPrioId,
          description: row.description || undefined,
          startDate: row.startDate || undefined,
          dueDate: row.dueDate || undefined,
          estimatedHours: row.estimatedHours ? Number(row.estimatedHours) : undefined,
          actualHours: row.actualHours ? Number(row.actualHours) : undefined,
        });
        successCount++;
      } catch (e) {
        errors.push(`Row ${row._row} (${row.subject}): ${e}`);
      }
      importProgress.value = { done: importProgress.value.done + 1, total: selected.length };
    }

    if (errors.length > 0) {
      importError.value = `${successCount} created, ${errors.length} failed:\n${errors.slice(0, 5).join("\n")}`;
      toast.info(`Imported ${successCount}/${selected.length} issues.`);
    } else {
      toast.success(`Imported ${successCount} issue${successCount !== 1 ? "s" : ""} to Backlog successfully.`);
      importDialogVisible.value = false;
    }
  } catch (e) {
    importError.value = String(e);
  } finally {
    importing.value = false;
    globalLoading.stop();
  }
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-1 overflow-x-hidden overflow-y-auto">
    <Fieldset
      class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested"
      legend="Search"
      toggleable
    >
      <div class="grid gap-3">
        <div class="grid items-end gap-3 lg:grid-cols-2">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Project</span>
            <select
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :value="ctrl.criteria.value.project"
              @change="ctrl.setField('project', ($event.target as HTMLSelectElement).value)"
            >
              <option value="">All projects</option>
              <option v-for="p in projects" :key="p.code" :value="p.code">{{ p.name }}</option>
            </select>
          </label>

          <div class="block min-w-0">
            <span class="text-xs font-bold text-muted">Status</span>
            <div class="mt-1 flex h-10 min-w-0 flex-wrap items-center gap-1 rounded-md border border-divider bg-panel p-1 text-sm leading-none">
              <Button
                :class="[
                  'flex min-w-0 items-center justify-center truncate rounded px-2 py-1 text-sm font-normal transition',
                  ctrl.criteria.value.status.length === 0 && !ctrl.criteria.value.notClosed ? 'bg-brand text-white' : 'hover:bg-canvas',
                ]"
                unstyled
                :disabled="ctrl.lookupLoading.value"
                @click="selectAll()"
              >
                All
              </Button>
              <Button
                v-for="s in statusOptions"
                :key="s"
                :class="[
                  'flex min-w-0 items-center justify-center truncate rounded px-2 py-1 text-sm font-normal transition',
                  !ctrl.criteria.value.notClosed && ctrl.criteria.value.status.includes(s) ? 'bg-brand text-white' : 'hover:bg-canvas',
                ]"
                unstyled
                :disabled="ctrl.lookupLoading.value"
                @click="toggleStatus(s)"
              >
                {{ s }}
              </Button>
              <Button
                :class="[
                  'flex min-w-0 items-center justify-center truncate rounded px-2 py-1 text-sm font-normal transition',
                  ctrl.criteria.value.notClosed ? 'bg-brand text-white' : 'hover:bg-canvas',
                ]"
                unstyled
                :disabled="ctrl.lookupLoading.value || statusOptions.length === 0"
                @click="selectNotClosed()"
              >
                Not Closed
              </Button>
            </div>
          </div>
        </div>

        <div class="grid gap-3 lg:grid-cols-2">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Issue Type</span>
            <select
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :disabled="ctrl.lookupLoading.value"
              :value="ctrl.criteria.value.issueType"
              @change="ctrl.setField('issueType', ($event.target as HTMLSelectElement).value)"
            >
              <option value="">All types</option>
              <option v-for="t in issueTypes" :key="t" :value="t">{{ t }}</option>
            </select>
          </label>
          <div class="grid gap-3 md:grid-cols-2">
            <label class="block min-w-0">
              <span class="text-xs font-bold text-muted">Category</span>
              <select
                class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                :disabled="ctrl.lookupLoading.value"
                :value="ctrl.criteria.value.category"
                @change="ctrl.setField('category', ($event.target as HTMLSelectElement).value)"
              >
                <option value="">All categories</option>
                <option v-for="c in categories" :key="c" :value="c">{{ c }}</option>
              </select>
            </label>
            <label class="block min-w-0">
              <span class="text-xs font-bold text-muted">Assignee</span>
              <select
                class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                :value="ctrl.criteria.value.assignee"
                @change="ctrl.setField('assignee', ($event.target as HTMLSelectElement).value)"
              >
                <option value="">All assignees</option>
                <option v-for="a in assignees" :key="a" :value="a">{{ a }}</option>
              </select>
            </label>
          </div>
        </div>

        <div class="grid gap-3 lg:grid-cols-2">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Keyword</span>
            <InputText
              class="mt-1 w-full"
              placeholder="Issue key or subject"
              :model-value="ctrl.criteria.value.keyword"
              @update:model-value="ctrl.setField('keyword', $event as string)"
              @keydown.enter="ctrl.search()"
            />
          </label>
          <div />
        </div>

        <Fieldset
          class="rounded-md border border-divider bg-panel p-3 fieldset-nested"
          legend="Advanced"
          toggleable
        >
          <div class="grid gap-3 lg:grid-cols-2">
            <div class="grid gap-3 md:grid-cols-2">
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Create date from</span>
                <Calendar
                  :model-value="ctrl.criteria.value.createDateFrom ? new Date(ctrl.criteria.value.createDateFrom + 'T00:00:00') : null"
                  class="mt-1 w-full"
                  date-format="yy/mm/dd"
                  placeholder="Select date"
                  show-icon
                  show-button-bar
                  @update:model-value="ctrl.setField('createDateFrom', $event ? `${$event.getFullYear()}-${String($event.getMonth() + 1).padStart(2, '0')}-${String($event.getDate()).padStart(2, '0')}` : '')"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Create date to</span>
                <Calendar
                  :model-value="ctrl.criteria.value.createDateTo ? new Date(ctrl.criteria.value.createDateTo + 'T00:00:00') : null"
                  class="mt-1 w-full"
                  date-format="yy/mm/dd"
                  placeholder="Select date"
                  show-icon
                  show-button-bar
                  @update:model-value="ctrl.setField('createDateTo', $event ? `${$event.getFullYear()}-${String($event.getMonth() + 1).padStart(2, '0')}-${String($event.getDate()).padStart(2, '0')}` : '')"
                />
              </label>
            </div>
            <div class="grid gap-3 md:grid-cols-2">
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Create user</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  :value="ctrl.criteria.value.createUser"
                  @change="ctrl.setField('createUser', ($event.target as HTMLSelectElement).value)"
                >
                  <option value="">All users</option>
                  <option v-for="u in uniqueCreateUsers" :key="u" :value="u">{{ u }}</option>
                </select>
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Priority</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  :value="ctrl.criteria.value.priorityFilter"
                  @change="ctrl.setField('priorityFilter', ($event.target as HTMLSelectElement).value)"
                >
                  <option value="">All priorities</option>
                  <option v-for="p in priorityOptions" :key="p" :value="p">{{ p }}</option>
                </select>
              </label>
            </div>
          </div>
        </Fieldset>

        <div class="flex items-center justify-end gap-2">
          <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined title="Reset search conditions" @click="ctrl.reset()" />
          <Button
            icon="pi pi-file-import"
            label="Import"
            :disabled="!ctrl.canOpenImport.value"
            :title="ctrl.canOpenImport.value ? 'Import issues to Backlog' : 'Select a project before importing'"
            @click="openImportDialog"
          />
          <Button icon="pi pi-search" label="Search" :disabled="!ctrl.criteria.value.project" title="Search" @click="ctrl.search()" />
        </div>
      </div>
    </Fieldset>

    <div v-if="ctrl.searchError.value" class="rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-700">
      {{ ctrl.searchError.value }}
    </div>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Issue backlog list</h3>
        <span class="text-xs text-muted">{{ ctrl.totalCount.value.toLocaleString("en-US") }} issues</span>
      </div>

      <DataTable
        class="app-data-table min-h-0"
        empty-message="No issues match the search conditions."
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '1180px' }"
        :value="ctrl.filteredItems.value"
        lazy
        paginator
        :first="ctrl.first.value"
        :rows="ctrl.pageSize.value"
        :total-records="ctrl.totalCount.value"
        :rows-per-page-options="[20, 50, 100]"
        paginator-template="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        current-page-report-template="Showing {first} to {last} of {totalRecords}"
        @page="ctrl.onPage($event)"
      >
        <Column field="issueType" header="Issue Type" body-class="whitespace-nowrap" />
        <Column field="issueKey" header="Issue Key" body-class="whitespace-nowrap font-bold text-ink" />
        <Column field="subject" header="Subject" body-class="max-w-[360px] truncate" />
        <Column field="assignee" header="Assignee" body-class="whitespace-nowrap" />
        <Column header="Status" body-class="whitespace-nowrap">
          <template #body="{ data }">
            <span :class="['rounded px-2 py-1 text-xs font-bold', statusTone(data.status)]">{{ data.status }}</span>
          </template>
        </Column>
        <Column header="Hours" body-class="num" header-class="num">
          <template #body="{ data }">{{ data.hours.toFixed(data.hours % 1 === 0 ? 0 : 1) }}</template>
        </Column>
        <Column header="Priority" body-class="whitespace-nowrap">
          <template #body="{ data }">
            <span :class="['rounded px-2 py-1 text-xs font-bold', priorityTone(data.priority)]">{{ data.priority }}</span>
          </template>
        </Column>
        <Column field="createDate" header="Create Date" body-class="whitespace-nowrap" />
        <Column field="createUser" header="Create User" body-class="whitespace-nowrap" />
      </DataTable>
    </section>

    <!-- Import to Backlog dialog -->
    <Dialog
      v-model:visible="importDialogVisible"
      header="Import Issues to Backlog"
      modal
      maximizable
      :style="{ width: '960px' }"
      :content-style="{ padding: '0', display: 'flex', flexDirection: 'column', overflow: 'hidden' }"
    >
      <!-- File upload -->
      <div class="border-b border-divider px-4 py-3">
        <div class="flex items-center gap-3">
          <input
            ref="fileInput"
            type="file"
            accept=".csv"
            class="hidden"
            @change="handleImportFile"
          />
          <Button icon="pi pi-upload" label="Choose CSV file" size="small" @click="triggerImportFile" />
          <span class="text-xs text-muted">
            Backlog CSV format: <code class="rounded bg-canvas px-1">Subject</code>,
            <code class="rounded bg-canvas px-1">Description</code>,
            <code class="rounded bg-canvas px-1">Issue Type</code>,
            <code class="rounded bg-canvas px-1">Priority</code>, ...
          </span>
        </div>
        <p v-if="importError" class="mt-3 whitespace-pre-line rounded-md border border-red-200 bg-red-50 p-2 text-sm text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300">
          {{ importError }}
        </p>
        <div v-if="importing" class="mt-3">
          <div class="flex items-center gap-2 text-sm text-muted">
            <i class="pi pi-spin pi-spinner text-brand" />
            Importing {{ importProgress.done }} / {{ importProgress.total }}...
          </div>
          <div class="mt-1 h-1.5 w-full overflow-hidden rounded-full bg-canvas">
            <div class="h-full rounded-full bg-brand transition-all" :style="{ width: `${importProgress.total ? (importProgress.done / importProgress.total) * 100 : 0}%` }" />
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="importRows.length === 0 && !importError" class="flex flex-1 items-center justify-center p-12">
        <p class="text-sm text-muted">Upload a CSV file to preview issues.</p>
      </div>

      <!-- Preview table -->
      <template v-if="importRows.length > 0">
        <div class="flex items-center gap-3 border-b border-divider px-4 py-2">
          <Checkbox
            :model-value="importRows.length > 0 && importSelectedCount === importRows.length"
            :binary="true"
            class="h-4 w-4 accent-brand"
            @update:model-value="toggleImportAll($event as boolean)"
          />
          <span class="text-sm font-bold text-ink">{{ importSelectedCount }} / {{ importRows.length }} selected</span>
        </div>

        <div class="min-h-0 flex-1 overflow-auto">
          <DataTable
            class="app-data-table"
            :value="importRows"
            :table-style="{ minWidth: '900px' }"
          >
            <Column header="" header-class="w-12" body-class="text-center w-12">
              <template #body="{ data }">
                <Checkbox v-model="data._selected" :binary="true" class="h-4 w-4 accent-brand" />
              </template>
            </Column>
            <Column field="_row" header="Row" header-class="w-14" body-class="font-mono text-xs text-muted w-14" />
            <Column field="subject" header="Subject" body-class="font-bold text-ink max-w-[260px] truncate" />
            <Column field="issueType" header="Type" header-class="w-24" body-class="whitespace-nowrap text-xs" />
            <Column field="assignee" header="Assignee" header-class="w-28" body-class="whitespace-nowrap" />
            <Column field="priority" header="Priority" header-class="w-20" body-class="whitespace-nowrap text-xs" />
            <Column header="Est." header-class="w-16 num" body-class="num">
              <template #body="{ data }">{{ data.estimatedHours || '-' }}</template>
            </Column>
            <Column field="startDate" header="Start" header-class="w-24" body-class="whitespace-nowrap text-xs" />
            <Column field="dueDate" header="Due" header-class="w-24" body-class="whitespace-nowrap text-xs" />
          </DataTable>
        </div>
      </template>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" outlined @click="importDialogVisible = false" />
          <Button
            icon="pi pi-file-import"
            :label="`Import ${importSelectedCount} issue${importSelectedCount !== 1 ? 's' : ''}`"
            :disabled="importSelectedCount === 0 || importing"
            @click="executeImport"
          />
        </div>
      </template>
    </Dialog>
  </section>
</template>
