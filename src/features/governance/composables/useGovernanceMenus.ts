import { ref, computed } from "vue";
import type { MenuConfig } from "@/_/types/menu-config";
import { listMenuConfigs, saveMenuConfig, saveAllMenuConfigs } from "@/tauri/commands/menu-config";
import { canUseTauriRuntime } from "@/tauri/commands/_base";

export type MenuItemConfig = MenuConfig;

export function useGovernanceMenus() {
  const items = ref<MenuItemConfig[]>([]);
  const editingKey = ref<string | null>(null);
  const draft = ref<MenuItemConfig | null>(null);
  const filterGroup = ref<string>("All");
  const searchQuery = ref("");
  const loading = ref(false);
  const error = ref<string | null>(null);

  const groups = computed(() => {
    const set = new Set(items.value.map((i) => i.group));
    return ["All", ...Array.from(set).sort()];
  });

  const filteredItems = computed(() => {
    let list = [...items.value].sort((a, b) => a.order - b.order);
    if (filterGroup.value !== "All") {
      list = list.filter((i) => i.group === filterGroup.value);
    }
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      list = list.filter(
        (i) =>
          i.title.toLowerCase().includes(q) ||
          i.key.toLowerCase().includes(q) ||
          i.path.toLowerCase().includes(q),
      );
    }
    return list;
  });

  const stats = computed(() => ({
    total: items.value.length,
    visible: items.value.filter((i) => i.visible).length,
    hidden: items.value.filter((i) => !i.visible).length,
  }));

  async function fetchItems() {
    loading.value = true;
    error.value = null;
    try {
      if (canUseTauriRuntime()) {
        items.value = await listMenuConfigs();
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function persistItem(item: MenuItemConfig) {
    if (!canUseTauriRuntime()) return;
    try {
      await saveMenuConfig(item);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function persistAll() {
    if (!canUseTauriRuntime()) return;
    try {
      await saveAllMenuConfigs({ items: items.value });
    } catch (e) {
      error.value = String(e);
    }
  }

  function selectItem(key: string) {
    editingKey.value = key;
    const item = items.value.find((i) => i.key === key);
    draft.value = item ? { ...item } : null;
  }

  function updateDraft<K extends keyof MenuItemConfig>(field: K, value: MenuItemConfig[K]) {
    if (draft.value) draft.value[field] = value;
  }

  async function saveDraft() {
    if (!draft.value || !editingKey.value) return;
    const idx = items.value.findIndex((i) => i.key === editingKey.value);
    if (idx >= 0) {
      items.value[idx] = { ...draft.value };
      await persistItem(items.value[idx]);
    }
  }

  function resetDraft() {
    if (editingKey.value) selectItem(editingKey.value);
  }

  async function toggleVisibility(key: string) {
    const item = items.value.find((i) => i.key === key);
    if (item) {
      item.visible = !item.visible;
      await persistItem(item);
      if (editingKey.value === key && draft.value) {
        draft.value.visible = item.visible;
      }
    }
  }

  async function moveUp(key: string) {
    const sorted = [...items.value].sort((a, b) => a.order - b.order);
    const idx = sorted.findIndex((i) => i.key === key);
    if (idx <= 0) return;
    const prevOrder = sorted[idx - 1].order;
    sorted[idx - 1].order = sorted[idx].order;
    sorted[idx].order = prevOrder;
    items.value = sorted;
    await persistAll();
  }

  async function moveDown(key: string) {
    const sorted = [...items.value].sort((a, b) => a.order - b.order);
    const idx = sorted.findIndex((i) => i.key === key);
    if (idx < 0 || idx >= sorted.length - 1) return;
    const nextOrder = sorted[idx + 1].order;
    sorted[idx + 1].order = sorted[idx].order;
    sorted[idx].order = nextOrder;
    items.value = sorted;
    await persistAll();
  }

  async function resetToDefault() {
    editingKey.value = null;
    draft.value = null;
    await fetchItems();
  }

  fetchItems();

  return {
    items,
    filteredItems,
    editingKey,
    draft,
    filterGroup,
    searchQuery,
    groups,
    stats,
    loading,
    error,
    fetchItems,
    selectItem,
    updateDraft,
    saveDraft,
    resetDraft,
    toggleVisibility,
    moveUp,
    moveDown,
    resetToDefault,
  };
}
