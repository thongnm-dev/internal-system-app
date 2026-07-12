<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import { useProjectTasks } from "../composables/useProjectTasks";

const route = useRoute();
const router = useRouter();
const projectCode = (route.params.id as string) || "";

const ctrl = useProjectTasks(projectCode);

function addTask() {
  router.push(`/projects/${encodeURIComponent(projectCode)}/tasks/new`);
}

function viewReport() {
  router.push(`/projects/${encodeURIComponent(projectCode)}/report`);
}

function openBacklog(issueKey: string) {
  if (!issueKey) return;
  router.push({ path: "/issue-backlog", query: { keyword: issueKey } });
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <section class="flex items-center justify-between gap-4 rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="min-w-0">
        <h3 class="truncate font-bold text-ink">Tasks</h3>
        <p class="mt-1 truncate text-sm text-muted">
          {{ projectCode }}<template v-if="ctrl.projectName.value"> - {{ ctrl.projectName.value }}</template>
        </p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <button
          class="flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
          type="button"
          @click="viewReport"
        >
          <i class="pi pi-chart-bar" />Report
        </button>
        <button
          class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
          type="button"
          @click="addTask"
        >
          <i class="pi pi-plus" />Add task
        </button>
      </div>
    </section>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Task list</h3>
        <span class="text-xs text-muted">{{ ctrl.tasks.value.length.toLocaleString("en-US") }} tasks</span>
      </div>
      <DataTable
        class="app-data-table min-h-0"
        empty-message="No tasks yet. Click Add task to create one."
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '980px' }"
        :value="ctrl.tasks.value"
      >
        <Column field="shortName" header="Short name" body-class="font-bold text-ink" />
        <Column header="Category">
          <template #body="{ data }">
            <div class="flex flex-wrap gap-1">
              <span
                v-for="category in data.categories"
                :key="category"
                class="rounded-md bg-emerald-50 px-2 py-0.5 text-xs font-bold text-brand"
              >
                {{ category }}
              </span>
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
        <Column header="Issue">
          <template #body="{ data }">
            <button
              v-if="data.issueKey"
              class="font-bold text-brand hover:underline"
              type="button"
              @click="openBacklog(data.issueKey)"
            >
              {{ data.issueKey }}
            </button>
            <span v-else class="text-muted">-</span>
          </template>
        </Column>
        <Column header="Action" body-class="text-center" header-class="w-20 text-center">
          <template #body="{ data }">
            <button
              class="inline-flex h-8 w-8 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
              type="button"
              title="Delete task"
              @click="ctrl.removeTask(data.id)"
            >
              <i class="pi pi-trash" />
            </button>
          </template>
        </Column>
      </DataTable>
    </section>
  </section>
</template>
