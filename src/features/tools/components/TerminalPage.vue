<script setup lang="ts">
import { onBeforeUnmount, onMounted } from "vue";
import Button from "primevue/button";
import { open } from "@tauri-apps/plugin-dialog";
import "@xterm/xterm/css/xterm.css";
import { useTerminal } from "../composables/useTerminal";
import { useToast } from "@/shared/composables/useToast";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";

const term = useTerminal();
const toast = useToast();

/** Ref callback: gắn xterm vào container khi phần tử DOM của tab được mount. */
function bind(key: string, el: Element | null) {
  if (el instanceof HTMLElement) void term.bindContainer(key, el);
}

async function pickDir() {
  if (!canUseTauriRuntime()) {
    toast.error(tauriRuntimeMessage);
    return;
  }
  try {
    const selected = await open({ directory: true, title: "Choose working directory" });
    if (typeof selected === "string") term.setStartDir(selected);
  } catch (e) {
    toast.error(friendlyError(e));
  }
}

onMounted(() => {
  void term.init();
});

onBeforeUnmount(() => {
  void term.dispose();
});
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <!-- Toolbar -->
    <div class="flex flex-wrap items-center gap-2 border-b border-divider px-3 py-2">
      <Button size="small" icon="pi pi-plus" label="New Terminal" @click="term.addTab()" />
      <Button
        size="small"
        severity="secondary"
        outlined
        icon="pi pi-folder-open"
        :label="term.startDir.value ? 'Change Folder' : 'Set Folder'"
        @click="pickDir"
      />
      <span v-if="term.startDir.value" class="truncate text-xs text-muted" :title="term.startDir.value">
        {{ term.startDir.value }}
      </span>
      <span class="ml-auto text-xs text-muted">
        New tabs open a shell in the selected folder (or your home directory).
      </span>
    </div>

    <!-- Tab bar -->
    <div v-if="term.tabs.value.length" class="flex items-center gap-1 overflow-x-auto border-b border-divider bg-panel px-2 py-1">
      <div
        v-for="tab in term.tabs.value"
        :key="tab.key"
        class="group flex cursor-pointer items-center gap-2 rounded-md px-3 py-1 text-xs"
        :class="tab.key === term.activeKey.value ? 'bg-brand/15 text-ink' : 'text-secondary hover:bg-canvas'"
        @click="term.setActive(tab.key)"
      >
        <i class="pi pi-desktop text-[10px]" />
        <span>{{ tab.title }}</span>
        <span v-if="tab.exited" class="text-[10px] text-muted">(exited)</span>
        <i
          class="pi pi-times text-[10px] opacity-50 hover:opacity-100"
          @click.stop="term.closeTab(tab.key)"
        />
      </div>
    </div>

    <!-- Terminal viewports (giữ tất cả mounted, chỉ hiện tab active) -->
    <div class="relative flex-1 overflow-hidden" style="background: #0b0f19">
      <template v-if="term.tabs.value.length">
        <div
          v-for="tab in term.tabs.value"
          v-show="tab.key === term.activeKey.value"
          :key="tab.key"
          class="absolute inset-0 p-2"
        >
          <div :ref="(el) => bind(tab.key, el as Element | null)" class="h-full w-full" />
        </div>
      </template>
      <div v-else class="flex h-full items-center justify-center text-sm text-muted">
        No terminal open. Click "New Terminal" to start.
      </div>
    </div>
  </div>
</template>
