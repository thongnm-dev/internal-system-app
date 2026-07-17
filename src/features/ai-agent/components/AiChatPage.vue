<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from "vue";
import Button from "primevue/button";
import Select from "primevue/select";
import { useAiChat, type ProviderId } from "../composables/useAiChat";

const {
  providers,
  sessions,
  activeId,
  providerId,
  model,
  draft,
  sending,
  messages,
  availableModels,
  canSend,
  createSession,
  openSession,
  deleteSession,
  selectProvider,
  selectModel,
  send,
} = useAiChat();

// Keep the selects visually consistent with the panel background and small font.
const selectPt = {
  root: { class: "!bg-panel !border-divider" },
  label: { class: "!text-xs !py-1.5 !text-ink" },
  option: { class: "!text-xs" },
};

const scrollRef = ref<HTMLElement | null>(null);

watch(
  () => messages.value.length,
  async () => {
    await nextTick();
    if (scrollRef.value) scrollRef.value.scrollTop = scrollRef.value.scrollHeight;
  },
);

// Resizable / collapsible history sidebar.
const SIDEBAR_MIN = 180;
const SIDEBAR_MAX = 420;
const sidebarWidth = ref(260);
const sidebarCollapsed = ref(false);
let cleanupDrag: (() => void) | null = null;

function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
}

function startDrag(event: MouseEvent) {
  event.preventDefault();
  const startX = event.clientX;
  const startWidth = sidebarWidth.value;
  function onMove(ev: MouseEvent) {
    const next = startWidth + (ev.clientX - startX);
    sidebarWidth.value = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, next));
  }
  function onUp() {
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
    document.body.style.userSelect = "";
    cleanupDrag = null;
  }
  document.body.style.userSelect = "none";
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
  cleanupDrag = onUp;
}

onBeforeUnmount(() => cleanupDrag?.());

function onProvider(id: ProviderId) {
  selectProvider(id);
}

function onEnter(event: KeyboardEvent) {
  // Enter sends, Shift+Enter inserts a newline.
  if (event.shiftKey) return;
  event.preventDefault();
  send();
}

function formatTime(ts: number) {
  return new Date(ts).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

function formatDate(ts: number) {
  return new Date(ts).toLocaleDateString([], { month: "short", day: "numeric" });
}
</script>

<template>
  <div class="flex flex-1 gap-3 overflow-hidden">
    <!-- History sidebar -->
    <aside
      v-if="!sidebarCollapsed"
      class="flex shrink-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm"
      :style="{ width: sidebarWidth + 'px' }"
    >
      <div class="flex items-center gap-2 border-b border-divider p-3">
        <Button icon="pi pi-angle-double-left" text rounded size="small" title="Collapse history" @click="toggleSidebar" />
        <span class="text-sm font-semibold text-ink">Chat history</span>
        <Button icon="pi pi-plus" size="small" class="ml-auto" title="New chat" @click="createSession" />
      </div>

      <div class="flex-1 space-y-1 overflow-auto p-2">
        <p v-if="sessions.length === 0" class="px-2 py-4 text-center text-xs text-muted">
          No chats yet. Start a new conversation.
        </p>

        <Button
          v-for="s in sessions"
          :key="s.id"
          class="group flex w-full items-center gap-2 rounded-md px-2 py-2 text-left transition-colors"
          :class="s.id === activeId ? 'bg-brand/10 text-brand' : 'text-secondary hover:bg-canvas'"
          unstyled
          @click="openSession(s.id)"
        >
          <i class="pi pi-comment shrink-0 text-xs" />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-xs font-bold">{{ s.title }}</span>
            <span class="block truncate text-[10px] text-muted">{{ formatDate(s.updatedAt) }} · {{ s.messages.length }} msgs</span>
          </span>
          <i
            class="pi pi-trash shrink-0 text-xs text-muted opacity-0 hover:text-red-500 group-hover:opacity-100"
            title="Delete chat"
            @click.stop="deleteSession(s.id)"
          />
        </Button>
      </div>
    </aside>

    <!-- Drag handle -->
    <div
      v-if="!sidebarCollapsed"
      class="group flex w-1.5 shrink-0 cursor-col-resize items-center justify-center"
      title="Drag to resize"
      @mousedown="startDrag"
    >
      <div class="h-10 w-1 rounded-full bg-divider transition-colors group-hover:bg-brand" />
    </div>

    <!-- Chat panel -->
    <div class="relative flex min-h-0 min-w-0 flex-1 flex-col rounded-lg border border-divider bg-panel shadow-sm">
      <!-- Toggle to expand history when collapsed -->
      <Button
        v-if="sidebarCollapsed"
        icon="pi pi-angle-double-right"
        severity="secondary"
        outlined
        size="small"
        class="absolute left-3 top-3 z-10"
        title="Show history"
        @click="toggleSidebar"
      />

      <!-- Messages -->
      <div ref="scrollRef" class="flex-1 space-y-4 overflow-auto p-4">
        <div
          v-if="messages.length === 0"
          class="flex h-full flex-col items-center justify-center gap-2 text-center"
        >
          <i class="pi pi-comments text-4xl text-muted/60" />
          <p class="text-sm text-muted">Start a conversation with your selected AI provider.</p>
        </div>

        <div
          v-for="msg in messages"
          :key="msg.id"
          class="flex"
          :class="msg.role === 'user' ? 'justify-end' : 'justify-start'"
        >
          <div
            class="max-w-[75%] rounded-lg px-4 py-2 text-sm"
            :class="msg.role === 'user'
              ? 'bg-brand text-white'
              : 'border border-divider bg-canvas text-ink'"
          >
            <p class="whitespace-pre-wrap break-words">{{ msg.content }}</p>
            <div
              class="mt-1 text-[10px]"
              :class="msg.role === 'user' ? 'text-white/70' : 'text-muted'"
            >
              <span v-if="msg.role === 'assistant'">{{ msg.model }} · </span>{{ formatTime(msg.createdAt) }}
            </div>
          </div>
        </div>

        <div v-if="sending" class="flex justify-start">
          <div class="rounded-lg border border-divider bg-canvas px-4 py-2 text-sm text-muted">
            <i class="pi pi-spin pi-spinner" /> Thinking…
          </div>
        </div>
      </div>

      <!-- Input group: row 1 message textbox, row 2 provider/model selectors -->
      <div class="border-t border-divider p-4">
        <div class="rounded-lg border border-divider bg-panel focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
          <!-- Row 1: message textbox -->
          <textarea
            v-model="draft"
            rows="2"
            class="max-h-40 min-h-[3rem] w-full resize-none rounded-t-lg bg-transparent px-3 py-2 text-sm text-ink outline-none"
            placeholder="Type a message… (Enter to send, Shift+Enter for a new line)"
            @keydown.enter="onEnter"
          />

          <!-- Row 2: AI agent + model selects -->
          <div class="flex items-center gap-2 border-t border-divider p-2">
            <Select
              :model-value="providerId"
              :options="providers"
              option-label="label"
              option-value="id"
              placeholder="AI Agent"
              size="small"
              class="w-40 bg-panel text-xs"
              :pt="selectPt"
              @update:model-value="onProvider"
            >
              <template #option="{ option }">
                <span class="flex items-center gap-2 text-xs"><i :class="option.icon" />{{ option.label }}</span>
              </template>
            </Select>

            <Select
              :model-value="model"
              :options="availableModels"
              placeholder="Model"
              size="small"
              class="w-56 bg-panel text-xs"
              :pt="selectPt"
              @update:model-value="selectModel"
            />

            <Button icon="pi pi-send" label="Send" size="small" class="ml-auto" :disabled="!canSend" @click="send" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
