<script setup lang="ts">
import { onMounted } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Dialog from "primevue/dialog";
import Select from "primevue/select";
import { ref } from "vue";
import { useStoreProcedure } from "../composables/useStoreProcedure";
import { useToast } from "@/shared/composables/useToast";

import { highlightSql } from "@/features/tools/utils/highlightSql";

const ctrl = useStoreProcedure();
const toast = useToast();
const confirmExecute = ref(false);

function openConfirm() {
  confirmExecute.value = true;
}

async function handleExecute() {
  confirmExecute.value = false;
  const allOk = await ctrl.executeFiltered();
  if (allOk) {
    toast.success(`All ${ctrl.summary.value?.total} stored procedures executed successfully.`);
  } else {
    toast.error(
      `${ctrl.summary.value?.error_count} of ${ctrl.summary.value?.total} stored procedures failed.`,
    );
  }
}

async function handleExecuteSingle(name: string) {
  const ok = await ctrl.executeSingle(name);
  if (ok) {
    toast.success(`${name} executed successfully.`);
  } else {
    toast.error(`${name} execution failed.`);
  }
}

onMounted(() => ctrl.init());
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Action bar -->
    <section class="flex items-center justify-end rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="flex items-center gap-2">
        <Button
          v-if="ctrl.hasResults.value"
          icon="pi pi-refresh"
          label="Reset"
          severity="secondary"
          outlined
          @click="ctrl.resetResults()"
        />
        <Button
          icon="pi pi-database"
          label="Execute All"
          severity="warning"
          :loading="ctrl.executing.value"
          @click="openConfirm"
        />
      </div>
    </section>

    <!-- Search fieldset -->
    <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Search" toggleable>
      <div class="grid gap-3">
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Procedure Name</span>
            <InputText class="mt-1 w-full" placeholder="Name" :model-value="ctrl.filterText.value" @update:model-value="ctrl.filterText.value = $event as string" />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Group</span>
            <Select class="mt-1 w-full" placeholder="All groups" :options="ctrl.groupOptions.value" :model-value="ctrl.filterGroup.value" show-clear @update:model-value="ctrl.filterGroup.value = $event ?? ''" />
          </label>
        </div>
        <div class="flex items-center justify-end gap-2">
          <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined @click="ctrl.resetFilters()" />
          <Button icon="pi pi-search" label="Search" />
        </div>
      </div>
    </Fieldset>

    <!-- SP table -->
    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Stored Procedure list</h3>
        <div class="flex items-center gap-3">
          <template v-if="ctrl.hasResults.value && ctrl.summary.value">
            <span class="inline-flex items-center gap-1 rounded-md bg-green-50 px-2 py-0.5 text-[11px] font-bold text-green-700 dark:bg-green-950 dark:text-green-400">
              <i class="pi pi-check text-[10px]" />
              {{ ctrl.summary.value.success_count }}
            </span>
            <span
              v-if="ctrl.summary.value.error_count > 0"
              class="inline-flex items-center gap-1 rounded-md bg-red-50 px-2 py-0.5 text-[11px] font-bold text-red-700 dark:bg-red-950 dark:text-red-400"
            >
              <i class="pi pi-times text-[10px]" />
              {{ ctrl.summary.value.error_count }}
            </span>
          </template>
          <span class="text-xs text-muted">{{ ctrl.filteredResults.value.length.toLocaleString("en-US") }} procedures</span>
        </div>
      </div>
      <p v-if="ctrl.error.value" class="border-b border-red-200 bg-red-50 px-4 py-2 text-sm text-red-700">{{ ctrl.error.value }}</p>
      <DataTable
        class="app-data-table min-h-0"
        :empty-message="ctrl.loading.value ? 'Loading...' : 'No stored procedures found.'"
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '640px' }"
        :value="ctrl.filteredResults.value"
        paginator
        :rows="50"
        :rows-per-page-options="[30, 50, 100]"
        paginator-template="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        current-page-report-template="Showing {first} to {last} of {totalRecords}"
      >
        <Column header="#" body-class="font-mono text-xs text-muted" :style="{ width: '60px' }">
          <template #body="{ index }">
            {{ index + 1 }}
          </template>
        </Column>
        <Column field="name" header="Procedure Name">
          <template #body="{ data }">
            <div class="flex items-center gap-2.5">
              <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand/10 text-xs font-bold text-brand">
                <i class="pi pi-code text-xs" />
              </span>
              <span class="font-mono text-sm font-semibold text-ink">{{ data.name }}</span>
            </div>
          </template>
        </Column>
        <Column header-class="text-center" body-class="text-center" :style="{ width: '120px' }">
          <template #body="{ data }">
            <div class="flex items-center justify-center gap-1">
              <Button
                icon="pi pi-eye"
                severity="secondary"
                text
                rounded
                size="small"
                @click="ctrl.viewScript(data.name)"
              />
              <Button
                icon="pi pi-play"
                severity="info"
                text
                rounded
                size="small"
                :loading="ctrl.executingNames.value.has(data.name)"
                @click="handleExecuteSingle(data.name)"
              />
            </div>
          </template>
        </Column>
      </DataTable>
    </section>

    <!-- View script dialog -->
    <Dialog
      :visible="!!ctrl.viewingName.value"
      class="w-full max-w-3xl rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="!$event && ctrl.closeViewer()"
    >
      <template #header>
        <h3 class="font-mono text-sm font-bold text-ink">{{ ctrl.viewingName.value }}</h3>
      </template>
      <div v-if="ctrl.viewingLoading.value" class="flex items-center justify-center py-8">
        <i class="pi pi-spin pi-spinner text-xl text-muted" />
      </div>
      <pre
        v-else
        class="sp-viewer max-h-[60vh] overflow-auto rounded-lg border border-divider bg-canvas p-4 font-mono text-sm leading-relaxed text-ink"
        v-html="highlightSql(ctrl.viewingContent.value)"
      />
      <template #footer>
        <div class="flex items-center justify-end">
          <Button
            icon="pi pi-play"
            label="Execute"
            severity="warning"
            :loading="ctrl.executingNames.value.has(ctrl.viewingName.value)"
            @click="handleExecuteSingle(ctrl.viewingName.value)"
          />
        </div>
      </template>
    </Dialog>

    <!-- Confirm dialog -->
    <Dialog
      :visible="confirmExecute"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="confirmExecute = $event"
    >
      <template #header>
        <h3 class="font-bold text-ink">Confirm Execute</h3>
      </template>
      <div class="space-y-3">
        <p class="text-sm text-secondary">
          This will run <strong>CREATE OR REPLACE</strong> for
          <strong>{{ ctrl.filteredResults.value.length }}</strong> stored procedures
          against the connected database.
        </p>
        <p class="text-xs text-muted">
          Existing functions will be replaced. This action is safe for production databases
          as it only updates function definitions.
        </p>
      </div>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button
            label="Cancel"
            severity="secondary"
            @click="confirmExecute = false"
          />
          <Button
            icon="pi pi-database"
            label="Execute All"
            severity="warning"
            @click="handleExecute"
          />
        </div>
      </template>
    </Dialog>
  </section>
</template>

<style scoped>
.sp-viewer :deep(.sql-kw) { color: #2563eb; font-weight: 600; }
.sp-viewer :deep(.sql-str) { color: #16a34a; }
.sp-viewer :deep(.sql-com) { color: #6b7280; font-style: italic; }
.sp-viewer :deep(.sql-num) { color: #b45309; }
[data-theme="dark"] .sp-viewer :deep(.sql-kw) { color: #93c5fd; }
[data-theme="dark"] .sp-viewer :deep(.sql-str) { color: #86efac; }
[data-theme="dark"] .sp-viewer :deep(.sql-com) { color: #9ca3af; }
[data-theme="dark"] .sp-viewer :deep(.sql-num) { color: #fcd34d; }
</style>
