<script setup lang="ts">
import { ref, onMounted } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import Password from "primevue/password";
import InputText from "primevue/inputtext";
import { useGovernanceUsers } from "../composables/useGovernanceUsers";
import { useToast } from "@/shared/composables/useToast";

const ctrl = useGovernanceUsers();
const toast = useToast();
const isDialogOpen = ref(false);
const confirmDeleteId = ref<number | null>(null);
const resetPwUserId = ref<number | null>(null);
const resetPwValue = ref("");

const roleBadgeClass = (role: string) => {
  const map: Record<string, string> = {
    admin: "bg-red-50 text-red-700 dark:bg-red-950 dark:text-red-300",
    manager: "bg-blue-50 text-blue-700 dark:bg-blue-950 dark:text-blue-300",
    member: "bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300",
    viewer: "bg-canvas text-muted",
  };
  return map[role] ?? "bg-canvas text-muted";
};

function openCreate() {
  ctrl.startCreate();
  isDialogOpen.value = true;
}

function openEdit(id: number) {
  ctrl.selectUser(id);
  isDialogOpen.value = true;
}

function closeDialog() {
  isDialogOpen.value = false;
}

async function saveAndClose() {
  if (await ctrl.saveDraft()) {
    toast.success(ctrl.isCreating.value ? "User created successfully." : "User updated successfully.");
    closeDialog();
  }
}

function confirmDelete(id: number) {
  confirmDeleteId.value = id;
}

async function executeDelete() {
  if (confirmDeleteId.value !== null) {
    await ctrl.removeUser(confirmDeleteId.value);
    confirmDeleteId.value = null;
  }
}

function openResetPassword(id: number) {
  resetPwUserId.value = id;
  resetPwValue.value = "";
}

async function executeResetPassword() {
  if (resetPwUserId.value !== null && resetPwValue.value.trim()) {
    if (await ctrl.resetPassword(resetPwUserId.value, resetPwValue.value)) {
      resetPwUserId.value = null;
      resetPwValue.value = "";
    }
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
      <Button icon="pi pi-plus" label="Add user" @click="openCreate" />
    </section>

    <!-- Search fieldset -->
    <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Search" toggleable>
      <div class="grid gap-3">
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Username</span>
            <InputText
              class="mt-1 w-full"
              placeholder="Username"
              :model-value="ctrl.filters.value.username"
              @update:model-value="ctrl.filters.value = { ...ctrl.filters.value, username: $event as string }"
            />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Full Name</span>
            <InputText
              class="mt-1 w-full"
              placeholder="Full name"
              :model-value="ctrl.filters.value.fullName"
              @update:model-value="ctrl.filters.value = { ...ctrl.filters.value, fullName: $event as string }"
            />
          </label>
        </div>
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Email</span>
            <InputText
              class="mt-1 w-full"
              placeholder="Email"
              :model-value="ctrl.filters.value.email"
              @update:model-value="ctrl.filters.value = { ...ctrl.filters.value, email: $event as string }"
            />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Phone</span>
            <InputText
              class="mt-1 w-full"
              placeholder="Phone number"
              :model-value="ctrl.filters.value.phone"
              @update:model-value="ctrl.filters.value = { ...ctrl.filters.value, phone: $event as string }"
            />
          </label>
        </div>
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Role</span>
            <select
              class="mt-1 flex h-10 w-full items-center rounded-md border border-divider bg-panel px-3 text-sm"
              :value="ctrl.filters.value.role"
              @change="ctrl.filters.value = { ...ctrl.filters.value, role: ($event.target as HTMLSelectElement).value }"
            >
              <option v-for="r in ctrl.roles.value" :key="r" :value="r">{{ r }}</option>
            </select>
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Status</span>
            <select
              class="mt-1 flex h-10 w-full items-center rounded-md border border-divider bg-panel px-3 text-sm"
              :value="ctrl.filters.value.status"
              @change="ctrl.filters.value = { ...ctrl.filters.value, status: ($event.target as HTMLSelectElement).value }"
            >
              <option v-for="s in ctrl.statuses.value" :key="s" :value="s">{{ s }}</option>
            </select>
          </label>
        </div>
        <div class="flex items-center justify-end gap-2">
          <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined @click="ctrl.resetFilters()" />
          <Button icon="pi pi-search" label="Search" @click="ctrl.search()" />
        </div>
      </div>
    </Fieldset>

    <!-- Users table -->
    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">User list</h3>
        <span class="text-xs text-muted">{{ ctrl.filteredUsers.value.length.toLocaleString("en-US") }} users</span>
      </div>
      <DataTable
        class="app-data-table min-h-0"
        :empty-message="ctrl.loading.value ? 'Loading...' : 'No users match the search conditions.'"
        :row-class="() => 'cursor-pointer'"
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '860px' }"
        :value="ctrl.filteredUsers.value"
        paginator
        :rows="20"
        :rows-per-page-options="[20, 50, 100]"
        paginator-template="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        current-page-report-template="Showing {first} to {last} of {totalRecords}"
        @row-click="(e: any) => openEdit(e.data.id)"
      >
        <Column field="id" header="ID" body-class="font-mono text-xs text-muted" :style="{ width: '60px' }" />
        <Column field="full_name" header="User">
          <template #body="{ data }">
            <div class="flex items-center gap-2.5">
              <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand/10 text-xs font-bold text-brand">
                {{ data.full_name.charAt(0).toUpperCase() }}
              </span>
              <div>
                <span class="font-semibold text-ink">{{ data.full_name }}</span>
                <span class="block text-xs text-muted">{{ data.username }}</span>
              </div>
            </div>
          </template>
        </Column>
        <Column field="email" header="Email">
          <template #body="{ data }">
            <span class="text-xs text-secondary">{{ data.email || "—" }}</span>
          </template>
        </Column>
        <Column field="phone" header="Phone">
          <template #body="{ data }">
            <span class="text-xs text-secondary">{{ data.phone || "—" }}</span>
          </template>
        </Column>
        <Column field="position" header="Position">
          <template #body="{ data }">
            <span class="text-xs text-secondary">{{ data.position || "—" }}</span>
          </template>
        </Column>
        <Column field="roles" header="Roles">
          <template #body="{ data }">
            <div class="flex flex-wrap gap-1">
              <span
                v-for="role in data.roles"
                :key="role"
                :class="['rounded-md px-2 py-0.5 text-[11px] font-bold', roleBadgeClass(role)]"
              >
                {{ role }}
              </span>
            </div>
          </template>
        </Column>
        <Column field="is_active" header="Status" header-class="text-center" body-class="text-center">
          <template #body="{ data }">
            <span :class="[
              'inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-bold',
              data.is_active
                ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300'
                : 'bg-canvas text-muted',
            ]">
              <i :class="['pi text-[10px]', data.is_active ? 'pi-check-circle' : 'pi-minus-circle']" />
              {{ data.is_active ? "active" : "inactive" }}
            </span>
          </template>
        </Column>
        <Column header="Actions" header-class="text-center" body-class="text-center" :style="{ width: '90px' }">
          <template #body="{ data }">
            <div class="flex items-center justify-center gap-1">
              <Button icon="pi pi-key" severity="secondary" text rounded size="small" title="Reset password" @click.stop="openResetPassword(data.id)" />
              <Button icon="pi pi-trash" severity="danger" text rounded size="small" title="Delete user" @click.stop="confirmDelete(data.id)" />
            </div>
          </template>
        </Column>
      </DataTable>
    </section>

    <!-- Add / Edit dialog -->
    <Dialog
      :visible="isDialogOpen"
      class="w-full max-w-xl rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="isDialogOpen = $event"
    >
      <template #header>
        <div>
          <h3 class="font-bold text-ink">{{ ctrl.isCreating.value ? "Add User" : "Edit User" }}</h3>
          <p v-if="ctrl.draft.value && !ctrl.isCreating.value" class="mt-1 text-sm text-muted">
            ID: {{ ctrl.draft.value.id }} &middot; {{ ctrl.draft.value.username }}
          </p>
        </div>
      </template>

      <div v-if="ctrl.draft.value" class="space-y-4">
        <div class="grid gap-4 md:grid-cols-2">
          <label class="block">
            <span class="text-xs font-bold text-muted">Username <span class="text-red-500">*</span></span>
            <InputText
              class="mt-1 w-full"
              :model-value="ctrl.draft.value.username"
              placeholder="username"
              autofocus
              :disabled="!ctrl.isCreating.value"
              @update:model-value="ctrl.updateDraft('username', $event as string)"
            />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Full Name <span class="text-red-500">*</span></span>
            <InputText
              class="mt-1 w-full"
              :model-value="ctrl.draft.value.fullName"
              placeholder="Full name"
              @update:model-value="ctrl.updateDraft('fullName', $event as string)"
            />
          </label>
        </div>

        <label v-if="ctrl.isCreating.value" class="block">
          <span class="text-xs font-bold text-muted">Password <span class="text-red-500">*</span></span>
          <Password
            class="mt-1 w-full"
            input-class="w-full"
            :model-value="ctrl.draft.value.password"
            placeholder="Password"
            :feedback="false"
            toggle-mask
            @update:model-value="ctrl.updateDraft('password', $event as string)"
          />
        </label>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="block">
            <span class="text-xs font-bold text-muted">Email</span>
            <InputText
              class="mt-1 w-full"
              :model-value="ctrl.draft.value.email"
              placeholder="mail@example.com"
              @update:model-value="ctrl.updateDraft('email', $event as string)"
            />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Phone</span>
            <InputText
              class="mt-1 w-full"
              :model-value="ctrl.draft.value.phone"
              placeholder="Phone number"
              @update:model-value="ctrl.updateDraft('phone', $event as string)"
            />
          </label>
        </div>

        <label class="block">
          <span class="text-xs font-bold text-muted">Position</span>
          <InputText
            class="mt-1 w-full"
            :model-value="ctrl.draft.value.position"
            placeholder="Job title or position"
            @update:model-value="ctrl.updateDraft('position', $event as string)"
          />
        </label>

        <div>
          <span class="text-xs font-bold text-muted">Roles</span>
          <div class="mt-1 flex flex-wrap gap-2">
            <Button
              v-for="role in ctrl.availableRoles.value"
              :key="role"
              :class="[
                'flex h-9 items-center gap-1.5 rounded-md border px-3 text-sm font-semibold transition',
                ctrl.draft.value.roles.includes(role)
                  ? 'border-brand bg-emerald-50 text-brand dark:bg-emerald-950'
                  : 'border-divider bg-panel text-secondary hover:border-brand',
              ]"
              unstyled
              @click="ctrl.toggleDraftRole(role)"
            >
              <i :class="['pi text-xs', ctrl.draft.value.roles.includes(role) ? 'pi-check-circle' : 'pi-circle']" />
              {{ role }}
            </Button>
          </div>
        </div>

        <div v-if="!ctrl.isCreating.value">
          <span class="text-xs font-bold text-muted">Status</span>
          <div class="mt-1 grid grid-cols-2 rounded-md border border-divider bg-canvas p-1">
            <Button
              :class="[
                'flex h-9 items-center justify-center gap-1.5 rounded-md text-sm font-bold transition',
                ctrl.draft.value.isActive ? 'bg-panel text-ink shadow-sm' : 'text-muted hover:text-secondary',
              ]"
              unstyled
              @click="ctrl.updateDraft('isActive', true)"
            >
              <i class="pi pi-check-circle text-xs" />
              active
            </Button>
            <Button
              :class="[
                'flex h-9 items-center justify-center gap-1.5 rounded-md text-sm font-bold transition',
                !ctrl.draft.value.isActive ? 'bg-panel text-ink shadow-sm' : 'text-muted hover:text-secondary',
              ]"
              unstyled
              @click="ctrl.updateDraft('isActive', false)"
            >
              <i class="pi pi-minus-circle text-xs" />
              inactive
            </Button>
          </div>
        </div>
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
      <p class="text-sm text-secondary">Are you sure you want to delete this user? This action cannot be undone.</p>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="confirmDeleteId = null" />
          <Button label="Delete" severity="danger" @click="executeDelete" />
        </div>
      </template>
    </Dialog>

    <!-- Reset password dialog -->
    <Dialog
      :visible="resetPwUserId !== null"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="resetPwUserId = null"
    >
      <template #header>
        <h3 class="font-bold text-ink">Reset Password</h3>
      </template>
      <label class="block">
        <span class="text-xs font-bold text-muted">New Password <span class="text-red-500">*</span></span>
        <Password
          class="mt-1 w-full"
          input-class="w-full"
          placeholder="Enter new password"
          :model-value="resetPwValue"
          :feedback="false"
          toggle-mask
          @update:model-value="resetPwValue = $event as string"
        />
      </label>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="resetPwUserId = null" />
          <Button label="Reset" :disabled="!resetPwValue.trim()" @click="executeResetPassword" />
        </div>
      </template>
    </Dialog>
  </section>
</template>
