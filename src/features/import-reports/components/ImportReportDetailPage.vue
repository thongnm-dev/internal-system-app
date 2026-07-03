<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import { useImportReportDetail } from "../composables/useImportReports";
import { emptyTotals, formatHourValue, totalMinutes } from "@/shared/utils/timeMath";
import type { AnalysisResult, ProjectSummary, SelectedPhaseDetail } from "@/shared/types/analysis";
import type { ImportReportDetail } from "@/shared/types/import-report";

const route = useRoute();
const router = useRouter();
const reportId = Number(route.params.id) || null;
const ctrl = useImportReportDetail(reportId);

type ImportProjectTableRow =
  | { id: string; kind: "project"; project: ProjectSummary; phase: null; rowCount: number; totals: ProjectSummary["totals"] }
  | { id: string; kind: "phase"; project: ProjectSummary; phase: ProjectSummary["phases"][number]; rowCount: number; totals: ProjectSummary["totals"] };

function buildImportSummary(detail: ImportReportDetail): AnalysisResult {
  const projects = new Map<string, ProjectSummary>();
  for (const row of detail.preview_rows) {
    const project = projects.get(row.project_code) ?? { project_code: row.project_code, project_name: row.project_name, totals: emptyTotals(), row_count: 0, phases: [] } satisfies ProjectSummary;
    let phase = project.phases.find((p) => p.process_code === row.process_code);
    if (!phase) { phase = { process_code: row.process_code, phase_name: row.phase_name, totals: emptyTotals(), row_count: 0, details: [] }; project.phases.push(phase); }
    project.row_count += 1; project.totals.regular_minutes += row.total_minutes;
    phase.row_count += 1; phase.totals.regular_minutes += row.total_minutes;
    phase.details.push({ date: row.date, work_content: row.work_content, total_minutes: row.total_minutes });
    projects.set(row.project_code, project);
  }
  const sorted = Array.from(projects.values()).sort((a, b) => totalMinutes(b.totals) - totalMinutes(a.totals));
  for (const p of sorted) p.phases.sort((a, b) => totalMinutes(b.totals) - totalMinutes(a.totals));
  return { source_path: detail.source_path, row_count: detail.row_count, grand_total: { regular_minutes: detail.total_minutes, normal_overtime_minutes: 0, legal_holiday_overtime_minutes: 0, legal_public_holiday_overtime_minutes: 0, late_night_overtime_minutes: 0 }, projects: sorted };
}

function buildRows(project: ProjectSummary): ImportProjectTableRow[] {
  return [
    { id: `${project.project_code}-all`, kind: "project", project, phase: null, rowCount: project.row_count, totals: project.totals },
    ...project.phases.map((phase) => ({ id: `${project.project_code}-${phase.process_code}`, kind: "phase" as const, project, phase, rowCount: phase.row_count, totals: phase.totals })),
  ];
}

const rows = computed(() => {
  if (!ctrl.detail.value) return [];
  return buildImportSummary(ctrl.detail.value).projects.flatMap(buildRows);
});

function formatTargetMonthRange(detail: ImportReportDetail) {
  if (!detail.target_month_from && !detail.target_month_to) return "No target month";
  if (detail.target_month_from === detail.target_month_to || !detail.target_month_to) return detail.target_month_from;
  if (!detail.target_month_from) return detail.target_month_to;
  return `${detail.target_month_from} - ${detail.target_month_to}`;
}

function onOpenDetail(detail: SelectedPhaseDetail) { void detail; }
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <div class="flex items-center justify-between gap-3">
      <button class="inline-flex h-10 items-center justify-center gap-2 rounded-md border border-divider bg-panel px-3 text-sm font-bold text-secondary hover:bg-canvas" type="button" @click="router.push('/import-reports')"><i class="pi pi-arrow-left" />Back</button>
    </div>

    <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="border-b border-divider pb-3"><h3 class="font-bold">Thong tin co ban</h3></div>
      <p v-if="!ctrl.detail.value" class="py-8 text-center text-sm text-muted">{{ ctrl.isLoading.value ? 'Loading report detail...' : 'No report detail.' }}</p>
      <div v-else class="mt-4 grid grid-cols-4 gap-3 text-sm">
        <div v-for="item in [
          { label: 'SEQ', value: `#${ctrl.detail.value.id}` },
          { label: 'Ten report', value: ctrl.detail.value.report_name || '-' },
          { label: 'Ten file da import', value: ctrl.detail.value.source_file_name || '-' },
          { label: 'Ngay gio import', value: ctrl.detail.value.imported_at || '-' },
          { label: 'Nguoi import', value: ctrl.detail.value.imported_by || '-' },
          { label: 'Thang target', value: formatTargetMonthRange(ctrl.detail.value) },
          { label: 'Rows', value: ctrl.detail.value.row_count.toLocaleString('en-US') },
          { label: 'Hours', value: formatHourValue(ctrl.detail.value.total_minutes) },
        ]" :key="item.label" class="min-w-0 rounded-md border border-divider bg-canvas px-3 py-2">
          <span class="block text-xs font-bold text-muted">{{ item.label }}</span>
          <strong class="mt-1 block truncate text-ink" :title="item.value">{{ item.value }}</strong>
        </div>
        <div class="col-span-2 min-w-0 rounded-md border border-divider bg-canvas px-3 py-2">
          <span class="block text-xs font-bold text-muted">Ghi chu</span>
          <strong class="mt-1 block truncate text-ink">{{ ctrl.detail.value.note || '-' }}</strong>
        </div>
        <div class="col-span-2 min-w-0 rounded-md border border-divider bg-canvas px-3 py-2">
          <span class="block text-xs font-bold text-muted">Source path</span>
          <strong class="mt-1 block truncate text-ink">{{ ctrl.detail.value.source_path || '-' }}</strong>
        </div>
      </div>
    </section>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="border-b border-divider px-4 py-3">
        <h3 class="font-bold">Danh sach chi tiet cua report</h3>
        <p class="mt-1 truncate text-xs text-muted">{{ ctrl.detail.value ? `${ctrl.detail.value.source_file_name} - saved batch #${ctrl.detail.value.id}` : 'No report data.' }}</p>
      </div>
      <DataTable class="app-data-table min-h-0" :empty-message="ctrl.isLoading.value ? 'Loading report rows...' : 'No report rows.'" :row-class="(row: any) => row.kind === 'project' ? 'bg-emerald-50 font-bold' : 'cursor-pointer'" scrollable scroll-height="flex" :table-style="{ minWidth: '920px' }" :value="rows" @row-click="(e: any) => { if (e.data.kind === 'phase') onOpenDetail({ project_code: e.data.project.project_code, project_name: e.data.project.project_name, phase: e.data.phase }); }">
        <Column header="Project"><template #body="{ data }"><strong v-if="data.kind === 'project'">{{ `${data.project.project_code} ${data.project.project_name}`.trim() }}</strong></template></Column>
        <Column header="Phase"><template #body="{ data }"><template v-if="data.kind === 'project'">All phases</template><template v-else><span class="mr-2 inline-block min-w-8 rounded bg-blue-100 px-1.5 py-0.5 text-center text-xs font-extrabold text-blue-800">{{ data.phase.process_code }}</span>{{ data.phase.phase_name }}</template></template></Column>
        <Column header="Rows" body-class="num" header-class="num"><template #body="{ data }">{{ data.rowCount.toLocaleString("en-US") }}</template></Column>
        <Column header="Total (hour)" body-class="num font-bold text-brand" header-class="num"><template #body="{ data }">{{ formatHourValue(totalMinutes(data.totals)) }}</template></Column>
      </DataTable>
    </section>
  </section>
</template>
