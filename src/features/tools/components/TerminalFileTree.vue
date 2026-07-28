<script setup lang="ts">
import { ref, watch } from "vue";
import { explorerReadDir, type FileEntry } from "@/tauri/commands/explorer";
import { friendlyError } from "@/tauri/commands/_base";
import TerminalTreeNode from "./TerminalTreeNode.vue";

const props = defineProps<{ root: string }>();

const entries = ref<FileEntry[]>([]);
const loading = ref(false);
const error = ref("");

async function load(path: string) {
  if (!path) {
    entries.value = [];
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    const result = await explorerReadDir(path);
    entries.value = result.entries;
  } catch (e) {
    error.value = friendlyError(e);
  } finally {
    loading.value = false;
  }
}

watch(() => props.root, load, { immediate: true });
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <div class="shrink-0 truncate px-2 py-1.5 text-[11px] font-semibold uppercase tracking-wide text-muted" :title="root">
      Explorer
    </div>
    <div class="min-h-0 flex-1 overflow-y-auto py-1">
      <div v-if="loading" class="flex items-center gap-2 px-2 py-2 text-xs text-muted">
        <i class="pi pi-spinner pi-spin" /> Đang tải…
      </div>
      <div v-else-if="error" class="px-2 py-2 text-xs text-red-600">{{ error }}</div>
      <div v-else-if="!entries.length" class="px-2 py-2 text-xs text-muted">Thư mục trống.</div>
      <TerminalTreeNode v-for="entry in entries" :key="entry.path" :entry="entry" :depth="0" />
    </div>
  </div>
</template>
