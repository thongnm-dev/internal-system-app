<script setup lang="ts">
import { computed, ref, watch, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import { useProjectRegistry } from "../composables/useProjectRegistry";
import type { ProjectSummaryResult } from "@/tauri/commands";

const router = useRouter();
const ctrl = useProjectRegistry();
const page = ref(1);
const pageSize = 10;
const contextMenu = ref<{ project: ProjectSummaryResult; x: number; y: number } | null>(null);

const pageCount = computed(() => Math.max(1, Math.ceil(ctrl.filteredProjects.value.length / pageSize)));
const visibleProjects = computed(() => ctrl.filteredProjects.value.slice((page.value - 1) * pageSize, page.value * pageSize));

watch(() => [ctrl.filters.value.code, ctrl.filters.value.name, ctrl.filters.value.keyword], () => { page.value = 1; });
watch(pageCount, (c) => { page.value = Math.min(page.value, c); });

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

async function deleteContextProject() {
  if (!contextMenu.value) return;
  const id = contextMenu.value.project.id;
  contextMenu.value = null;
  await ctrl.removeProject(id);
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
      <button class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90" type="button" @click="router.push('/projects/new')"><i class="pi pi-plus" />Register</button>
    </section>

    <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Search" toggleable>
      <div class="grid gap-3">
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Project Code</span>
            <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Code" type="search" :value="ctrl.filters.value.code" @input="ctrl.filters.value = { ...ctrl.filters.value, code: ($event.target as HTMLInputElement).value }" />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Project Name</span>
            <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Name" type="search" :value="ctrl.filters.value.name" @input="ctrl.filters.value = { ...ctrl.filters.value, name: ($event.target as HTMLInputElement).value }" />
          </label>
        </div>
        <label>
          <span class="text-xs font-bold text-muted">Keyword Search</span>
          <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Code, name, client" type="search" :value="ctrl.filters.value.keyword" @input="ctrl.filters.value = { ...ctrl.filters.value, keyword: ($event.target as HTMLInputElement).value }" />
        </label>
        <div class="flex items-center justify-end gap-2">
          <button class="flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas" type="button" @click="ctrl.resetFilters()"><i class="pi pi-refresh" />Reset</button>
          <button class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90" type="button" @click="ctrl.loadProjects()"><i class="pi pi-refresh" />Reload</button>
        </div>
      </div>
    </Fieldset>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Project list</h3>
        <span class="text-xs text-muted">{{ ctrl.filteredProjects.value.length.toLocaleString("en-US") }} projects</span>
      </div>
      <p v-if="ctrl.loadError.value" class="border-b border-red-200 bg-red-50 px-4 py-2 text-sm text-red-700">{{ ctrl.loadError.value }}</p>
      <DataTable class="app-data-table min-h-0" :empty-message="ctrl.isLoading.value ? 'Loading...' : 'No projects match the search conditions.'" :row-class="() => 'cursor-pointer'" scrollable scroll-height="flex" :table-style="{ minWidth: '980px' }" :value="visibleProjects" @row-click="(e: any) => router.push(`/projects/${e.data.id}`)" @row-contextmenu="openContextMenu">
        <Column field="code" header="Code" body-class="font-bold text-ink"><template #body="{ data }">{{ data.code || '-' }}</template></Column>
        <Column field="name" header="Name"><template #body="{ data }">{{ data.name || '-' }}</template></Column>
        <Column field="client" header="Client"><template #body="{ data }">{{ data.client || '-' }}</template></Column>
        <Column header="Members" body-class="num" header-class="num"><template #body="{ data }">{{ data.member_count.toLocaleString("en-US") }}</template></Column>
        <Column header="Created" body-class="num" header-class="num"><template #body="{ data }">{{ formatDate(data.created_at) }}</template></Column>
      </DataTable>

      <div v-if="contextMenu" class="fixed z-50 min-w-48 overflow-hidden rounded-md border border-divider bg-panel py-1 text-sm shadow-xl" :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }" @click.stop @contextmenu.prevent>
        <button class="flex h-10 w-full items-center gap-2 rounded-none px-3 text-left text-sm font-semibold text-secondary hover:bg-canvas" type="button" @click="goToProjectRoute('')"><i class="pi pi-pencil" />Edit</button>
        <button class="flex h-10 w-full items-center gap-2 rounded-none px-3 text-left text-sm font-semibold text-secondary hover:bg-canvas" type="button" @click="goToProjectRoute('/tasks/new')"><i class="pi pi-plus" />Add Task</button>
        <button class="flex h-10 w-full items-center gap-2 rounded-none px-3 text-left text-sm font-semibold text-secondary hover:bg-canvas" type="button" @click="goToProjectRoute('/tasks')"><i class="pi pi-list" />View Tasks</button>
        <button class="flex h-10 w-full items-center gap-2 rounded-none px-3 text-left text-sm font-semibold text-secondary hover:bg-canvas" type="button" @click="goToProjectRoute('/report')"><i class="pi pi-chart-bar" />View Report</button>
        <div class="my-1 border-t border-divider" />
        <button class="flex h-10 w-full items-center gap-2 rounded-none px-3 text-left text-sm font-semibold text-red-600 hover:bg-red-50" type="button" @click="deleteContextProject"><i class="pi pi-trash" />Delete</button>
      </div>

      <div class="flex items-center justify-between gap-4 border-t border-divider px-4 py-3">
        <span class="text-sm text-muted">Page {{ page.toLocaleString("en-US") }} / {{ pageCount.toLocaleString("en-US") }}</span>
        <div class="flex items-center gap-2">
          <button class="flex h-9 w-9 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-50" :disabled="page <= 1" type="button" @click="page = Math.max(1, page - 1)"><i class="pi pi-chevron-left" /></button>
          <button class="flex h-9 w-9 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-50" :disabled="page >= pageCount" type="button" @click="page = Math.min(pageCount, page + 1)"><i class="pi pi-chevron-right" /></button>
        </div>
      </div>
    </section>
  </section>
</template>
