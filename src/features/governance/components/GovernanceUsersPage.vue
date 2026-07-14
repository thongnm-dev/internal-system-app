<script setup lang="ts">
import { ref, onMounted } from "vue";
import Dialog from "primevue/dialog";
import { useGovernanceUsers } from "../composables/useGovernanceUsers";

const ctrl = useGovernanceUsers();
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
  if (await ctrl.saveDraft()) closeDialog();
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

    <!-- Top bar -->
    <section class="flex flex-wrap items-end gap-3 rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <label class="block min-w-0 flex-1">
        <span class="text-xs font-bold text-muted">Search</span>
        <span class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
          <i class="pi pi-search shrink-0 text-muted" />
          <input
            class="min-w-0 flex-1 border-0 bg-transparent p-0 text-sm text-ink outline-none shadow-none"
            placeholder="Username, name, email, or position"
            type="search"
            :value="ctrl.searchQuery.value"
            @input="ctrl.searchQuery.value = ($event.target as HTMLInputElement).value"
          />
        </span>
      </label>
      <label class="block w-36">
        <span class="text-xs font-bold text-muted">Role</span>
        <select
          class="mt-1 flex h-10 w-full items-center rounded-md border border-divider bg-panel px-3 text-sm"
          :value="ctrl.filterRole.value"
          @change="ctrl.filterRole.value = ($event.target as HTMLSelectElement).value"
        >
          <option v-for="r in ctrl.roles.value" :key="r" :value="r">{{ r }}</option>
        </select>
      </label>
      <label class="block w-36">
        <span class="text-xs font-bold text-muted">Status</span>
        <select
          class="mt-1 flex h-10 w-full items-center rounded-md border border-divider bg-panel px-3 text-sm"
          :value="ctrl.filterStatus.value"
          @change="ctrl.filterStatus.value = ($event.target as HTMLSelectElement).value"
        >
          <option v-for="s in ctrl.statuses.value" :key="s" :value="s">{{ s }}</option>
        </select>
      </label>
      <div class="flex items-center gap-2">
        <button
          class="flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
          type="button"
          title="Reload users"
          @click="ctrl.loadUsers()"
        >
          <i class="pi pi-refresh" />
          Reload
        </button>
        <button
          class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
          type="button"
          @click="openCreate"
        >
          <i class="pi pi-plus" />
          Add user
        </button>
      </div>
    </section>

    <!-- Stats -->
    <section class="grid grid-cols-3 gap-3">
      <div class="rounded-lg border border-divider bg-panel p-3 shadow-sm">
        <span class="text-xs font-bold text-muted">Total</span>
        <strong class="mt-1 block text-2xl text-ink">{{ ctrl.stats.value.total }}</strong>
      </div>
      <div class="rounded-lg border border-divider bg-panel p-3 shadow-sm">
        <span class="text-xs font-bold text-emerald-600">Active</span>
        <strong class="mt-1 block text-2xl text-ink">{{ ctrl.stats.value.active }}</strong>
      </div>
      <div class="rounded-lg border border-divider bg-panel p-3 shadow-sm">
        <span class="text-xs font-bold text-muted">Inactive</span>
        <strong class="mt-1 block text-2xl text-ink">{{ ctrl.stats.value.inactive }}</strong>
      </div>
    </section>

    <!-- Loading -->
    <p v-if="ctrl.loading.value" class="flex items-center gap-2 rounded-lg border border-divider bg-panel p-4 text-sm text-muted shadow-sm">
      <i class="pi pi-spinner animate-spin" />
      Loading users...
    </p>

    <!-- Users table -->
    <section v-if="!ctrl.loading.value" class="min-h-0 flex-1 overflow-auto rounded-lg border border-divider bg-panel shadow-sm">
      <table class="w-full text-sm">
        <thead class="sticky top-0 z-10 bg-panel">
          <tr class="border-b border-divider text-left text-xs font-bold uppercase text-muted">
            <th class="px-4 py-3">ID</th>
            <th class="px-4 py-3">User</th>
            <th class="px-4 py-3">Email</th>
            <th class="px-4 py-3">Position</th>
            <th class="px-4 py-3">Roles</th>
            <th class="px-4 py-3 text-center">Status</th>
            <th class="px-4 py-3 text-center">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="u in ctrl.filteredUsers.value"
            :key="u.id"
            class="cursor-pointer border-b border-divider transition hover:bg-canvas"
            @click="openEdit(u.id)"
          >
            <td class="px-4 py-2.5 font-mono text-xs text-muted">{{ u.id }}</td>
            <td class="px-4 py-2.5">
              <div class="flex items-center gap-2.5">
                <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand/10 text-xs font-bold text-brand">
                  {{ u.full_name.charAt(0).toUpperCase() }}
                </span>
                <div>
                  <span class="font-semibold text-ink">{{ u.full_name }}</span>
                  <span class="block text-xs text-muted">{{ u.username }}</span>
                </div>
              </div>
            </td>
            <td class="px-4 py-2.5 text-xs text-secondary">{{ u.email || "—" }}</td>
            <td class="px-4 py-2.5 text-xs text-secondary">{{ u.position || "—" }}</td>
            <td class="px-4 py-2.5">
              <div class="flex flex-wrap gap-1">
                <span
                  v-for="role in u.roles"
                  :key="role"
                  :class="['rounded-md px-2 py-0.5 text-[11px] font-bold', roleBadgeClass(role)]"
                >
                  {{ role }}
                </span>
              </div>
            </td>
            <td class="px-4 py-2.5 text-center">
              <span :class="[
                'inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-bold',
                u.is_active
                  ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300'
                  : 'bg-canvas text-muted',
              ]">
                <i :class="['pi text-[10px]', u.is_active ? 'pi-check-circle' : 'pi-minus-circle']" />
                {{ u.is_active ? "active" : "inactive" }}
              </span>
            </td>
            <td class="px-4 py-2.5 text-center">
              <div class="flex items-center justify-center gap-1">
                <button
                  class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-canvas hover:text-ink"
                  type="button"
                  title="Reset password"
                  @click.stop="openResetPassword(u.id)"
                >
                  <i class="pi pi-key text-xs" />
                </button>
                <button
                  class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-canvas hover:text-red-600"
                  type="button"
                  title="Delete user"
                  @click.stop="confirmDelete(u.id)"
                >
                  <i class="pi pi-trash text-xs" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-if="ctrl.filteredUsers.value.length === 0" class="p-6 text-center text-sm text-muted">No users match the current filter.</p>
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
