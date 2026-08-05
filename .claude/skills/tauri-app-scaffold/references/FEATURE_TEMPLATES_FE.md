# Feature templates — Frontend

Companion to [FEATURE_TEMPLATES.md](FEATURE_TEMPLATES.md) — read that first for the why/when. This file is the ready-to-copy frontend baseline: types → IPC commands → composable → page, all under the fictitious `template`/`Template` domain.

## Types — `src/_/types/_template.ts`

Rule: one file per domain; field names stay snake_case to mirror the Rust structs untouched (no camelCase conversion on either side); optional fields (`?`) mirror `Option<T>`.

```ts
// TEMPLATE — not a real domain, not imported anywhere.
// Copy to `<domain>.ts`, rename the three types to match the domain.

export type TemplateItemSummary = {
  id: number;
  name: string;
  description: string;
  created_at: string;
};

export type CreateTemplateItemRequest = {
  name: string;
  description?: string;
};

export type UpdateTemplateItemRequest = {
  name: string;
  description?: string;
};
```

## IPC commands — `src/tauri/commands/_template.ts`

Rule: one file per backend domain, mirroring the matching `<domain>_commands.rs` 1:1. A request DTO is always passed wrapped as `{ request }` (or `{ request, <otherArg> }`) because that's the parameter name the Tauri macro expects on the Rust side.

```ts
// TEMPLATE — not exported from `index.ts`; importing this elsewhere won't resolve. That's intentional.
// Copy to `<domain>.ts`, rename functions/command strings/types, then
// add `export * from "./<domain>";` to `index.ts`.

import { safeInvoke } from "./_base";
import type {
  CreateTemplateItemRequest,
  TemplateItemSummary,
  UpdateTemplateItemRequest,
} from "@/_/types/_template";

export function listTemplateItems() {
  return safeInvoke<TemplateItemSummary[]>("list_template_items");
}

export function createTemplateItem(request: CreateTemplateItemRequest) {
  return safeInvoke<TemplateItemSummary>("create_template_item", { request });
}

export function updateTemplateItem(itemId: number, request: UpdateTemplateItemRequest) {
  return safeInvoke<TemplateItemSummary>("update_template_item", { itemId, request });
}

export function deleteTemplateItem(itemId: number) {
  return safeInvoke<void>("delete_template_item", { itemId });
}
```

## Composable — `src/features/_template/composables/useTemplate.ts`

Rule: the composable owns ALL state, data fetching, and Tauri calls for its page(s). Components call the composable; they never call `safeInvoke`/command functions directly. Errors are caught here and exposed as a plain `error` ref (string), not thrown up to the component.

```ts
// TEMPLATE — copy this folder's composable + page to `src/features/<domain>/`,
// rename `Template`/`template` throughout, point imports at the real
// `<domain>` command/type files.

import { ref, computed } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import {
  listTemplateItems,
  createTemplateItem,
  updateTemplateItem,
  deleteTemplateItem,
} from "@/tauri/commands/_template";
import type {
  TemplateItemSummary,
  CreateTemplateItemRequest,
  UpdateTemplateItemRequest,
} from "@/_/types/_template";

export function useTemplate() {
  const items = ref<TemplateItemSummary[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const draft = ref<{ id: number; name: string; description: string } | null>(null);
  const isCreating = ref(false);
  const nameFilter = ref("");

  const filteredItems = computed(() => {
    const q = nameFilter.value.toLowerCase();
    return q ? items.value.filter((i) => i.name.toLowerCase().includes(q)) : items.value;
  });

  async function fetchItems() {
    if (!canUseTauriRuntime()) return;
    loading.value = true;
    error.value = null;
    try {
      items.value = await listTemplateItems();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  function startCreate() {
    isCreating.value = true;
    draft.value = { id: 0, name: "", description: "" };
  }

  function selectItem(id: number) {
    isCreating.value = false;
    const item = items.value.find((i) => i.id === id);
    if (item) draft.value = { id: item.id, name: item.name, description: item.description };
  }

  async function saveDraft(): Promise<boolean> {
    if (!draft.value?.name.trim()) {
      error.value = "Name is required.";
      return false;
    }
    error.value = null;
    try {
      if (isCreating.value) {
        await createTemplateItem({ name: draft.value.name, description: draft.value.description || undefined });
      } else {
        await updateTemplateItem(draft.value.id, {
          name: draft.value.name,
          description: draft.value.description || undefined,
        });
      }
      await fetchItems();
      draft.value = null;
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  async function removeItem(id: number) {
    error.value = null;
    try {
      await deleteTemplateItem(id);
      await fetchItems();
    } catch (e) {
      error.value = String(e);
    }
  }

  return { items, filteredItems, loading, error, draft, isCreating, nameFilter, fetchItems, startCreate, selectItem, saveDraft, removeItem };
}
```

## Page — `src/features/_template/components/TemplateListPage.vue`

Rule: PrimeVue components only for form controls (`InputText`, `Button`, never raw `<input>`); semantic Tailwind tokens (`bg-canvas`, `text-ink`, `text-muted`, `border-border`) instead of raw palette colors.

```vue
<!--
  TEMPLATE — not registered in any route.
  Copy to `src/features/<domain>/components/`, rename `Template`/`template`,
  add a route in `src/app/router/routes.ts`.
-->
<script setup lang="ts">
import { onMounted, ref } from "vue";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { useTemplate } from "../composables/useTemplate";

const ctrl = useTemplate();
const newName = ref("");

onMounted(ctrl.fetchItems);

async function handleAdd() {
  const name = newName.value.trim();
  if (!name) return;
  ctrl.startCreate();
  if (ctrl.draft.value) ctrl.draft.value.name = name;
  if (await ctrl.saveDraft()) newName.value = "";
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <h1 class="text-lg font-semibold text-ink">Template items</h1>

    <div class="flex gap-2">
      <InputText v-model="newName" placeholder="New item name" class="flex-1" @keyup.enter="handleAdd" />
      <Button label="Add" @click="handleAdd" />
    </div>

    <InputText v-model="ctrl.nameFilter.value" placeholder="Filter by name" class="max-w-xs" />

    <p v-if="ctrl.error.value" class="text-sm text-red-500">{{ ctrl.error.value }}</p>
    <p v-else-if="ctrl.loading.value" class="text-sm text-muted">Loading...</p>

    <ul v-else class="flex flex-col gap-1">
      <li
        v-for="item in ctrl.filteredItems.value"
        :key="item.id"
        class="flex items-center justify-between rounded border border-border bg-surface px-3 py-2 text-ink"
      >
        <span>{{ item.name }}</span>
        <Button icon="pi pi-trash" severity="danger" text rounded size="small" @click="ctrl.removeItem(item.id)" />
      </li>
      <li v-if="ctrl.filteredItems.value.length === 0" class="text-sm text-muted">No items yet.</li>
    </ul>
  </div>
</template>
```

See [FEATURE_TEMPLATES.md](FEATURE_TEMPLATES.md) for the unregistered-file rules and when to generate these.
