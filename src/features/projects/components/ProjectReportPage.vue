<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import { useProjectTasks } from "../composables/useProjectTasks";
import { formatHourValue, overtimeMinutes, totalMinutes } from "@/shared/utils/timeMath";

const route = useRoute();
const router = useRouter();
const projectCode = (route.params.id as string) || "";

const ctrl = useProjectTasks(projectCode);

const totalHour = computed(() =>
  ctrl.project.value ? formatHourValue(totalMinutes(ctrl.project.value.totals)) : "-",
);
const overtimeHour = computed(() =>
  ctrl.project.value ? formatHourValue(overtimeMinutes(ctrl.project.value.totals)) : "-",
);
const phaseCount = computed(() => ctrl.project.value?.phases.length ?? 0);

const estimatedHour = computed(() => {
  const sum = ctrl.tasks.value.reduce((total, task) => total + (Number(task.estimateHour) || 0), 0);
  return sum > 0 ? sum.toLocaleString("en-US") : "-";
});

function goBack() {
  router.push(`/projects/${encodeURIComponent(projectCode)}/tasks`);
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-auto">
    <section class="flex items-center justify-between gap-4 rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="min-w-0">
        <h3 class="truncate font-bold text-ink">Report</h3>
        <p class="mt-1 truncate text-sm text-muted">
          {{ projectCode }}<template v-if="ctrl.projectName.value"> - {{ ctrl.projectName.value }}</template>
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

    <p v-if="!ctrl.project.value" class="rounded-lg border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">
      No analysis data found for this project.
    </p>

    <template v-else>
      <!-- Summary cards -->
      <section class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <p class="text-xs font-bold text-muted">Total hour</p>
          <p class="mt-1 text-2xl font-extrabold text-brand">{{ totalHour }}</p>
        </div>
        <div class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <p class="text-xs font-bold text-muted">Overtime hour</p>
          <p class="mt-1 text-2xl font-extrabold text-ink">{{ overtimeHour }}</p>
        </div>
        <div class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <p class="text-xs font-bold text-muted">Phases</p>
          <p class="mt-1 text-2xl font-extrabold text-ink">{{ phaseCount.toLocaleString("en-US") }}</p>
        </div>
        <div class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <p class="text-xs font-bold text-muted">Estimated (tasks)</p>
          <p class="mt-1 text-2xl font-extrabold text-ink">{{ estimatedHour }}</p>
        </div>
      </section>

      <!-- Phase breakdown -->
      <section class="flex flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
        <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
          <h3 class="font-bold">Phase breakdown</h3>
        </div>
        <DataTable
          class="app-data-table"
          empty-message="No phase data."
          :table-style="{ minWidth: '720px' }"
          :value="ctrl.project.value.phases"
        >
          <Column field="process_code" header="Process" body-class="font-bold text-ink" />
          <Column field="phase_name" header="Phase" />
          <Column header="Rows" body-class="num" header-class="num">
            <template #body="{ data }">{{ data.row_count.toLocaleString("en-US") }}</template>
          </Column>
          <Column header="Hour" body-class="num font-extrabold text-brand" header-class="num">
            <template #body="{ data }">{{ formatHourValue(totalMinutes(data.totals)) }}</template>
          </Column>
        </DataTable>
      </section>
    </template>
  </section>
</template>
