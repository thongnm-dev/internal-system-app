<script setup lang="ts">
import { computed } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import { formatHourValue, totalMinutes } from "@/shared/utils/timeMath";
import type {
  AnalysisResult,
  MinuteTotals,
  PhaseSummary,
  ProjectSummary,
  SelectedPhaseDetail,
} from "@/shared/types/analysis";

type ProjectTableRow =
  | { id: string; kind: "project"; project: ProjectSummary; phase: null; totals: MinuteTotals }
  | { id: string; kind: "phase"; project: ProjectSummary; phase: PhaseSummary; totals: MinuteTotals };

const props = withDefaults(
  defineProps<{
    result: AnalysisResult | null;
    compact?: boolean;
  }>(),
  { compact: false },
);

const emit = defineEmits<{
  phaseClick: [detail: SelectedPhaseDetail];
}>();

function buildProjectRows(project: ProjectSummary, compact: boolean): ProjectTableRow[] {
  const phases = compact ? project.phases.slice(0, 4) : project.phases;
  return [
    { id: `${project.project_code}-all`, kind: "project", project, phase: null, totals: project.totals },
    ...phases.map((phase) => ({
      id: `${project.project_code}-${phase.process_code}`,
      kind: "phase" as const,
      project,
      phase,
      totals: phase.totals,
    })),
  ];
}

const rows = computed(() => props.result?.projects.flatMap((p) => buildProjectRows(p, props.compact)) ?? []);

function rowClass(row: ProjectTableRow): string {
  return [
    row.kind === "project" ? "bg-emerald-50 font-bold" : "",
    row.kind === "phase" ? "cursor-pointer" : "",
  ].join(" ");
}

function onRowClick(event: { data: ProjectTableRow }) {
  if (event.data.kind === "phase") {
    emit("phaseClick", {
      project_code: event.data.project.project_code,
      project_name: event.data.project.project_name,
      phase: event.data.phase,
    });
  }
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-card">
    <div class="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-4 border-b border-divider px-4 py-3">
      <h3 class="font-bold">Project summary</h3>
      <span class="min-w-0 truncate text-right text-xs text-muted">{{ result?.source_path ?? "" }}</span>
    </div>
    <DataTable
      class="app-data-table min-h-0"
      empty-message="No analysis data yet."
      :row-class="(row: any) => rowClass(row as ProjectTableRow)"
      scrollable
      scroll-height="flex"
      :table-style="{ minWidth: '980px' }"
      :value="rows"
      @row-click="onRowClick"
    >
      <Column header="Project">
        <template #body="{ data }">
          <strong v-if="data.kind === 'project'">
            {{ `${data.project.project_code} ${data.project.project_name}`.trim() }}
          </strong>
        </template>
      </Column>
      <Column header="Phase">
        <template #body="{ data }">
          <template v-if="data.kind === 'project'">All phases</template>
          <template v-else>
            <span class="mr-2 inline-block min-w-8 rounded bg-blue-100 px-1.5 py-0.5 text-center text-xs font-extrabold text-blue-800">
              {{ data.phase.process_code }}
            </span>
            {{ data.phase.process_name }}
          </template>
        </template>
      </Column>
      <Column header="Regular (hour)" body-class="num" header-class="num">
        <template #body="{ data }">{{ formatHourValue(data.totals.regular_minutes) }}</template>
      </Column>
      <Column header="Normal OT (hour)" body-class="num" header-class="num">
        <template #body="{ data }">{{ formatHourValue(data.totals.normal_overtime_minutes) }}</template>
      </Column>
      <Column header="Holiday OT (hour)" body-class="num" header-class="num">
        <template #body="{ data }">{{ formatHourValue(data.totals.legal_holiday_overtime_minutes) }}</template>
      </Column>
      <Column header="Public Holiday OT (hour)" body-class="num" header-class="num">
        <template #body="{ data }">{{ formatHourValue(data.totals.legal_public_holiday_overtime_minutes) }}</template>
      </Column>
      <Column header="Late Night OT (hour)" body-class="num" header-class="num">
        <template #body="{ data }">{{ formatHourValue(data.totals.late_night_overtime_minutes) }}</template>
      </Column>
      <Column header="Total (hour)" body-class="num font-extrabold text-brand" header-class="num">
        <template #body="{ data }">{{ formatHourValue(totalMinutes(data.totals)) }}</template>
      </Column>
    </DataTable>
  </section>
</template>
