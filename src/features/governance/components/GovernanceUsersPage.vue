<script setup lang="ts">
import { ref, onMounted } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import Dialog from "primevue/dialog";
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
      <button class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90" type="button" @click="openCreate">
        <i class="pi pi-plus" />
        Add user
      </button>
    </section>

    <!-- Search fieldset -->
    <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Search" toggleable>
      <div class="grid gap-3">
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Username</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Username"
              type="search"
              :value="ctrl.filters.value.username"
              @input="ctrl.filters.value = { ...ctrl.filters.value, username: ($event.target as HTMLInputElement).value }"
            />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Full Name</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Full name"
              type="search"
              :value="ctrl.filters.value.fullName"
              @input="ctrl.filters.value = { ...ctrl.filters.value, fullName: ($event.target as HTMLInputElement).value }"
            />
          </label>
        </div>
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Email</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Email"
              type="search"
              :value="ctrl.filters.value.email"
              @input="ctrl.filters.value = { ...ctrl.filters.value, email: ($event.target as HTMLInputElement).value }"
            />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Phone</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Phone number"
              type="search"
              :value="ctrl.filters.value.phone"
              @input="ctrl.filters.value = { ...ctrl.filters.value, phone: ($event.target as HTMLInputElement).value }"
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
          <button class="flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas" type="button" @click="ctrl.resetFilters()">
            <i class="pi pi-refresh" />
            Reset
          </button>
          <button class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90" type="button" @click="ctrl.search()">
            <i class="pi pi-search" />
            Search
          </button>
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
              <button
                class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-canvas hover:text-ink"
                type="button"
                title="Reset password"
                @click.stop="openResetPassword(data.id)"
              >
                <i class="pi pi-key text-xs" />
              </button>
              <button
                class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-canvas hover:text-red-600"
                type="button"
                title="Delete user"
                @click.stop="confirmDelete(data.id)"
              >
                <i class="pi pi-trash text-xs" />
              </button>
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
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100 disabled:opacity-50"
              :value="ctrl.draft.value.username"
              placeholder="username"
              autofocus
              :disabled="!ctrl.isCreating.value"
              @input="ctrl.updateDraft('username', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Full Name <span class="text-red-500">*</span></span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :value="ctrl.draft.value.fullName"
              placeholder="Full name"
              @input="ctrl.updateDraft('fullName', ($event.target as HTMLInputElement).value)"
            />
          </label>
        </div>

        <label v-if="ctrl.isCreating.value" class="block">
          <span class="text-xs font-bold text-muted">Password <span class="text-red-500">*</span></span>
          <input
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            type="password"
            :value="ctrl.draft.value.password"
            placeholder="Password"
            @input="ctrl.updateDraft('password', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="block">
            <span class="text-xs font-bold text-muted">Email</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              type="email"
              :value="ctrl.draft.value.email"
              placeholder="mail@example.com"
              @input="ctrl.updateDraft('email', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Phone</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :value="ctrl.draft.value.phone"
              placeholder="Phone number"
              @input="ctrl.updateDraft('phone', ($event.target as HTMLInputElement).value)"
            />
          </label>
        </div>

        <label class="block">
          <span class="text-xs font-bold text-muted">Position</span>
          <input
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            :value="ctrl.draft.value.position"
            placeholder="Job title or position"
            @input="ctrl.updateDraft('position', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <div>
          <span class="text-xs font-bold text-muted">Roles</span>
          <div class="mt-1 flex flex-wrap gap-2">
            <button
              v-for="role in ctrl.availableRoles.value"
              :key="role"
              type="button"
              :class="[
                'flex h-9 items-center gap-1.5 rounded-md border px-3 text-sm font-semibold transition',
                ctrl.draft.value.roles.includes(role)
                  ? 'border-brand bg-emerald-50 text-brand dark:bg-emerald-950'
                  : 'border-divider bg-panel text-secondary hover:border-brand',
              ]"
              @click="ctrl.toggleDraftRole(role)"
            >
              <i :class="['pi text-xs', ctrl.draft.value.roles.includes(role) ? 'pi-check-circle' : 'pi-circle']" />
              {{ role }}
            </button>
          </div>
        </div>

        <div v-if="!ctrl.isCreating.value">
          <span class="text-xs font-bold text-muted">Status</span>
          <div class="mt-1 grid grid-cols-2 rounded-md border border-divider bg-canvas p-1">
            <button
              type="button"
              :class="[
                'flex h-9 items-center justify-center gap-1.5 rounded-md text-sm font-bold transition',
                ctrl.draft.value.isActive ? 'bg-panel text-ink shadow-sm' : 'text-muted hover:text-secondary',
              ]"
              @click="ctrl.updateDraft('isActive', true)"
            >
              <i class="pi pi-check-circle text-xs" />
              active
            </button>
            <button
              type="button"
              :class="[
                'flex h-9 items-center justify-center gap-1.5 rounded-md text-sm font-bold transition',
                !ctrl.draft.value.isActive ? 'bg-panel text-ink shadow-sm' : 'text-muted hover:text-secondary',
              ]"
              @click="ctrl.updateDraft('isActive', false)"
            >
              <i class="pi pi-minus-circle text-xs" />
              inactive
            </button>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <button
            class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            type="button"
            @click="closeDialog"
          >
            Cancel
          </button>
          <button
            class="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
            type="button"
            @click="saveAndClose"
          >
            {{ ctrl.isCreating.value ? "Create" : "Save" }}
          </button>
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
          <button
            class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            type="button"
            @click="confirmDeleteId = null"
          >
            Cancel
          </button>
          <button
            class="h-10 rounded-md bg-red-600 px-4 text-sm font-bold text-white hover:opacity-90"
            type="button"
            @click="executeDelete"
          >
            Delete
          </button>
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
        <input
          class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
          type="password"
          placeholder="Enter new password"
          :value="resetPwValue"
          @input="resetPwValue = ($event.target as HTMLInputElement).value"
        />
      </label>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <button
            class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            type="button"
            @click="resetPwUserId = null"
          >
            Cancel
          </button>
          <button
            class="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            :disabled="!resetPwValue.trim()"
            @click="executeResetPassword"
          >
            Reset
          </button>
        </div>
      </template>
    </Dialog>
  </section>
</template>
