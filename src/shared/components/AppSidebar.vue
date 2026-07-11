<script setup lang="ts">
import { appRoutes } from "@/app/router/routes";
import type { MenuKey } from "@/shared/types/app";

defineProps<{
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
  { id: "projectSkills", icon: "pi-book" },
  { id: "issueBacklog", icon: "pi-list-check" },
  { id: "excel2md", icon: "pi-file" },
  { id: "dailyWorkNotes", icon: "pi-pencil" },
  { id: "dailyReport", icon: "pi-calendar" },
  { id: "importCsv", icon: "pi-database" },
];

const settingsRoute = appRoutes.find((r) => r.key === "settings");
const settingsLabel = settingsRoute?.title ?? "Settings";

function labelFor(id: MenuKey): string {
  return appRoutes.find((r) => r.key === id)?.title ?? id;
}
</script>

<template>
  <aside class="flex min-h-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-text">
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

    <nav :class="['flex-1 space-y-1', isCollapsed ? 'p-2' : 'p-3']">
      <div v-for="item in items" :key="item.id" class="group relative">
        <button
          :class="[
            'flex h-10 w-full items-center rounded-md text-sm font-semibold transition',
            isCollapsed ? 'justify-center px-0' : 'gap-3 px-3 text-left',
            activeMenu === item.id
              ? 'bg-sidebar-active text-sidebar-text-active'
              : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
          ]"
          type="button"
          :title="isCollapsed ? undefined : labelFor(item.id)"
          @click="emit('menuChange', item.id)"
        >
          <i :class="`pi ${item.icon} shrink-0`" />
          <span v-if="!isCollapsed">{{ labelFor(item.id) }}</span>
        </button>
        <span
          v-if="isCollapsed"
          class="pointer-events-none absolute left-[calc(100%+8px)] top-1/2 z-50 -translate-y-1/2 whitespace-nowrap rounded-md bg-bar px-2.5 py-1.5 text-xs font-semibold text-bar-strong opacity-0 shadow-float transition-opacity group-hover:opacity-100"
        >
          {{ labelFor(item.id) }}
        </span>
      </div>
    </nav>

    <div :class="['border-t border-sidebar-border text-sm text-sidebar-text', isCollapsed ? 'p-2' : 'p-4']">
      <div class="group relative">
        <button
          :class="[
            'flex h-10 w-full items-center rounded-md text-sm font-semibold transition',
            isCollapsed ? 'justify-center px-0' : 'gap-3 px-3 text-left',
            activeMenu === 'settings'
              ? 'bg-sidebar-active text-sidebar-text-active'
              : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
          ]"
          type="button"
          :title="isCollapsed ? undefined : settingsLabel"
          @click="emit('menuChange', 'settings')"
        >
          <i class="pi pi-cog shrink-0" />
          <span v-if="!isCollapsed">{{ settingsLabel }}</span>
        </button>
        <span
          v-if="isCollapsed"
          class="pointer-events-none absolute left-[calc(100%+12px)] top-1/2 z-50 -translate-y-1/2 whitespace-nowrap rounded-md bg-bar px-2.5 py-1.5 text-xs font-semibold text-bar-strong opacity-0 shadow-float transition-opacity group-hover:opacity-100"
        >
          {{ settingsLabel }}
        </span>
      </div>
    </div>
  </aside>
</template>
