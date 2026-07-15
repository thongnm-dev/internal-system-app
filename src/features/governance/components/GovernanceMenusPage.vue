<script setup lang="ts">
import { ref } from "vue";
import Dialog from "primevue/dialog";
import { useGovernanceMenus } from "../composables/useGovernanceMenus";

const ctrl = useGovernanceMenus();
const isEditing = ref(false);

const groupBadgeClass = (group: string) =>
  group === "—"
    ? "bg-canvas text-muted"
    : group === "Tools"
      ? "bg-blue-50 text-blue-700"
      : "bg-emerald-50 text-emerald-700";

function openEdit(key: string) {
  ctrl.selectItem(key);
  isEditing.value = true;
}

function closeDialog() {
  isEditing.value = false;
}

function saveAndClose() {
  ctrl.saveDraft();
  closeDialog();
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Top bar -->
    <section class="flex flex-wrap items-end gap-3 rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <label class="block min-w-0 flex-1">
        <span class="text-xs font-bold text-muted">Search</span>
        <span class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
          <i class="pi pi-search shrink-0 text-muted" />
          <input
            class="min-w-0 flex-1 border-0 bg-transparent p-0 text-sm text-ink outline-none shadow-none"
            placeholder="Title, key, or path"
            type="search"
            :value="ctrl.searchQuery.value"
            @input="ctrl.searchQuery.value = ($event.target as HTMLInputElement).value"
          />
        </span>
      </label>
      <label class="block w-44">
        <span class="text-xs font-bold text-muted">Group</span>
        <select
          class="mt-1 flex h-10 w-full items-center rounded-md border border-divider bg-panel px-3 text-sm"
          :value="ctrl.filterGroup.value"
          @change="ctrl.filterGroup.value = ($event.target as HTMLSelectElement).value"
        >
          <option v-for="g in ctrl.groups.value" :key="g" :value="g">{{ g }}</option>
        </select>
      </label>
      <button
        class="flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
        type="button"
        title="Reset to defaults"
        @click="ctrl.resetToDefault()"
      >
        <i class="pi pi-refresh" />
        Reset
      </button>
    </section>

    <!-- Menu table -->
    <section class="min-h-0 flex-1 overflow-auto rounded-lg border border-divider bg-panel shadow-sm">
      <table class="w-full text-sm">
        <thead class="sticky top-0 z-10 bg-panel">
          <tr class="border-b border-divider text-left text-xs font-bold uppercase text-muted">
            <th class="px-4 py-3">Menu</th>
            <th class="px-4 py-3">Path</th>
            <th class="px-4 py-3">Group</th>
            <th class="px-4 py-3 text-center">Visible</th>
            <th class="px-4 py-3 text-center">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item in ctrl.filteredItems.value"
            :key="item.key"
            class="cursor-pointer border-b border-divider transition hover:bg-canvas"
            @click="openEdit(item.key)"
          >
            <td class="px-4 py-2.5">
              <div class="flex items-center gap-2">
                <i :class="`pi ${item.icon} text-muted`" />
                <span class="font-semibold text-ink">{{ item.title }}</span>
              </div>
              <span class="text-xs text-muted">{{ item.key }}</span>
            </td>
            <td class="px-4 py-2.5 font-mono text-xs text-secondary">{{ item.path }}</td>
            <td class="px-4 py-2.5">
              <span :class="['rounded-md px-2 py-0.5 text-[11px] font-bold', groupBadgeClass(item.group)]">
                {{ item.group }}
              </span>
            </td>
            <td class="px-4 py-2.5 text-center">
              <button
                type="button"
                :title="item.visible ? 'Hide menu' : 'Show menu'"
                @click.stop="ctrl.toggleVisibility(item.key)"
              >
                <i :class="['pi', item.visible ? 'pi-eye text-brand' : 'pi-eye-slash text-muted']" />
              </button>
            </td>
            <td class="px-4 py-2.5 text-center">
              <div class="flex items-center justify-center gap-1">
                <button
                  class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-canvas hover:text-ink"
                  type="button"
                  title="Move up"
                  @click.stop="ctrl.moveUp(item.key)"
                >
                  <i class="pi pi-chevron-up text-xs" />
                </button>
                <button
                  class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-canvas hover:text-ink"
                  type="button"
                  title="Move down"
                  @click.stop="ctrl.moveDown(item.key)"
                >
                  <i class="pi pi-chevron-down text-xs" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-if="ctrl.filteredItems.value.length === 0" class="p-6 text-center text-sm text-muted">No menus match the current filter.</p>
    </section>

    <!-- Edit dialog -->
    <Dialog
      :visible="isEditing"
      class="w-full max-w-xl rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="isEditing = $event"
    >
      <template #header>
        <div>
          <h3 class="font-bold text-ink">Edit Menu</h3>
          <p v-if="ctrl.draft.value" class="mt-1 text-sm text-muted">{{ ctrl.draft.value.key }}</p>
        </div>
      </template>

      <div v-if="ctrl.draft.value" class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Title <span class="text-red-500">*</span></span>
          <input
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            :value="ctrl.draft.value.title"
            placeholder="Menu title"
            autofocus
            @input="ctrl.updateDraft('title', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <label class="block">
          <span class="text-xs font-bold text-muted">Path</span>
          <input
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            :value="ctrl.draft.value.path"
            placeholder="/route-path"
            @input="ctrl.updateDraft('path', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="block">
            <span class="text-xs font-bold text-muted">Icon</span>
            <div class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
              <i :class="`pi ${ctrl.draft.value.icon} text-muted`" />
              <input
                class="min-w-0 flex-1 border-0 bg-transparent p-0 text-sm text-ink outline-none shadow-none"
                :value="ctrl.draft.value.icon"
                placeholder="pi-home"
                @input="ctrl.updateDraft('icon', ($event.target as HTMLInputElement).value)"
              />
            </div>
          </label>

          <label class="block">
            <span class="text-xs font-bold text-muted">Group</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :value="ctrl.draft.value.group"
              placeholder="— (none), Tools, Governance"
              @input="ctrl.updateDraft('group', ($event.target as HTMLInputElement).value)"
            />
          </label>

          <label class="block">
            <span class="text-xs font-bold text-muted">Order</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              type="number"
              :value="ctrl.draft.value.order"
              min="0"
              @input="ctrl.updateDraft('order', Number(($event.target as HTMLInputElement).value) || 0)"
            />
          </label>

          <label class="block">
            <span class="text-xs font-bold text-muted">Visible</span>
            <button
              type="button"
              :class="[
                'mt-1 flex h-10 w-full items-center gap-2 rounded-md border px-3 text-sm font-semibold transition',
                ctrl.draft.value.visible
                  ? 'border-brand bg-emerald-50 text-brand'
                  : 'border-divider bg-panel text-secondary hover:border-brand',
              ]"
              @click="ctrl.updateDraft('visible', !ctrl.draft.value.visible)"
            >
              <i :class="['pi', ctrl.draft.value.visible ? 'pi-eye' : 'pi-eye-slash']" />
              {{ ctrl.draft.value.visible ? "Shown in sidebar" : "Hidden from sidebar" }}
            </button>
          </label>
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
            Save
          </button>
        </div>
      </template>
    </Dialog>
  </section>
</template>
