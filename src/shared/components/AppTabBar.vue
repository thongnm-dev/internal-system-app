<script setup lang="ts">
import type { TabItem } from "@/shared/composables/useTabNavigation";

defineProps<{
  tabs: TabItem[];
  activeKey: string | null;
}>();

const emit = defineEmits<{
  activate: [key: string];
  close: [key: string];
}>();
</script>

<template>
  <div
    v-if="tabs.length > 0"
    class="flex items-center gap-1 border-b border-divider-light pb-2"
  >
    <div
      v-for="tab in tabs"
      :key="tab.key"
      :class="[
        'group flex shrink-0 cursor-pointer select-none items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-semibold transition',
        activeKey === tab.key
          ? 'bg-brand/10 text-brand'
          : 'text-muted hover:bg-panel hover:text-secondary',
      ]"
      @click="emit('activate', tab.key)"
    >
      <span class="whitespace-nowrap">{{ tab.title }}</span>
      <button
        v-if="tabs.length > 1"
        :class="[
          'flex h-4 w-4 items-center justify-center rounded-full text-[10px] transition',
          activeKey === tab.key
            ? 'opacity-60 hover:bg-red-500/20 hover:text-red-600 hover:opacity-100'
            : 'opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:bg-red-500/20 hover:text-red-600',
        ]"
        title="Close tab"
        @click.stop="emit('close', tab.key)"
      >
        <i class="pi pi-times" />
      </button>
    </div>
  </div>
</template>
