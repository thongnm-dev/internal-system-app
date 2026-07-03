<script setup lang="ts">
import { ref } from "vue";
import { useRoute } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import MessageBanner from "@/shared/components/MessageBanner.vue";
import { useImportIssues, priorityTone } from "../composables/useImportIssues";

const route = useRoute();
const initialProject = (route.query.project as string) || "";
const ctrl = useImportIssues(initialProject);

const fileInputRef = ref<HTMLInputElement | null>(null);

function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement;
  ctrl.selectFile(input.files?.[0] ?? null);
}

function formatEmpty(value: string) {
  return value || "-";
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="flex items-center gap-2">
        <i class="pi pi-file-import text-xl text-brand" />
        <h3 class="font-bold">Import issues</h3>
      </div>

      <div class="mt-4 grid gap-3 lg:grid-cols-[minmax(220px,320px)_minmax(0,1fr)_auto_auto]">
        <label class="block min-w-0">
          <span class="text-xs font-bold text-muted">Project</span>
          <select
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            :value="ctrl.projectCode.value"
            @change="ctrl.projectCode.value = ($event.target as HTMLSelectElement).value"
          >
            <option value="">Select project</option>
            <option
              v-for="opt in ctrl.projectOptions.value"
              :key="opt.value"
              :value="opt.value"
            >
              {{ opt.label }}
            </option>
          </select>
        </label>

        <label class="block min-w-0">
          <span class="text-xs font-bold text-muted">CSV file</span>
          <div class="mt-1 flex h-10 items-center rounded-md border border-divider bg-panel px-3 text-sm text-secondary">
            <span class="min-w-0 truncate">{{ ctrl.selectedFile.value?.name ?? "Select issues CSV file..." }}</span>
          </div>
        </label>

        <div class="flex items-end">
          <input
            ref="fileInputRef"
            class="hidden"
            type="file"
            accept=".csv,text/csv"
            @change="onFileChange"
          />
          <button
            class="flex h-10 w-full items-center justify-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            type="button"
            title="Browse CSV"
            @click="fileInputRef?.click()"
          >
            <i class="pi pi-folder-open" />
            Browse
          </button>
        </div>

        <div class="flex items-end">
          <button
            class="flex h-10 w-full items-center justify-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
            type="button"
            :disabled="ctrl.isImporting.value"
            @click="ctrl.importCsv()"
          >
            <i class="pi pi-file-import" />
            Import
          </button>
        </div>
      </div>
    </section>

    <MessageBanner :message="ctrl.message.value" :mode="ctrl.messageMode.value" />

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <div class="min-w-0">
          <h3 class="font-bold">Imported issue list</h3>
          <p class="mt-1 truncate text-xs text-muted">
            {{ ctrl.selectedProject.value ? ctrl.selectedProject.value.label : "No project selected" }}
            {{ ctrl.selectedFile.value ? ` / ${ctrl.selectedFile.value.name}` : "" }}
          </p>
        </div>
        <span class="text-xs text-muted">{{ ctrl.rows.value.length.toLocaleString("en-US") }} issues</span>
      </div>

      <DataTable
        class="app-data-table min-h-0"
        empty-message="No imported issues. Select a project and CSV file, then click Import."
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '1680px' }"
        :value="ctrl.rows.value"
      >
        <Column field="subject" header="Subject" body-class="max-w-[280px] truncate font-bold text-ink" />
        <Column field="description" header="Description" body-class="max-w-[320px] truncate" />
        <Column field="issueType" header="Issue Type" body-class="whitespace-nowrap" />
        <Column field="assignee" header="Assignee" body-class="whitespace-nowrap" />
        <Column field="startDate" header="Start Date" body-class="whitespace-nowrap" />
        <Column field="dueDate" header="Due Date" body-class="whitespace-nowrap" />
        <Column header="Estimated Hours" body-class="num" header-class="num">
          <template #body="{ data }">{{ formatEmpty(data.estimatedHours) }}</template>
        </Column>
        <Column header="Actual Hours" body-class="num" header-class="num">
          <template #body="{ data }">{{ formatEmpty(data.actualHours) }}</template>
        </Column>
        <Column field="categories" header="Categories" body-class="max-w-[220px] truncate" />
        <Column field="version" header="Version" body-class="whitespace-nowrap" />
        <Column field="milestones" header="Milestones" body-class="whitespace-nowrap" />
        <Column header="Priority" body-class="whitespace-nowrap">
          <template #body="{ data }">
            <span v-if="data.priority" :class="['rounded px-2 py-1 text-xs font-bold', priorityTone(data.priority)]">{{ data.priority }}</span>
          </template>
        </Column>
        <Column field="parentIssue" header="Parent issue" body-class="whitespace-nowrap" />
        <Column field="bugTypes" header="Bug Types" body-class="whitespace-nowrap" />
        <Column field="bugSeverityLevels" header="Bug severity levels" body-class="whitespace-nowrap" />
        <Column field="testPhase" header="Test Phase" body-class="whitespace-nowrap" />
      </DataTable>
    </section>
  </section>
</template>
