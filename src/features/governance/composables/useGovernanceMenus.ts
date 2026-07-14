import { ref, computed } from "vue";

export type MenuItemConfig = {
  key: string;
  title: string;
  path: string;
  icon: string;
  group: string;
  visible: boolean;
  order: number;
};

const STORAGE_KEY = "pjjyuji.governance.menus";

function defaultMenus(): MenuItemConfig[] {
  return [
    { key: "overview", title: "Overview", path: "/overview", icon: "pi-home", group: "—", visible: true, order: 0 },
    { key: "projects", title: "Projects", path: "/projects", icon: "pi-table", group: "—", visible: true, order: 1 },
    { key: "issueBacklog", title: "Issue Backlog", path: "/issue-backlog", icon: "pi-list-check", group: "—", visible: true, order: 2 },
    { key: "dailyWorkNotes", title: "Daily Work Notes", path: "/daily-work-notes", icon: "pi-pencil", group: "—", visible: true, order: 3 },
    { key: "dailyReport", title: "Daily Report", path: "/daily-report", icon: "pi-calendar", group: "—", visible: true, order: 4 },
    { key: "excel2md", title: "Excel to Markdown", path: "/excel2md", icon: "pi-file", group: "Tools", visible: true, order: 5 },
    { key: "importCsv", title: "Import CSV", path: "/import-csv", icon: "pi-database", group: "Tools", visible: true, order: 6 },
    { key: "cloudS3", title: "S3 Browser", path: "/cloud/s3", icon: "pi-folder-open", group: "Cloud", visible: true, order: 7 },
    { key: "aiChat", title: "AI Chat", path: "/ai/chat", icon: "pi-comments", group: "AI Agent", visible: true, order: 8 },
    { key: "projectSkills", title: "Skills", path: "/project-skills", icon: "pi-book", group: "Governance", visible: true, order: 9 },
    { key: "governanceMenus", title: "Menus", path: "/governance/menus", icon: "pi-bars", group: "Governance", visible: true, order: 10 },
    { key: "governanceUsers", title: "Users", path: "/governance/users", icon: "pi-users", group: "Governance", visible: true, order: 11 },
    { key: "governanceLogs", title: "Logs", path: "/governance/logs", icon: "pi-history", group: "Governance", visible: true, order: 12 },
    { key: "settings", title: "Settings", path: "/settings", icon: "pi-cog", group: "—", visible: true, order: 13 },
  ];
}

function load(): MenuItemConfig[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw);
  } catch { /* ignore */ }
  return defaultMenus();
}

function save(items: MenuItemConfig[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
}

export function useGovernanceMenus() {
  const items = ref<MenuItemConfig[]>(load());
  const editingKey = ref<string | null>(null);
  const draft = ref<MenuItemConfig | null>(null);
  const filterGroup = ref<string>("All");
  const searchQuery = ref("");

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

  function selectItem(key: string) {
    editingKey.value = key;
    const item = items.value.find((i) => i.key === key);
    draft.value = item ? { ...item } : null;
  }

  function updateDraft<K extends keyof MenuItemConfig>(field: K, value: MenuItemConfig[K]) {
    if (draft.value) draft.value[field] = value;
  }

  function saveDraft() {
    if (!draft.value || !editingKey.value) return;
    const idx = items.value.findIndex((i) => i.key === editingKey.value);
    if (idx >= 0) {
      items.value[idx] = { ...draft.value };
      save(items.value);
    }
  }

  function resetDraft() {
    if (editingKey.value) selectItem(editingKey.value);
  }

  function toggleVisibility(key: string) {
    const item = items.value.find((i) => i.key === key);
    if (item) {
      item.visible = !item.visible;
      save(items.value);
      if (editingKey.value === key && draft.value) {
        draft.value.visible = item.visible;
      }
    }
  }

  function moveUp(key: string) {
    const sorted = [...items.value].sort((a, b) => a.order - b.order);
    const idx = sorted.findIndex((i) => i.key === key);
    if (idx <= 0) return;
    const prevOrder = sorted[idx - 1].order;
    sorted[idx - 1].order = sorted[idx].order;
    sorted[idx].order = prevOrder;
    items.value = sorted;
    save(items.value);
  }

  function moveDown(key: string) {
    const sorted = [...items.value].sort((a, b) => a.order - b.order);
    const idx = sorted.findIndex((i) => i.key === key);
    if (idx < 0 || idx >= sorted.length - 1) return;
    const nextOrder = sorted[idx + 1].order;
    sorted[idx + 1].order = sorted[idx].order;
    sorted[idx].order = nextOrder;
    items.value = sorted;
    save(items.value);
  }

  function resetToDefault() {
    items.value = defaultMenus();
    editingKey.value = null;
    draft.value = null;
    save(items.value);
  }

  return {
    items,
    filteredItems,
    editingKey,
    draft,
    filterGroup,
    searchQuery,
    groups,
    stats,
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
