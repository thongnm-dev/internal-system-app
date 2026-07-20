<script setup lang="ts">
import { ref, watch } from "vue";
import Button from "primevue/button";
import { useMenuStore } from "@/app/stores/menu";
import type { MenuKey } from "@/_/types/app";

const props = defineProps<{
  activeMenu: MenuKey;
  isCollapsed: boolean;
}>();

const emit = defineEmits<{
  menuChange: [key: MenuKey];
  toggleCollapse: [];
}>();

const menu = useMenuStore();

// menu_configs chỉ lưu label nhóm (menu_group), không có icon nhóm — icon là
// dữ liệu trình bày nên map tĩnh ở đây, có fallback cho nhóm mới.
const GROUP_ICONS: Record<string, string> = {
  Tools: "pi-wrench",
  Cloud: "pi-cloud",
  "AI Agent": "pi-sparkles",
  Governance: "pi-shield",
};
const groupIcon = (label: string) => GROUP_ICONS[label] ?? "pi-folder";

const groupOpen = ref<Record<string, boolean>>({});

function isGroupActive(items: { key: string }[]) {
  return items.some((i) => i.key === props.activeMenu);
}

// Tự mở nhóm chứa menu đang active (kể cả khi menu vừa được nạp từ store).
watch(
  [() => props.activeMenu, () => menu.groups],
  ([key]) => {
    for (const g of menu.groups) {
      if (g.items.some((i) => i.key === key)) groupOpen.value[g.label] = true;
    }
  },
  { immediate: true },
);

function tooltipOpts(label: string) {
  return { value: label, disabled: !props.isCollapsed, showDelay: 300 };
}
</script>

<template>
  <aside class="flex min-h-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar text-sidebar-text">
    <div :class="['border-b border-sidebar-border', isCollapsed ? 'p-3' : 'p-5']">
      <div :class="['flex items-center gap-3', isCollapsed ? 'justify-center' : 'justify-between']">
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-brand font-bold text-white">PJ</div>
        <Button
          v-if="!isCollapsed"
          icon="pi pi-chevron-left"
          text
          rounded
          size="small"
          class="shrink-0 text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active"
          title="Collapse sidebar"
          @click="emit('toggleCollapse')"
        />
      </div>

      <template v-if="isCollapsed">
        <Button
          icon="pi pi-chevron-right"
          text
          class="mt-3 w-full text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active"
          title="Expand sidebar"
          @click="emit('toggleCollapse')"
        />
      </template>
      <template v-else>
        <h1 class="mt-4 text-xl font-bold leading-tight text-sidebar-title">Manager System</h1>
      </template>
    </div>

    <nav :class="['flex-1 space-y-1 overflow-y-auto overflow-x-hidden', isCollapsed ? 'p-2' : 'p-3']">
      <Button
        v-for="item in menu.topLevelItems"
        :key="item.key"
        v-tooltip.right="tooltipOpts(item.title)"
        :class="[
          'flex h-10 w-full items-center rounded-md text-sm font-semibold transition',
          isCollapsed ? 'justify-center px-0' : 'gap-3 px-3 text-left',
          activeMenu === item.key
            ? 'bg-sidebar-active text-sidebar-text-active'
            : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
        ]"
        unstyled
        @click="emit('menuChange', item.key as MenuKey)"
      >
        <i :class="`pi ${item.icon} shrink-0`" />
        <span v-if="!isCollapsed">{{ item.title }}</span>
      </Button>

      <!-- Collapsible groups -->
      <template v-for="g in menu.groups" :key="g.label">
        <Button
          v-if="isCollapsed"
          v-tooltip.right="tooltipOpts(g.label)"
          :class="[
            'flex h-10 w-full items-center justify-center rounded-md text-sm font-semibold transition',
            isGroupActive(g.items)
              ? 'bg-sidebar-active text-sidebar-text-active'
              : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
          ]"
          unstyled
          @click="groupOpen[g.label] = !groupOpen[g.label]"
        >
          <i :class="`pi ${groupIcon(g.label)} shrink-0`" />
        </Button>

        <Button
          v-if="!isCollapsed"
          :class="[
            'flex h-10 w-full items-center gap-3 rounded-md px-3 text-left text-sm font-semibold transition',
            isGroupActive(g.items)
              ? 'text-sidebar-text-active'
              : 'text-sidebar-text hover:text-sidebar-text-active',
          ]"
          unstyled
          @click="groupOpen[g.label] = !groupOpen[g.label]"
        >
          <i :class="`pi ${groupIcon(g.label)} shrink-0`" />
          <span class="flex-1">{{ g.label }}</span>
          <i :class="['pi shrink-0 text-xs transition-transform', groupOpen[g.label] ? 'pi-chevron-down' : 'pi-chevron-right']" />
        </Button>

        <template v-if="groupOpen[g.label]">
          <Button
            v-for="child in g.items"
            :key="child.key"
            v-tooltip.right="tooltipOpts(child.title)"
            :class="[
              'flex h-9 w-full items-center rounded-md text-sm font-medium transition',
              isCollapsed ? 'justify-center px-0' : 'gap-3 pl-9 pr-3 text-left',
              activeMenu === child.key
                ? 'bg-sidebar-active text-sidebar-text-active'
                : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
            ]"
            unstyled
            @click="emit('menuChange', child.key as MenuKey)"
          >
            <i :class="`pi ${child.icon} shrink-0 text-xs`" />
            <span v-if="!isCollapsed">{{ child.title }}</span>
          </Button>
        </template>
      </template>
    </nav>

    <div v-if="menu.settingsMenu" :class="['border-t border-sidebar-border text-sm text-sidebar-text', isCollapsed ? 'p-2' : 'p-4']">
      <Button
        v-tooltip.right="tooltipOpts(menu.settingsMenu.title)"
        :class="[
          'flex h-10 w-full items-center rounded-md text-sm font-semibold transition',
          isCollapsed ? 'justify-center px-0' : 'gap-3 px-3 text-left',
          activeMenu === menu.settingsMenu.key
            ? 'bg-sidebar-active text-sidebar-text-active'
            : 'text-sidebar-text hover:bg-sidebar-hover hover:text-sidebar-text-active',
        ]"
        unstyled
        @click="emit('menuChange', (menu.settingsMenu?.key ?? 'settings') as MenuKey)"
      >
        <i :class="`pi ${menu.settingsMenu.icon} shrink-0`" />
        <span v-if="!isCollapsed">{{ menu.settingsMenu.title }}</span>
      </Button>
    </div>
  </aside>
</template>
