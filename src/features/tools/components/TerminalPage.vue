<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import Menu from "primevue/menu";
import type { MenuItem } from "primevue/menuitem";
import { open } from "@tauri-apps/plugin-dialog";
import "@xterm/xterm/css/xterm.css";
import { useTerminal } from "../composables/useTerminal";
import TerminalFileTree from "./TerminalFileTree.vue";
import { useToast } from "@/shared/composables/useToast";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";

/** Các CLI agent có thể khởi chạy trực tiếp trong một tab terminal mới. */
const AGENT_OPTIONS = [
  { title: "Claude", command: "claude", icon: "pi pi-sparkles" },
  { title: "Codex", command: "codex", icon: "pi pi-microchip-ai" },
  { title: "Copilot", command: "copilot", icon: "pi pi-github" },
] as const;

const term = useTerminal();
const toast = useToast();

const activeTab = computed(() => term.tabs.value.find((t) => t.key === term.activeKey.value) ?? null);

// === Drag-to-resize cột cây thư mục (nhớ độ rộng vào localStorage) ===
const TREE_WIDTH_KEY = "terminal.width.tree";
function loadTreeWidth() {
  const raw = Number(localStorage.getItem(TREE_WIDTH_KEY) ?? "");
  return Number.isFinite(raw) && raw > 0 ? Math.max(160, Math.min(480, raw)) : 224;
}
const treeWidth = ref(loadTreeWidth());
const isResizingTree = ref(false);
const viewportRowRef = ref<HTMLElement | null>(null);
let activeTreeResizeMove: ((e: MouseEvent) => void) | null = null;

function startResizeTree(e: MouseEvent) {
  e.preventDefault();
  isResizingTree.value = true;
  const move = (ev: MouseEvent) => {
    const right = viewportRowRef.value?.getBoundingClientRect().right ?? 0;
    treeWidth.value = Math.max(160, Math.min(480, right - ev.clientX));
  };
  activeTreeResizeMove = move;
  document.addEventListener("mousemove", move);
  document.addEventListener("mouseup", endResizeTree);
}

function endResizeTree() {
  isResizingTree.value = false;
  if (activeTreeResizeMove) document.removeEventListener("mousemove", activeTreeResizeMove);
  document.removeEventListener("mouseup", endResizeTree);
  activeTreeResizeMove = null;
  localStorage.setItem(TREE_WIDTH_KEY, String(Math.round(treeWidth.value)));
}

const agentMenuBottom = ref<InstanceType<typeof Menu> | null>(null);

/** Khởi chạy agent ngay trong tab đang active (gõ lệnh vào shell hiện có), không mở tab mới. */
function launchAgent(agent: (typeof AGENT_OPTIONS)[number]) {
  const tab = activeTab.value;
  if (!tab) {
    // Chưa có tab nào — tạo mới để có chỗ chạy agent.
    term.addTab({ title: agent.title, autoCommand: agent.command });
    return;
  }
  term.runCommand(tab.key, agent.command);
}

const agentMenuItems: MenuItem[] = AGENT_OPTIONS.map((agent) => ({
  label: agent.title,
  icon: agent.icon,
  command: () => launchAgent(agent),
}));

function toggleAgentMenuBottom(event: Event) {
  agentMenuBottom.value?.toggle(event);
}

/** Ref callback: gắn xterm vào container khi phần tử DOM của tab được mount. */
function bind(key: string, el: Element | null) {
  if (el instanceof HTMLElement) void term.bindContainer(key, el);
}

async function pickDir() {
  if (!canUseTauriRuntime()) {
    toast.error(tauriRuntimeMessage);
    return;
  }
  const tab = activeTab.value;
  if (!tab) return;
  try {
    const selected = await open({ directory: true, title: "Choose working directory" });
    if (typeof selected === "string") await term.changeTabDir(tab.key, selected);
  } catch (e) {
    toast.error(friendlyError(e));
  }
}

onMounted(() => {
  void term.init();
});

onBeforeUnmount(() => {
  void term.dispose();
  if (activeTreeResizeMove) document.removeEventListener("mousemove", activeTreeResizeMove);
  document.removeEventListener("mouseup", endResizeTree);
});
</script>

<template>
  <div class="flex h-full flex-col gap-2 overflow-hidden">
    <!-- Tab bar -->
    <div class="flex shrink-0 items-center gap-1 rounded-lg border border-divider bg-panel px-2 py-1 text-xs shadow-sm">
      <div
        v-for="tab in term.tabs.value"
        :key="tab.key"
        class="group flex h-7 shrink-0 cursor-pointer items-center gap-2 rounded-md px-2 transition-colors"
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
      <button
        type="button"
        class="flex h-7 shrink-0 items-center justify-center rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand"
        title="New Terminal"
        @click="term.addTab()"
      >
        <i class="pi pi-plus text-[10px]" />
      </button>
    </div>

    <!-- Terminal viewports (giữ tất cả mounted, chỉ hiện tab active) -->
    <div ref="viewportRowRef" class="flex min-h-0 flex-1 overflow-hidden" :class="isResizingTree ? 'select-none' : ''">
      <div class="relative min-w-0 flex-1 overflow-hidden" style="background: #0b0f19">
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

      <template v-if="activeTab?.startDir">
        <!-- Resize handle: terminal | cây thư mục -->
        <div
          class="flex w-2 shrink-0 cursor-col-resize items-center justify-center hover:bg-brand/10"
          :class="isResizingTree ? 'bg-brand/20' : ''"
          @mousedown="startResizeTree"
        >
          <div class="h-8 w-0.5 rounded-full bg-divider" :class="isResizingTree ? 'bg-brand' : ''" />
        </div>

        <!-- Cột cây thư mục: chỉ hiện khi tab đang active đã chọn thư mục làm việc -->
        <TerminalFileTree
          :root="activeTab.startDir"
          :style="{ width: treeWidth + 'px' }"
          class="shrink-0 overflow-hidden border-l border-divider bg-panel"
        />
      </template>
    </div>

    <!-- Bottom toolbar: per-tab working directory -->
    <div
      v-if="activeTab"
      class="flex shrink-0 flex-wrap items-center gap-1 rounded-lg border border-divider bg-panel px-2 py-1 text-xs shadow-sm"
    >
      <button
        type="button"
        class="flex h-7 shrink-0 items-center gap-1.5 rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand"
        :title="activeTab.startDir"
        @click="pickDir"
      >
        <i class="pi pi-folder-open text-[11px]" />
        <span class="font-medium">{{ activeTab.startDir ? "Change Folder" : "Set Folder" }}</span>
      </button>

      <!-- Agent picker: chỉ hiện sau khi đã chọn thư mục làm việc, để khởi chạy agent ngay trong thư mục đó -->
      <template v-if="activeTab.startDir">
        <button
          type="button"
          class="flex h-7 shrink-0 items-center gap-1 rounded-md px-2 text-secondary transition-colors hover:bg-canvas hover:text-brand"
          :title="`Launch agent in ${activeTab.startDir}`"
          aria-haspopup="true"
          @click="toggleAgentMenuBottom"
        >
          <i class="pi pi-microchip-ai text-[11px]" />
          <span class="font-medium">Agent</span>
          <i class="pi pi-angle-down text-[9px]" />
        </button>
        <Menu ref="agentMenuBottom" :model="agentMenuItems" :popup="true" />
      </template>

      <template v-if="activeTab.git">
        <span class="mx-0.5 h-4 w-px shrink-0 bg-divider" />
        <span class="flex h-7 shrink-0 items-center gap-1 px-1 text-secondary" :title="activeTab.git.repoName">
          <i class="pi pi-code text-[11px] text-brand" />
          {{ activeTab.git.repoName }}
        </span>
        <span class="flex h-7 shrink-0 items-center gap-1 px-1 text-secondary">
          <i class="pi pi-share-alt text-[11px] text-brand" />
          {{ activeTab.git.branch }}
        </span>
        <span
          v-if="activeTab.git.diffCount > 0"
          class="flex h-7 shrink-0 items-center rounded-full bg-brand/15 px-2 font-medium text-brand"
        >
          {{ activeTab.git.diffCount }} changed
        </span>
      </template>
    </div>
  </div>
</template>
