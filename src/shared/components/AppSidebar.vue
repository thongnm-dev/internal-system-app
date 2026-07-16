<script setup lang="ts">
import { ref, watch } from "vue";
import { appRoutes } from "@/app/router/routes";
import type { MenuKey } from "@/_/types/app";

const props = defineProps<{
  activeMenu: MenuKey;
  isCollapsed: boolean;
}>();

const emit = defineEmits<{
  menuChange: [key: MenuKey];
  toggleCollapse: [];
}>();

const items: { id: MenuKey; icon: string }[] = [
  { id: "overview", icon: "pi-home" },
  { id: "projects", icon: "pi-table" },
  { id: "issueBacklog", icon: "pi-list-check" },
  { id: "dailyWorkNotes", icon: "pi-pencil" },
  { id: "dailyReport", icon: "pi-calendar" },
];

type MenuGroup = {
  label: string;
  icon: string;
  children: { id: MenuKey; icon: string }[];
};

const groups: MenuGroup[] = [
  {
    label: "Tools",
    icon: "pi-wrench",
    children: [
      { id: "excel2md", icon: "pi-file" },
      { id: "copyTools", icon: "pi-copy" },
      { id: "importCsv", icon: "pi-database" },
      { id: "sqlEditor", icon: "pi-server" },
      { id: "exploreFaster", icon: "pi-compass" },
    ],
  },
  {
    label: "Cloud",
    icon: "pi-cloud",
    children: [
      { id: "cloudS3", icon: "pi-folder-open" },
      { id: "cloudS3Upload", icon: "pi-upload" },
      { id: "cloudS3Download", icon: "pi-download" },
    ],
  },
  {
    label: "AI Agent",
    icon: "pi-sparkles",
    children: [
      { id: "aiChat", icon: "pi-comments" },
      { id: "aiUsage", icon: "pi-chart-bar" },
    ],
  },
  {
    label: "Governance",
    icon: "pi-shield",
    children: [
      { id: "projectSkills", icon: "pi-book" },
      { id: "governanceMenus", icon: "pi-bars" },
      { id: "governanceUsers", icon: "pi-users" },
      { id: "governanceLogs", icon: "pi-history" },
    ],
  },
];

const groupKeySet = new Map<string, Set<MenuKey>>(
  groups.map((g) => [g.label, new Set(g.children.map((c) => c.id))]),
);

const groupOpen = ref<Record<string, boolean>>(
  Object.fromEntries(groups.map((g) => [g.label, false])),
);

watch(
  () => props.activeMenu,
  (key) => {
    for (const g of groups) {
      if (groupKeySet.get(g.label)!.has(key)) groupOpen.value[g.label] = true;
    }
  },
  { immediate: true },
);

const settingsRoute = appRoutes.find((r) => r.key === "settings");
const settingsLabel = settingsRoute?.title ?? "Settings";

function labelFor(id: MenuKey): string {
  return appRoutes.find((r) => r.key === id)?.title ?? id;
}

function tooltipOpts(label: string) {
  return { value: label, disabled: !props.isCollapsed, showDelay: 300 };
}
</script>

<template>
  <aside class="flex min-h-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar text-sidebar-text">
    <div :class="['border-b border-sidebar-border', isCollapsed ? 'p-3' : 'p-5']">
      <div :class="['flex items-center gap-3', isCollapsed ? 'justify-center' : 'justify-between']">
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-brand font-bold text-white">PJ</div>
        <button
          v-if="!isCollapsed"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active"
          type="button"
          title="Collapse sidebar"
          @click="emit('toggleCollapse')"
        >
          <i class="pi pi-chevron-left" />
        </button>
      </div>

      <template v-if="isCollapsed">
        <button
          class="mt-3 flex h-9 w-full items-center justify-center rounded-md text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active"
          type="button"
          title="Expand sidebar"
          @click="emit('toggleCollapse')"
        >
          <i class="pi pi-chevron-right" />
        </button>
      </template>
      <template v-else>
        <h1 class="mt-4 text-xl font-bold leading-tight text-sidebar-title">Manager System</h1>
      </template>
    </div>

    <nav :class="['flex-1 space-y-1 overflow-y-auto overflow-x-hidden', isCollapsed ? 'p-2' : 'p-3']">
      <button
        v-for="item in items"
        :key="item.id"
        v-tooltip.right="tooltipOpts(labelFor(item.id))"
        :class="[
          'flex h-10 w-full items-center rounded-md text-sm font-semibold transition',
          isCollapsed ? 'justify-center px-0' : 'gap-3 px-3 text-left',
          activeMenu === item.id
            ? 'bg-sidebar-active text-sidebar-text-active'
            : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
        ]"
        type="button"
        @click="emit('menuChange', item.id)"
      >
        <i :class="`pi ${item.icon} shrink-0`" />
        <span v-if="!isCollapsed">{{ labelFor(item.id) }}</span>
      </button>

      <!-- Collapsible groups -->
      <template v-for="g in groups" :key="g.label">
        <button
          v-if="isCollapsed"
          v-tooltip.right="tooltipOpts(g.label)"
          :class="[
            'flex h-10 w-full items-center justify-center rounded-md text-sm font-semibold transition',
            groupKeySet.get(g.label)!.has(activeMenu)
              ? 'bg-sidebar-active text-sidebar-text-active'
              : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
          ]"
          type="button"
          @click="groupOpen[g.label] = !groupOpen[g.label]"
        >
          <i :class="`pi ${g.icon} shrink-0`" />
        </button>

        <button
          v-if="!isCollapsed"
          class="flex h-10 w-full items-center gap-3 rounded-md px-3 text-left text-sm font-semibold transition"
          :class="[
            groupKeySet.get(g.label)!.has(activeMenu)
              ? 'text-sidebar-text-active'
              : 'text-sidebar-text hover:text-sidebar-text-active',
          ]"
          type="button"
          @click="groupOpen[g.label] = !groupOpen[g.label]"
        >
          <i :class="`pi ${g.icon} shrink-0`" />
          <span class="flex-1">{{ g.label }}</span>
          <i :class="['pi shrink-0 text-xs transition-transform', groupOpen[g.label] ? 'pi-chevron-down' : 'pi-chevron-right']" />
        </button>

        <template v-if="groupOpen[g.label]">
          <button
            v-for="child in g.children"
            :key="child.id"
            v-tooltip.right="tooltipOpts(labelFor(child.id))"
            :class="[
              'flex h-9 w-full items-center rounded-md text-sm font-medium transition',
              isCollapsed ? 'justify-center px-0' : 'gap-3 pl-9 pr-3 text-left',
              activeMenu === child.id
                ? 'bg-sidebar-active text-sidebar-text-active'
                : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
            ]"
            type="button"
            @click="emit('menuChange', child.id)"
          >
            <i :class="`pi ${child.icon} shrink-0 text-xs`" />
            <span v-if="!isCollapsed">{{ labelFor(child.id) }}</span>
          </button>
        </template>
      </template>
    </nav>

    <div :class="['border-t border-sidebar-border text-sm text-sidebar-text', isCollapsed ? 'p-2' : 'p-4']">
      <button
        v-tooltip.right="tooltipOpts(settingsLabel)"
        :class="[
          'flex h-10 w-full items-center rounded-md text-sm font-semibold transition',
          isCollapsed ? 'justify-center px-0' : 'gap-3 px-3 text-left',
          activeMenu === 'settings'
            ? 'bg-sidebar-active text-sidebar-text-active'
            : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
        ]"
        type="button"
        @click="emit('menuChange', 'settings')"
      >
        <i class="pi pi-cog shrink-0" />
        <span v-if="!isCollapsed">{{ settingsLabel }}</span>
      </button>
    </div>
  </aside>
</template>
