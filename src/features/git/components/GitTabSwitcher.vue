<script setup lang="ts">
defineProps<{
  activeTab: "changes" | "history";
  changesCount: number;
  historyCount: number;
  refreshing: boolean;
}>();

defineEmits<{
  switchTab: [tab: "changes" | "history"];
  refresh: [];
}>();
</script>

<template>
  <div class="flex items-center gap-1 border-b border-divider p-1">
    <button
      class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors"
      :class="activeTab === 'changes' ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
      @click="$emit('switchTab', 'changes')"
    >
      <i class="pi pi-pencil text-[11px]" /> Changes
      <span
        v-if="changesCount"
        class="rounded-full px-1.5 text-[10px] font-bold"
        :class="activeTab === 'changes' ? 'bg-white/25' : 'bg-canvas text-secondary'"
      >
        {{ changesCount }}
      </span>
    </button>
    <button
      class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors"
      :class="activeTab === 'history' ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
      @click="$emit('switchTab', 'history')"
    >
      <i class="pi pi-history text-[11px]" /> History
      <span
        v-if="historyCount"
        class="rounded-full px-1.5 text-[10px] font-bold"
        :class="activeTab === 'history' ? 'bg-white/25' : 'bg-canvas text-secondary'"
      >
        {{ historyCount }}
      </span>
    </button>
    <button
      class="rounded p-1 text-muted transition-colors hover:bg-canvas hover:text-brand"
      title="Làm mới"
      @click="$emit('refresh')"
    >
      <i v-if="refreshing" class="pi pi-spinner pi-spin text-[11px]" />
      <i v-else class="pi pi-sync text-[11px]" />
    </button>
  </div>
</template>
