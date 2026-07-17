<script setup lang="ts">
import { ref, watch, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { useProjectRegistry } from "../composables/useProjectRegistry";
import type { ProjectSummaryResult } from "@/_/types/project";

const router = useRouter();
const ctrl = useProjectRegistry();
const contextMenu = ref<{ project: ProjectSummaryResult; x: number; y: number } | null>(null);

function openContextMenu(event: { originalEvent: Event; data: ProjectSummaryResult }) {
  const mouseEvent = event.originalEvent as MouseEvent;
  mouseEvent.preventDefault();
  contextMenu.value = { project: event.data, x: mouseEvent.clientX, y: mouseEvent.clientY };
}

function closeContextMenu() { contextMenu.value = null; }

function goToProjectRoute(suffix: string) {
  if (!contextMenu.value) return;
  const id = contextMenu.value.project.id;
  contextMenu.value = null;
  router.push(`/projects/${id}${suffix}`);
}

watch(contextMenu, (val) => {
  if (val) {
    window.addEventListener("click", closeContextMenu);
    window.addEventListener("scroll", closeContextMenu, true);
  } else {
    window.removeEventListener("click", closeContextMenu);
    window.removeEventListener("scroll", closeContextMenu, true);
  }
});

onUnmounted(() => {
  window.removeEventListener("click", closeContextMenu);
  window.removeEventListener("scroll", closeContextMenu, true);
});

function formatDate(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? "-"
    : date.toLocaleDateString("en-US", { year: "numeric", month: "2-digit", day: "2-digit" });
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <section class="flex items-center justify-end rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <Button icon="pi pi-plus" label="Register" @click="router.push('/projects/new')" />
    </section>

    <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Search" toggleable>
      <div class="grid gap-3">
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Project Code</span>
            <InputText class="mt-1 w-full" placeholder="Code" :model-value="ctrl.filters.value.code" @update:model-value="ctrl.filters.value = { ...ctrl.filters.value, code: $event as string }" />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Project Name</span>
            <InputText class="mt-1 w-full" placeholder="Name" :model-value="ctrl.filters.value.name" @update:model-value="ctrl.filters.value = { ...ctrl.filters.value, name: $event as string }" />
          </label>
        </div>
        <label>
          <span class="text-xs font-bold text-muted">Keyword Search</span>
          <InputText class="mt-1 w-full" placeholder="Code, name, client" :model-value="ctrl.filters.value.keyword" @update:model-value="ctrl.filters.value = { ...ctrl.filters.value, keyword: $event as string }" />
        </label>
        <div class="flex items-center justify-end gap-2">
          <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined @click="ctrl.resetFilters()" />
          <Button icon="pi pi-search" label="Search" @click="ctrl.loadProjects()" />
        </div>
      </div>
    </Fieldset>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Project list</h3>
        <span class="text-xs text-muted">{{ ctrl.filteredProjects.value.length.toLocaleString("en-US") }} projects</span>
      </div>
      <p v-if="ctrl.loadError.value" class="border-b border-red-200 bg-red-50 px-4 py-2 text-sm text-red-700">{{ ctrl.loadError.value }}</p>
      <DataTable
        class="app-data-table min-h-0"
        :empty-message="ctrl.isLoading.value ? 'Loading...' : 'No projects match the search conditions.'"
        :row-class="() => 'cursor-pointer'"
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '980px' }"
        :value="ctrl.filteredProjects.value"
        paginator
        :rows="20"
        :rows-per-page-options="[20, 50, 100]"
        paginator-template="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        current-page-report-template="Showing {first} to {last} of {totalRecords}"
        @row-click="(e: any) => router.push(`/projects/${e.data.id}`)"
        @row-contextmenu="openContextMenu"
      >
        <Column field="code" header="Code" body-class="font-bold text-ink"><template #body="{ data }">{{ data.code || '-' }}</template></Column>
        <Column field="name" header="Name"><template #body="{ data }">{{ data.name || '-' }}</template></Column>
        <Column field="client" header="Client"><template #body="{ data }">{{ data.client || '-' }}</template></Column>
        <Column header="Members" body-class="num" header-class="num"><template #body="{ data }">{{ data.member_count.toLocaleString("en-US") }}</template></Column>
        <Column header="Created" body-class="num" header-class="num"><template #body="{ data }">{{ formatDate(data.created_at) }}</template></Column>
      </DataTable>

      <div v-if="contextMenu" class="fixed z-50 min-w-48 overflow-hidden rounded-lg border border-divider bg-panel py-1 shadow-xl" :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }" @click.stop @contextmenu.prevent>
        <Button class="ctx-item" unstyled @click="goToProjectRoute('')"><i class="pi pi-pencil" /> Edit</Button>
        <Button class="ctx-item" unstyled @click="goToProjectRoute('/tasks/new')"><i class="pi pi-plus" /> Add Task</Button>
        <Button class="ctx-item" unstyled @click="goToProjectRoute('/tasks')"><i class="pi pi-list" /> View Tasks</Button>
        <Button class="ctx-item" unstyled @click="goToProjectRoute('/report')"><i class="pi pi-chart-bar" /> View Report</Button>
      </div>
    </section>
  </section>
</template>

<style scoped>
.ctx-item {
  @apply flex h-8 w-full items-center gap-2.5 px-3 text-left text-sm text-secondary hover:bg-canvas;
}
</style>
