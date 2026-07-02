<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import { useProjects } from "../composables/useProjects";
import { formatHourValue, totalMinutes } from "@/shared/utils/timeMath";
import type { ProjectSummary } from "@/shared/types/statistics";

const router = useRouter();
const ctrl = useProjects();
const page = ref(1);
const pageSize = 10;
const contextMenu = ref<{ project: ProjectSummary; x: number; y: number } | null>(null);

function normalize(value: string) { return value.trim().toLocaleLowerCase(); }

const filteredProjects = computed(() => {
  const projects = ctrl.result.value?.projects ?? [];
  const code = normalize(ctrl.filters.value.code);
  const name = normalize(ctrl.filters.value.name);
  const keyword = normalize(ctrl.filters.value.keyword);
  return projects.filter((p) => {
    const matchesCode = !code || normalize(p.project_code).includes(code);
    const matchesName = !name || normalize(p.project_name).includes(name);
    const matchesKeyword = !keyword || normalize([p.project_code, p.project_name, ...p.phases.flatMap((ph) => [ph.process_code, ph.phase_name, ...ph.details.map((d) => d.work_content)])].join(" ")).includes(keyword);
    return matchesCode && matchesName && matchesKeyword;
  });
});

const pageCount = computed(() => Math.max(1, Math.ceil(filteredProjects.value.length / pageSize)));
const visibleProjects = computed(() => filteredProjects.value.slice((page.value - 1) * pageSize, page.value * pageSize));

watch(() => [ctrl.filters.value.code, ctrl.filters.value.name, ctrl.filters.value.keyword], () => { page.value = 1; });
watch(pageCount, (c) => { page.value = Math.min(page.value, c); });

function closeContextMenu() { contextMenu.value = null; }

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

function bugCount(project: ProjectSummary) {
  return project.phases.reduce((total, phase) => total + phase.details.filter((d) => /\b(bug|defect|issue)\b/i.test(d.work_content)).length, 0);
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <section class="flex items-center justify-end rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
      <button class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90" type="button" @click="router.push('/projects/new')"><i class="pi pi-plus" />Register</button>
    </section>

    <Fieldset class="rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested" legend="Search" toggleable>
      <div class="grid gap-3">
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-slate-500">Project Code</span>
            <input class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Code" type="search" :value="ctrl.filters.value.code" @input="ctrl.filters.value = { ...ctrl.filters.value, code: ($event.target as HTMLInputElement).value }" />
          </label>
          <label>
            <span class="text-xs font-bold text-slate-500">Project Name</span>
            <input class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Name" type="search" :value="ctrl.filters.value.name" @input="ctrl.filters.value = { ...ctrl.filters.value, name: ($event.target as HTMLInputElement).value }" />
          </label>
        </div>
        <label>
          <span class="text-xs font-bold text-slate-500">Keyword Search</span>
          <input class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Code, name, phase, work content" type="search" :value="ctrl.filters.value.keyword" @input="ctrl.filters.value = { ...ctrl.filters.value, keyword: ($event.target as HTMLInputElement).value }" />
        </label>
        <div class="flex items-center justify-end gap-2">
          <button class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90" type="button" @click="ctrl.searchProjects()"><i class="pi pi-search" />Search</button>
          <button class="flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50" type="button" @click="ctrl.resetFilters()"><i class="pi pi-refresh" />Reset</button>
        </div>
      </div>
    </Fieldset>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-stone-200 px-4 py-3">
        <h3 class="font-bold">Project list</h3>
        <span class="text-xs text-slate-500">{{ filteredProjects.length.toLocaleString("en-US") }} projects</span>
      </div>
      <DataTable class="app-data-table min-h-0" :empty-message="!ctrl.result.value ? 'No analysis data yet.' : 'No projects match the search conditions.'" :row-class="() => 'cursor-pointer'" scrollable scroll-height="flex" :table-style="{ minWidth: '980px' }" :value="visibleProjects" @row-click="(e: any) => router.push(`/projects/${encodeURIComponent(e.data.project_code)}`)">
        <Column field="project_code" header="Code" body-class="font-bold text-ink"><template #body="{ data }">{{ data.project_code || '-' }}</template></Column>
        <Column field="project_name" header="Name"><template #body="{ data }">{{ data.project_name || '-' }}</template></Column>
        <Column header="Total hour" body-class="num font-extrabold text-brand" header-class="num"><template #body="{ data }">{{ formatHourValue(totalMinutes(data.totals)) }}</template></Column>
        <Column header="Bug count" body-class="num" header-class="num"><template #body="{ data }">{{ bugCount(data).toLocaleString("en-US") }}</template></Column>
      </DataTable>

      <div v-if="contextMenu" class="fixed z-50 min-w-48 overflow-hidden rounded-md border border-slate-200 bg-white py-1 text-sm shadow-xl" :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }" @click.stop>
        <button class="flex h-10 w-full items-center gap-2 rounded-none px-3 text-left text-sm font-semibold text-slate-700 hover:bg-slate-50" type="button"><i class="pi pi-file" />Manage SKILL.md</button>
      </div>

      <div class="flex items-center justify-between gap-4 border-t border-stone-200 px-4 py-3">
        <span class="text-sm text-slate-500">Page {{ page.toLocaleString("en-US") }} / {{ pageCount.toLocaleString("en-US") }}</span>
        <div class="flex items-center gap-2">
          <button class="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50" :disabled="page <= 1" type="button" @click="page = Math.max(1, page - 1)"><i class="pi pi-chevron-left" /></button>
          <button class="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50" :disabled="page >= pageCount" type="button" @click="page = Math.min(pageCount, page + 1)"><i class="pi pi-chevron-right" /></button>
        </div>
      </div>
    </section>
  </section>
</template>
