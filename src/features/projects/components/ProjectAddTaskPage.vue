<script setup lang="ts">
import { ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Checkbox from "primevue/checkbox";
import {
  useProjectTasks,
  type ProjectTask,
  type ProjectTaskInput,
} from "../composables/useProjectTasks";

const route = useRoute();
const router = useRouter();
const projectId = (route.params.id as string) || "";

const ctrl = useProjectTasks(projectId);

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

function importSelected() {
  const selected = parsed.value.filter((r) => r._selected);
  for (const row of selected) {
    ctrl.addTask({
      shortName: row.shortName,
      description: row.description,
      categories: row.categories,
      assignee: row.assignee,
      estimateHour: row.estimateHour,
      dueDate: row.dueDate,
      issueKey: row.issueKey,
    });
  }
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
      <button
        class="flex h-9 shrink-0 items-center gap-2 rounded-md border border-divider bg-panel px-3 text-sm font-bold text-secondary hover:bg-canvas"
        type="button"
        @click="goBack"
      >
        <i class="pi pi-arrow-left" />Back
      </button>
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
        <button
          class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
          type="button"
          @click="triggerFileInput"
        >
          <i class="pi pi-upload" />
          Choose CSV file
        </button>
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
      <section class="flex items-center justify-between gap-4 rounded-lg border border-divider bg-panel px-4 py-3 shadow-sm">
        <div class="flex items-center gap-3">
          <Checkbox
            :model-value="selectedCount === parsed.length"
            :binary="true"
            class="h-4 w-4 accent-brand"
            @update:model-value="toggleAll($event as boolean)"
          />
          <span class="text-sm font-bold text-ink">{{ selectedCount }} / {{ parsed.length }} selected</span>
        </div>
        <button
          class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
          type="button"
          :disabled="selectedCount === 0"
          @click="importSelected"
        >
          <i class="pi pi-check" />
          Import {{ selectedCount }} task{{ selectedCount !== 1 ? 's' : '' }}
        </button>
      </section>

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
  </section>
</template>
