<script setup lang="ts">
import { computed, ref } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import { useImportCsv } from "../composables/useImportCsv";
import MessageBanner from "@/shared/components/MessageBanner.vue";
import { emptyTotals, formatHourValue, totalMinutes } from "@/shared/utils/timeMath";
import type {
  AnalysisResult,
  ImportCsvPreviewResult,
  ProjectSummary,
  SelectedPhaseDetail,
} from "@/shared/types/statistics";

const ctrl = useImportCsv();
const isDetailDialogOpen = ref(false);

type ImportProjectTableRow =
  | { id: string; kind: "project"; project: ProjectSummary; phase: null; rowCount: number; totals: ProjectSummary["totals"] }
  | { id: string; kind: "phase"; project: ProjectSummary; phase: ProjectSummary["phases"][number]; rowCount: number; totals: ProjectSummary["totals"] };

function buildImportSummary(result: ImportCsvPreviewResult): AnalysisResult {
  const projects = new Map<string, ProjectSummary>();
  for (const row of result.preview_rows) {
    const project = projects.get(row.project_code) ?? {
      project_code: row.project_code,
      project_name: row.project_name,
      totals: emptyTotals(),
      row_count: 0,
      phases: [],
    } satisfies ProjectSummary;

    let phase = project.phases.find((p) => p.process_code === row.process_code);
    if (!phase) {
      phase = { process_code: row.process_code, phase_name: row.phase_name, totals: emptyTotals(), row_count: 0, details: [] };
      project.phases.push(phase);
    }
    project.row_count += 1;
    project.totals.regular_minutes += row.total_minutes;
    phase.row_count += 1;
    phase.totals.regular_minutes += row.total_minutes;
    phase.details.push({ date: row.date, work_content: row.work_content, total_minutes: row.total_minutes });
    projects.set(row.project_code, project);
  }
  const sorted = Array.from(projects.values()).sort((a, b) => totalMinutes(b.totals) - totalMinutes(a.totals));
  for (const p of sorted) p.phases.sort((a, b) => totalMinutes(b.totals) - totalMinutes(a.totals));
  return { source_path: result.source_path, row_count: result.row_count, grand_total: { regular_minutes: result.total_minutes, normal_overtime_minutes: 0, legal_holiday_overtime_minutes: 0, legal_public_holiday_overtime_minutes: 0, late_night_overtime_minutes: 0 }, projects: sorted };
}

function buildImportProjectRows(project: ProjectSummary): ImportProjectTableRow[] {
  return [
    { id: `${project.project_code}-all`, kind: "project", project, phase: null, rowCount: project.row_count, totals: project.totals },
    ...project.phases.map((phase) => ({ id: `${project.project_code}-${phase.process_code}`, kind: "phase" as const, project, phase, rowCount: phase.row_count, totals: phase.totals })),
  ];
}

const previewRows = computed(() => {
  if (!ctrl.previewResult.value) return [];
  const summary = buildImportSummary(ctrl.previewResult.value);
  return summary.projects.flatMap(buildImportProjectRows);
});

const hiddenCsvDetailHeaders = new Set(["社員番号", "給与社員番号", "プロジェクトコード（日本）", "固定プロジェクト名", "プロセス"]);

function formatCsvMinuteValue(value: string | undefined): string {
  const minutes = Number((value ?? "").trim().replace(/,/g, ""));
  if (!Number.isFinite(minutes) || minutes === 0) return "-";
  return formatHourValue(minutes);
}

function onOpenDetail(detail: SelectedPhaseDetail) {
  // placeholder for phase detail
  void detail;
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <div class="grid grid-cols-[minmax(0,1fr)_320px] gap-4">
      <div class="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
        <div class="flex items-center gap-2">
          <i class="pi pi-database text-xl text-brand" />
          <h3 class="font-bold">Monthly report CSV import</h3>
        </div>
        <p class="mt-2 text-sm text-slate-600">Upload an exported CSV from the system and save it as check data for monthly report matching.</p>
        <div class="mt-4 grid grid-cols-[1fr_auto_auto] gap-2">
          <input class="h-10 min-w-0 rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Select CSV file..." type="text" :value="ctrl.csvPath.value" @input="ctrl.updateCsvPath(($event.target as HTMLInputElement).value)" />
          <button class="flex h-10 items-center justify-center rounded-md border border-slate-300 bg-white px-4 text-slate-700 hover:bg-slate-50" type="button" title="Browse CSV" @click="ctrl.pickCsvFile()"><i class="pi pi-folder-open" /></button>
          <button class="flex h-10 items-center justify-center gap-2 rounded-md bg-slate-800 px-3 text-sm font-bold text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-60" type="button" :disabled="ctrl.isImporting.value" @click="ctrl.previewCsv()"><i class="pi pi-file-import" />Import</button>
        </div>
      </div>
      <div class="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
        <span class="text-sm font-bold text-slate-500">Current import</span>
        <strong class="mt-2 block text-2xl leading-tight text-slate-900">{{ ctrl.result.value ? ctrl.result.value.row_count.toLocaleString("en-US") : "-" }}</strong>
        <p class="mt-1 text-sm text-slate-500">saved rows</p>
      </div>
    </div>

    <MessageBanner :message="ctrl.message.value" :mode="ctrl.messageMode.value" />

    <div class="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_320px] gap-4 overflow-hidden">
      <!-- Preview -->
      <section class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
        <div class="grid items-start gap-3 border-b border-stone-200 px-4 py-3">
          <div class="min-w-0">
            <h3 class="font-bold">Preview</h3>
            <p class="mt-1 truncate text-xs text-slate-500">
              {{ ctrl.previewResult.value ? `${ctrl.previewResult.value.source_file_name}${ctrl.result.value ? ` - saved batch #${ctrl.result.value.batch_id}` : ''}` : 'No CSV data imported yet.' }}
            </p>
          </div>
          <button class="inline-flex h-9 items-center justify-center gap-2 rounded-md border border-slate-200 bg-white px-3 text-xs font-bold text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50" type="button" :disabled="!ctrl.previewResult.value" title="Show detail list" @click="isDetailDialogOpen = true"><i class="pi pi-list" />Show detail</button>
        </div>
        <DataTable class="app-data-table min-h-0" empty-message="No data. Select a CSV file, then click Import." :row-class="(row: any) => row.kind === 'project' ? 'bg-emerald-50 font-bold' : 'cursor-pointer'" scrollable scroll-height="flex" :table-style="{ minWidth: '920px' }" :value="previewRows" @row-click="(e: any) => { if (e.data.kind === 'phase') onOpenDetail({ project_code: e.data.project.project_code, project_name: e.data.project.project_name, phase: e.data.phase }); }">
          <Column header="Project">
            <template #body="{ data }"><strong v-if="data.kind === 'project'">{{ `${data.project.project_code} ${data.project.project_name}`.trim() }}</strong></template>
          </Column>
          <Column header="Phase">
            <template #body="{ data }">
              <template v-if="data.kind === 'project'">All phases</template>
              <template v-else><span class="mr-2 inline-block min-w-8 rounded bg-blue-100 px-1.5 py-0.5 text-center text-xs font-extrabold text-blue-800">{{ data.phase.process_code }}</span>{{ data.phase.phase_name }}</template>
            </template>
          </Column>
          <Column header="Rows" body-class="num" header-class="num"><template #body="{ data }">{{ data.rowCount.toLocaleString("en-US") }}</template></Column>
          <Column header="Total (hour)" body-class="num font-bold text-brand" header-class="num"><template #body="{ data }">{{ formatHourValue(totalMinutes(data.totals)) }}</template></Column>
        </DataTable>
      </section>

      <!-- Import History -->
      <aside class="min-h-0 overflow-auto rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
        <h3 class="font-bold">Recent imports</h3>
        <div class="mt-4 space-y-3">
          <p v-if="ctrl.batches.value.length === 0" class="text-sm text-slate-500">No import history.</p>
          <div v-for="batch in ctrl.batches.value" :key="batch.id" class="border-b border-stone-100 pb-3 last:border-b-0">
            <div class="flex items-center justify-between gap-3">
              <span class="min-w-0 truncate text-sm font-bold">{{ batch.source_file_name }}</span>
              <span class="text-xs font-bold text-brand">#{{ batch.id }}</span>
            </div>
            <p class="mt-1 text-xs text-slate-500">{{ batch.imported_at }}</p>
            <p class="mt-1 text-xs text-slate-500">{{ batch.row_count.toLocaleString("en-US") }} rows / {{ formatHourValue(batch.total_minutes) }} hours</p>
          </div>
        </div>
      </aside>
    </div>

    <!-- Detail Dialog -->
    <Dialog v-if="ctrl.previewResult.value" :visible="isDetailDialogOpen" class="w-full max-w-6xl overflow-hidden rounded-lg bg-white shadow-2xl" :style="{ maxHeight: '86vh' }" @update:visible="isDetailDialogOpen = $event">
      <template #header>
        <div class="min-w-0">
          <h3 class="truncate text-lg font-bold text-slate-900">CSV detail</h3>
          <p class="mt-1 truncate text-sm text-slate-500">{{ ctrl.previewResult.value.source_file_name }}{{ ctrl.result.value ? ` - saved batch #${ctrl.result.value.batch_id}` : '' }}</p>
        </div>
      </template>
      <DataTable class="app-data-table min-h-0" empty-message="No CSV rows." scrollable scroll-height="flex" :table-style="{ minWidth: `${Math.max(920, (ctrl.previewResult.value?.raw_headers.filter(h => !hiddenCsvDetailHeaders.has(h)).length ?? 0) * 140)}px` }" :value="ctrl.previewResult.value.raw_rows.map((cells, i) => ({ cells, id: i }))">
        <Column v-for="(col, idx) in ctrl.previewResult.value.raw_headers.map((h, i) => ({ header: h, index: i })).filter(c => !hiddenCsvDetailHeaders.has(c.header))" :key="`${col.header}-${col.index}`" :header="col.header" :body-class="ctrl.previewResult.value.minute_column_indexes.includes(col.index) ? 'num' : undefined" :header-class="ctrl.previewResult.value.minute_column_indexes.includes(col.index) ? 'num' : undefined">
          <template #body="{ data }">{{ ctrl.previewResult.value!.minute_column_indexes.includes(col.index) ? formatCsvMinuteValue(data.cells[col.index]) : (data.cells[col.index] || '') }}</template>
        </Column>
      </DataTable>
    </Dialog>
  </section>
</template>
