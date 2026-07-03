<script setup lang="ts">
import { useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import {
  assignees,
  bugClasses,
  categories,
  issueTypes,
  projects,
  statusOptions,
  uniqueCreateUsers,
  useIssueBacklog,
  statusTone,
  priorityTone,
  type BacklogSearchCriteria,
} from "../composables/useIssueBacklog";

const router = useRouter();
const ctrl = useIssueBacklog();

function openImport() {
  if (ctrl.criteria.value.project) {
    router.push({ path: "/import-issues", query: { project: ctrl.criteria.value.project } });
  }
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-1 overflow-x-hidden overflow-y-auto">
    <Fieldset
      class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested"
      legend="Search"
      toggleable
    >
      <div class="grid gap-3">
        <div class="grid items-end gap-3 lg:grid-cols-2">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Project</span>
            <select
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-xs text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :value="ctrl.criteria.value.project"
              @change="ctrl.setField('project', ($event.target as HTMLSelectElement).value)"
            >
              <option value="">All projects</option>
              <option v-for="p in projects" :key="p" :value="p">{{ p }}</option>
            </select>
          </label>

          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Status</span>
            <div class="mt-1 grid h-10 min-w-0 grid-cols-6 gap-1 rounded-md border border-divider bg-panel p-1 text-xs leading-none">
              <button
                v-for="s in statusOptions"
                :key="s"
                :class="[
                  'flex min-w-0 items-center justify-center truncate rounded px-1.5 py-0 text-xs font-normal transition',
                  ctrl.criteria.value.status === s ? 'bg-brand text-white' : 'hover:bg-canvas',
                ]"
                type="button"
                @click="ctrl.setField('status', s)"
              >
                {{ s }}
              </button>
            </div>
          </label>
        </div>

        <div class="grid gap-3 lg:grid-cols-2">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Issue Type</span>
            <select
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-xs text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :value="ctrl.criteria.value.issueType"
              @change="ctrl.setField('issueType', ($event.target as HTMLSelectElement).value)"
            >
              <option value="">All types</option>
              <option v-for="t in issueTypes" :key="t" :value="t">{{ t }}</option>
            </select>
          </label>
          <div class="grid gap-3 md:grid-cols-2">
            <label class="block min-w-0">
              <span class="text-xs font-bold text-muted">Category</span>
              <select
                class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-xs text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                :value="ctrl.criteria.value.category"
                @change="ctrl.setField('category', ($event.target as HTMLSelectElement).value)"
              >
                <option value="">All categories</option>
                <option v-for="c in categories" :key="c" :value="c">{{ c }}</option>
              </select>
            </label>
            <label class="block min-w-0">
              <span class="text-xs font-bold text-muted">Assignee</span>
              <select
                class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-xs text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                :value="ctrl.criteria.value.assignee"
                @change="ctrl.setField('assignee', ($event.target as HTMLSelectElement).value)"
              >
                <option value="">All assignees</option>
                <option v-for="a in assignees" :key="a" :value="a">{{ a }}</option>
              </select>
            </label>
          </div>
        </div>

        <div class="grid gap-3 lg:grid-cols-2">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Keyword</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Issue key or subject"
              type="search"
              :value="ctrl.criteria.value.keyword"
              @input="ctrl.setField('keyword', ($event.target as HTMLInputElement).value)"
              @keydown.enter="ctrl.search()"
            />
          </label>
          <div />
        </div>

        <Fieldset
          class="rounded-md border border-divider bg-panel p-3 fieldset-nested"
          legend="Advanced"
          toggleable
        >
          <div class="grid gap-3 lg:grid-cols-2">
            <div class="grid gap-3 md:grid-cols-2">
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Create date from</span>
                <input
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none hover:border-brand focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  type="date"
                  :value="ctrl.criteria.value.createDateFrom"
                  @change="ctrl.setField('createDateFrom', ($event.target as HTMLInputElement).value)"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Create date to</span>
                <input
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none hover:border-brand focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  type="date"
                  :value="ctrl.criteria.value.createDateTo"
                  @change="ctrl.setField('createDateTo', ($event.target as HTMLInputElement).value)"
                />
              </label>
            </div>
            <div class="grid gap-3 md:grid-cols-2">
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Create user</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-xs text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  :value="ctrl.criteria.value.createUser"
                  @change="ctrl.setField('createUser', ($event.target as HTMLSelectElement).value)"
                >
                  <option value="">All users</option>
                  <option v-for="u in uniqueCreateUsers" :key="u" :value="u">{{ u }}</option>
                </select>
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Bug class</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-xs text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  :value="ctrl.criteria.value.bugClass"
                  @change="ctrl.setField('bugClass', ($event.target as HTMLSelectElement).value)"
                >
                  <option value="">All bug classes</option>
                  <option v-for="b in bugClasses" :key="b" :value="b">{{ b }}</option>
                </select>
              </label>
            </div>
          </div>
        </Fieldset>

        <div class="flex items-center justify-end gap-2">
          <button
            class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
            type="button"
            :disabled="!ctrl.canOpenImport.value"
            :title="ctrl.canOpenImport.value ? 'Import issues for selected project' : 'Select a project before importing issues'"
            @click="openImport"
          >
            <i class="pi pi-file-import" />
            Import
          </button>
          <button
            class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
            type="button"
            title="Search"
            @click="ctrl.search()"
          >
            <i class="pi pi-search" />
            Search
          </button>
          <button
            class="flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            type="button"
            title="Reset search conditions"
            @click="ctrl.reset()"
          >
            <i class="pi pi-refresh" />
            Reset
          </button>
        </div>
      </div>
    </Fieldset>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Issue backlog list</h3>
        <span class="text-xs text-muted">{{ ctrl.filteredItems.value.length.toLocaleString("en-US") }} issues</span>
      </div>

      <DataTable
        class="app-data-table min-h-0"
        empty-message="No issues match the search conditions."
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '1180px' }"
        :value="ctrl.filteredItems.value"
      >
        <Column field="issueType" header="Issue Type" body-class="whitespace-nowrap" />
        <Column field="issueKey" header="Issue Key" body-class="whitespace-nowrap font-bold text-ink" />
        <Column field="subject" header="Subject" body-class="max-w-[360px] truncate" />
        <Column field="assignee" header="Assignee" body-class="whitespace-nowrap" />
        <Column header="Status" body-class="whitespace-nowrap">
          <template #body="{ data }">
            <span :class="['rounded px-2 py-1 text-xs font-bold', statusTone(data.status)]">{{ data.status }}</span>
          </template>
        </Column>
        <Column header="Hours" body-class="num" header-class="num">
          <template #body="{ data }">{{ data.hours.toFixed(data.hours % 1 === 0 ? 0 : 1) }}</template>
        </Column>
        <Column header="Priority" body-class="whitespace-nowrap">
          <template #body="{ data }">
            <span :class="['rounded px-2 py-1 text-xs font-bold', priorityTone(data.priority)]">{{ data.priority }}</span>
          </template>
        </Column>
        <Column field="createDate" header="Create Date" body-class="whitespace-nowrap" />
        <Column field="createUser" header="Create User" body-class="whitespace-nowrap" />
      </DataTable>
    </section>
  </section>
</template>
