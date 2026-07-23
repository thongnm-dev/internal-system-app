<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";
import Column from "primevue/column";
import DataTable from "primevue/datatable";
import Fieldset from "primevue/fieldset";
import InputText from "primevue/inputtext";
import { useAiTasks } from "../composables/useAiTasks";
import { TASK_CATEGORY_META } from "@/_/types/ai-task";
import type { AiTaskCategory, AiTaskResult } from "@/tauri/commands/ai-task";
import AiTaskDialog from "./AiTaskDialog.vue";
import type { TaskDialogPayload } from "./AiTaskDialog.vue";

const ctrl = useAiTasks();

const showTaskDialog = ref(false);
const dialogTask = ref<AiTaskResult | null>(null);
const dialogLoading = ref(false);

function openCreateDialog() {
  dialogTask.value = null;
  showTaskDialog.value = true;
}

function openEditDialog(task: AiTaskResult) {
  dialogTask.value = task;
  showTaskDialog.value = true;
}

async function handleDialogSave(payload: TaskDialogPayload) {
  dialogLoading.value = true;
  try {
    if (dialogTask.value) {
      await ctrl.saveTask(dialogTask.value.id, payload);
    } else {
      await ctrl.createTask(payload);
    }
    showTaskDialog.value = false;
  } finally {
    dialogLoading.value = false;
  }
}

function categoryLabel(cat: string): string {
  return TASK_CATEGORY_META[cat as AiTaskCategory]?.label ?? cat;
}

function categoryBadgeClass(cat: string): string {
  return TASK_CATEGORY_META[cat as AiTaskCategory]?.badgeClass ?? "bg-canvas text-muted";
}

function formatDate(value: string): string {
  if (!value) return "—";
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? "—"
    : date.toLocaleDateString("en-US", { year: "numeric", month: "2-digit", day: "2-digit" });
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Action bar -->
    <section class="flex items-center justify-end rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <Button icon="pi pi-plus" label="New Task" @click="openCreateDialog" />
    </section>

    <!-- Search fieldset -->
    <Fieldset class="fieldset-nested rounded-lg border border-divider bg-panel p-4 shadow-md" legend="Search" toggleable>
      <div class="grid gap-3">
        <label>
          <span class="text-xs font-bold text-muted">Keyword Search</span>
          <InputText
            v-model="ctrl.filters.value.keyword"
            class="mt-1 w-full"
            placeholder="Task code, task name"
            @keydown.enter="ctrl.search()"
          />
        </label>
        <div class="flex items-center justify-end gap-2">
          <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined @click="ctrl.resetFilters(); ctrl.search()" />
          <Button icon="pi pi-search" label="Search" @click="ctrl.search()" />
        </div>
      </div>
    </Fieldset>

    <!-- Results table -->
    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Task list</h3>
        <span class="text-xs text-muted">{{ ctrl.tasks.value.length.toLocaleString("en-US") }} tasks</span>
      </div>
      <p v-if="ctrl.loadError.value" class="border-b border-red-200 bg-red-50 px-4 py-2 text-sm text-red-700">{{ ctrl.loadError.value }}</p>
      <DataTable
        class="app-data-table min-h-0"
        :empty-message="ctrl.isLoading.value ? 'Loading...' : 'No tasks match the search conditions.'"
        :row-class="() => 'cursor-pointer'"
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '780px' }"
        :value="ctrl.tasks.value"
        paginator
        :rows="20"
        :rows-per-page-options="[20, 50, 100]"
        paginator-template="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        current-page-report-template="Showing {first} to {last} of {totalRecords}"
        @row-click="(e: any) => openEditDialog(e.data)"
      >
        <Column field="task_cd" header="Task Code" body-class="font-bold text-ink" />
        <Column field="task_name" header="Task Name">
          <template #body="{ data }">{{ data.task_name || '—' }}</template>
        </Column>
        <Column header="Category">
          <template #body="{ data }">
            <span :class="['inline-block rounded-full px-2.5 py-0.5 text-[11px] font-bold', categoryBadgeClass(data.category)]">
              {{ categoryLabel(data.category) }}
            </span>
          </template>
        </Column>
        <Column header="Status" header-class="text-center" body-class="text-center">
          <template #body="{ data }">
            <span v-if="data.is_complete" class="inline-block rounded-full bg-emerald-100 px-2.5 py-0.5 text-[11px] font-bold text-emerald-700">Done</span>
            <span v-else class="inline-block rounded-full bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-muted">Pending</span>
          </template>
        </Column>
        <Column field="created_by" header="Created By" />
        <Column header="Created" header-class="num" body-class="num">
          <template #body="{ data }">{{ formatDate(data.created_at) }}</template>
        </Column>
        <Column header="Updated" header-class="num" body-class="num">
          <template #body="{ data }">{{ formatDate(data.updated_at) }}</template>
        </Column>
      </DataTable>
    </section>

    <!-- Shared Task Dialog (create + edit) -->
    <AiTaskDialog
      v-model:visible="showTaskDialog"
      :task="dialogTask"
      :loading="dialogLoading"
      @save="handleDialogSave"
    />
  </section>
</template>
