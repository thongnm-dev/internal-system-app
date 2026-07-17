<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
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
          <InputText
            class="min-w-0 flex-1 border-0 bg-transparent p-0 text-sm text-ink outline-none shadow-none"
            placeholder="Title, key, or path"
            :model-value="ctrl.searchQuery.value"
            @update:model-value="ctrl.searchQuery.value = $event as string"
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
      <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined title="Reset to defaults" @click="ctrl.resetToDefault()" />
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
              <Button
                :icon="item.visible ? 'pi pi-eye' : 'pi pi-eye-slash'"
                :title="item.visible ? 'Hide menu' : 'Show menu'"
                text
                rounded
                size="small"
                :class="item.visible ? 'text-brand' : 'text-muted'"
                @click.stop="ctrl.toggleVisibility(item.key)"
              />
            </td>
            <td class="px-4 py-2.5 text-center">
              <div class="flex items-center justify-center gap-1">
                <Button icon="pi pi-chevron-up" text rounded size="small" title="Move up" @click.stop="ctrl.moveUp(item.key)" />
                <Button icon="pi pi-chevron-down" text rounded size="small" title="Move down" @click.stop="ctrl.moveDown(item.key)" />
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
          <InputText
            class="mt-1 w-full"
            :model-value="ctrl.draft.value.title"
            placeholder="Menu title"
            autofocus
            @update:model-value="ctrl.updateDraft('title', $event as string)"
          />
        </label>

        <label class="block">
          <span class="text-xs font-bold text-muted">Path</span>
          <InputText
            class="mt-1 w-full"
            :model-value="ctrl.draft.value.path"
            placeholder="/route-path"
            @update:model-value="ctrl.updateDraft('path', $event as string)"
          />
        </label>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="block">
            <span class="text-xs font-bold text-muted">Icon</span>
            <div class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
              <i :class="`pi ${ctrl.draft.value.icon} text-muted`" />
              <InputText
                class="min-w-0 flex-1 border-0 bg-transparent p-0 text-sm text-ink outline-none shadow-none"
                :model-value="ctrl.draft.value.icon"
                placeholder="pi-home"
                @update:model-value="ctrl.updateDraft('icon', $event as string)"
              />
            </div>
          </label>

          <label class="block">
            <span class="text-xs font-bold text-muted">Group</span>
            <InputText
              class="mt-1 w-full"
              :model-value="ctrl.draft.value.group"
              placeholder="— (none), Tools, Governance"
              @update:model-value="ctrl.updateDraft('group', $event as string)"
            />
          </label>

          <label class="block">
            <span class="text-xs font-bold text-muted">Order</span>
            <InputNumber
              class="mt-1 w-full"
              :model-value="ctrl.draft.value.order"
              :min="0"
              :useGrouping="false"
              @update:model-value="ctrl.updateDraft('order', $event ?? 0)"
            />
          </label>

          <label class="block">
            <span class="text-xs font-bold text-muted">Visible</span>
            <Button
              :icon="ctrl.draft.value.visible ? 'pi pi-eye' : 'pi pi-eye-slash'"
              :label="ctrl.draft.value.visible ? 'Shown in sidebar' : 'Hidden from sidebar'"
              :class="[
                'mt-1 w-full',
                ctrl.draft.value.visible
                  ? 'border-brand bg-emerald-50 text-brand'
                  : '',
              ]"
              :severity="ctrl.draft.value.visible ? undefined : 'secondary'"
              :outlined="!ctrl.draft.value.visible"
              @click="ctrl.updateDraft('visible', !ctrl.draft.value.visible)"
            />
          </label>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" outlined @click="closeDialog" />
          <Button label="Save" @click="saveAndClose" />
        </div>
      </template>
    </Dialog>
  </section>
</template>
