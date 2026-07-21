<script setup lang="ts">
import { ref, computed } from "vue";
import Button from "primevue/button";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import { open } from "@tauri-apps/plugin-dialog";
import { canUseTauriRuntime, safeInvoke } from "@/tauri/commands/_base";
import { getProjectDetail } from "@/tauri/commands/project";
import {
  backlogGetProjectLookup,
  backlogListIssueTypes,
  backlogListPriorities,
  backlogCreateIssue,
  type BacklogIssueType as BacklogIssueTypeModel,
  type BacklogPriority as BacklogPriorityModel,
} from "@/tauri/commands/backlog";
import { projects } from "../composables/useIssueBacklog";
import { useToast } from "@/shared/composables/useToast";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";

type IssueCsvRow = {
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
};

type ImportRow = IssueCsvRow & { _selected: boolean; _row: number };

const props = defineProps<{
  visible: boolean;
  projectCode: string;
}>();

const emit = defineEmits<{
  "update:visible": [value: boolean];
}>();

const toast = useToast();
const globalLoading = useGlobalLoading();

const importRows = ref<ImportRow[]>([]);
const importError = ref("");
const importing = ref(false);
const importProgress = ref({ done: 0, total: 0 });
const importSelectedCount = computed(() => importRows.value.filter((r) => r._selected).length);

function onShow() {
  importRows.value = [];
  importError.value = "";
}

function close() {
  emit("update:visible", false);
}

async function browseAndParseImportCsv() {
  const path = await open({
    title: "Select issue CSV file",
    filters: [{ name: "CSV", extensions: ["csv"] }],
    multiple: false,
    directory: false,
  });
  if (!path) return;

  importError.value = "";
  importRows.value = [];

  try {
    const parsed = await safeInvoke<IssueCsvRow[]>("parse_issue_csv", { path });
    if (parsed.length === 0) {
      importError.value = "No valid rows found in the CSV file.";
      return;
    }
    importRows.value = parsed
      .filter((r) => r.subject)
      .map((r, i) => ({ ...r, _selected: true, _row: i + 2 }));
  } catch (e) {
    importError.value = String(e);
  }
}

function toggleImportAll(checked: boolean) {
  importRows.value.forEach((r) => (r._selected = checked));
}

async function executeImport() {
  if (!canUseTauriRuntime()) return;
  if (!props.projectCode) { importError.value = "Please select a project first."; return; }

  const selected = importRows.value.filter((r) => r._selected);
  if (!selected.length) return;

  importing.value = true;
  importError.value = "";
  importProgress.value = { done: 0, total: selected.length };

  globalLoading.start();
  try {
    const proj = projects.value.find((p) => p.code === props.projectCode);
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
      close();
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
  <Dialog
    :visible="props.visible"
    header="Import Issues to Backlog"
    modal
    maximizable
    :style="{ width: '960px' }"
    :content-style="{ padding: '0', display: 'flex', flexDirection: 'column', overflow: 'hidden' }"
    @update:visible="emit('update:visible', $event)"
    @show="onShow"
  >
    <div class="border-b border-divider px-4 py-3">
      <div class="flex items-center gap-3">
        <Button icon="pi pi-upload" label="Choose CSV file" size="small" @click="browseAndParseImportCsv" />
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

    <div v-if="importRows.length === 0 && !importError" class="flex flex-1 items-center justify-center p-12">
      <p class="text-sm text-muted">Upload a CSV file to preview issues.</p>
    </div>

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
      <div class="flex items-center justify-end gap-2 pt-4">
        <Button label="Cancel" severity="secondary" outlined @click="close" />
        <Button
          icon="pi pi-file-import"
          :label="`Import ${importSelectedCount} issue${importSelectedCount !== 1 ? 's' : ''}`"
          :disabled="importSelectedCount === 0 || importing"
          @click="executeImport"
        />
      </div>
    </template>
  </Dialog>
</template>
