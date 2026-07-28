<script setup lang="ts">
import { ref } from "vue";
import { explorerReadDir, explorerOpenFile, type FileEntry } from "@/tauri/commands/explorer";
import { friendlyError } from "@/tauri/commands/_base";

const props = defineProps<{ entry: FileEntry; depth: number }>();

const expanded = ref(false);
const loaded = ref(false);
const loading = ref(false);
const children = ref<FileEntry[]>([]);
const error = ref("");

/** Thư mục: mở/thu gọn (nạp con lần đầu khi mở). File: mở bằng ứng dụng mặc định. */
async function toggle() {
  if (!props.entry.is_dir) {
    void explorerOpenFile(props.entry.path).catch(() => undefined);
    return;
  }
  expanded.value = !expanded.value;
  if (!expanded.value || loaded.value) return;

  loading.value = true;
  error.value = "";
  try {
    const result = await explorerReadDir(props.entry.path);
    children.value = result.entries;
    loaded.value = true;
  } catch (e) {
    error.value = friendlyError(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div>
    <button
      type="button"
      class="flex w-full items-center gap-1 rounded px-1 py-0.5 text-left text-xs text-secondary hover:bg-canvas"
      :style="{ paddingLeft: `${depth * 14 + 6}px` }"
      :title="entry.path"
      @click="toggle"
    >
      <i
        v-if="entry.is_dir"
        class="pi shrink-0 text-[9px] text-muted"
        :class="expanded ? 'pi-chevron-down' : 'pi-chevron-right'"
      />
      <span v-else class="inline-block w-[9px] shrink-0" />
      <i class="pi shrink-0 text-[11px]" :class="entry.is_dir ? 'pi-folder text-amber-500' : 'pi-file text-muted'" />
      <span class="truncate">{{ entry.name }}</span>
    </button>

    <div v-if="entry.is_dir && expanded">
      <div v-if="loading" class="px-1 py-1 text-[11px] text-muted" :style="{ paddingLeft: `${(depth + 1) * 14 + 6}px` }">
        <i class="pi pi-spinner pi-spin" />
      </div>
      <div v-else-if="error" class="truncate px-1 py-1 text-[11px] text-red-600" :style="{ paddingLeft: `${(depth + 1) * 14 + 6}px` }" :title="error">
        {{ error }}
      </div>
      <div v-else-if="!children.length" class="px-1 py-1 text-[11px] text-muted" :style="{ paddingLeft: `${(depth + 1) * 14 + 6}px` }">
        (trống)
      </div>
      <TerminalTreeNode v-for="child in children" :key="child.path" :entry="child" :depth="depth + 1" />
    </div>
  </div>
</template>
