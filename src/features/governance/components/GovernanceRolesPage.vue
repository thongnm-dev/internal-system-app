<script setup lang="ts">
import { ref, onMounted } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Textarea from "primevue/textarea";
import { useGovernanceRoles } from "../composables/useGovernanceRoles";
import { useToast } from "@/shared/composables/useToast";

const ctrl = useGovernanceRoles();
const toast = useToast();
const isDialogOpen = ref(false);
const confirmDeleteId = ref<number | null>(null);

function openCreate() {
  ctrl.startCreate();
  isDialogOpen.value = true;
}

function openEdit(id: number) {
  ctrl.selectRole(id);
  isDialogOpen.value = true;
}

function closeDialog() {
  isDialogOpen.value = false;
}

async function saveAndClose() {
  if (await ctrl.saveDraft()) {
    toast.success(ctrl.isCreating.value ? "Role created successfully." : "Role updated successfully.");
    closeDialog();
  }
}

function confirmDelete(id: number) {
  confirmDeleteId.value = id;
}

async function executeDelete() {
  if (confirmDeleteId.value !== null) {
    const ok = await ctrl.removeRole(confirmDeleteId.value);
    if (ok) toast.success("Role deleted successfully.");
    confirmDeleteId.value = null;
  }
}

onMounted(() => ctrl.init());
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Error banner -->
    <p v-if="ctrl.error.value" class="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300">
      {{ ctrl.error.value }}
    </p>

    <!-- Action bar -->
    <section class="flex items-center justify-end rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <Button icon="pi pi-plus" label="Add role" @click="openCreate" />
    </section>

    <!-- Search fieldset -->
    <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Search" toggleable>
      <div class="grid gap-3">
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Name</span>
            <InputText
              class="mt-1 w-full"
              placeholder="Role name"
              :model-value="ctrl.filters.value.name"
              @update:model-value="ctrl.filters.value = { ...ctrl.filters.value, name: $event as string }"
            />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Description</span>
            <InputText
              class="mt-1 w-full"
              placeholder="Description"
              :model-value="ctrl.filters.value.description"
              @update:model-value="ctrl.filters.value = { ...ctrl.filters.value, description: $event as string }"
            />
          </label>
        </div>
        <div class="flex items-center justify-end gap-2">
          <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined @click="ctrl.resetFilters()" />
          <Button icon="pi pi-search" label="Search" @click="ctrl.search()" />
        </div>
      </div>
    </Fieldset>

    <!-- Roles table -->
    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Role list</h3>
        <span class="text-xs text-muted">{{ ctrl.filteredRoles.value.length.toLocaleString("en-US") }} roles</span>
      </div>
      <DataTable
        class="app-data-table min-h-0"
        :empty-message="ctrl.loading.value ? 'Loading...' : 'No roles match the search conditions.'"
        :row-class="() => 'cursor-pointer'"
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '640px' }"
        :value="ctrl.filteredRoles.value"
        paginator
        :rows="20"
        :rows-per-page-options="[20, 50, 100]"
        paginator-template="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        current-page-report-template="Showing {first} to {last} of {totalRecords}"
        @row-click="(e: any) => openEdit(e.data.id)"
      >
        <Column field="id" header="ID" body-class="font-mono text-xs text-muted" :style="{ width: '60px' }" />
        <Column field="name" header="Role">
          <template #body="{ data }">
            <div class="flex items-center gap-2.5">
              <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand/10 text-xs font-bold text-brand">
                <i class="pi pi-shield text-xs" />
              </span>
              <span class="font-semibold text-ink">{{ data.name }}</span>
            </div>
          </template>
        </Column>
        <Column field="description" header="Description">
          <template #body="{ data }">
            <span class="text-xs text-secondary">{{ data.description || "—" }}</span>
          </template>
        </Column>
        <Column field="user_count" header="Users" header-class="text-center" body-class="text-center" :style="{ width: '90px' }">
          <template #body="{ data }">
            <span class="inline-flex items-center gap-1 rounded-md bg-canvas px-2 py-0.5 text-[11px] font-bold text-secondary">
              <i class="pi pi-users text-[10px]" />
              {{ data.user_count.toLocaleString("en-US") }}
            </span>
          </template>
        </Column>
        <Column field="created_at" header="Created" body-class="text-xs text-muted" :style="{ width: '170px' }">
          <template #body="{ data }">
            <span class="text-xs text-muted">{{ data.created_at || "—" }}</span>
          </template>
        </Column>
        <Column header="Actions" header-class="text-center" body-class="text-center" :style="{ width: '70px' }">
          <template #body="{ data }">
            <div class="flex items-center justify-center gap-1">
              <Button icon="pi pi-trash" severity="danger" text rounded size="small" title="Delete role" @click.stop="confirmDelete(data.id)" />
            </div>
          </template>
        </Column>
      </DataTable>
    </section>

    <!-- Add / Edit dialog -->
    <Dialog
      :visible="isDialogOpen"
      class="w-full max-w-lg rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="isDialogOpen = $event"
    >
      <template #header>
        <div>
          <h3 class="font-bold text-ink">{{ ctrl.isCreating.value ? "Add Role" : "Edit Role" }}</h3>
          <p v-if="ctrl.draft.value && !ctrl.isCreating.value" class="mt-1 text-sm text-muted">
            ID: {{ ctrl.draft.value.id }}
          </p>
        </div>
      </template>

      <div v-if="ctrl.draft.value" class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Name <span class="text-red-500">*</span></span>
          <InputText
            class="mt-1 w-full"
            :model-value="ctrl.draft.value.name"
            placeholder="e.g. manager"
            autofocus
            @update:model-value="ctrl.updateDraft('name', $event as string)"
          />
        </label>

        <label class="block">
          <span class="text-xs font-bold text-muted">Description</span>
          <Textarea
            class="mt-1 w-full"
            :model-value="ctrl.draft.value.description"
            placeholder="What this role is allowed to do"
            rows="3"
            auto-resize
            @update:model-value="ctrl.updateDraft('description', $event as string)"
          />
        </label>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="closeDialog" />
          <Button :label="ctrl.isCreating.value ? 'Create' : 'Save'" @click="saveAndClose" />
        </div>
      </template>
    </Dialog>

    <!-- Delete confirmation dialog -->
    <Dialog
      :visible="confirmDeleteId !== null"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="confirmDeleteId = null"
    >
      <template #header>
        <h3 class="font-bold text-ink">Confirm Delete</h3>
      </template>
      <p class="text-sm text-secondary">Are you sure you want to delete this role? This action cannot be undone.</p>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="confirmDeleteId = null" />
          <Button label="Delete" severity="danger" @click="executeDelete" />
        </div>
      </template>
    </Dialog>
  </section>
</template>
