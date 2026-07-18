<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import { useSqlEditor, type SuggestItem } from "../composables/useSqlEditor";
import { getCaretCoordinates } from "../utils/caretCoordinates";

const sql = useSqlEditor();

/** Kết quả của tab đang mở (null nếu tab chưa chạy query nào). */
const activeResult = computed(() => sql.activeTab.value?.result ?? null);

/** Nhóm nút thao tác với nội dung query (không phụ thuộc kết nối). */
const toolbarLeft = [
  { key: "format", label: "Format", icon: "pi pi-align-left", title: "Beautify SQL", action: () => sql.formatActiveQuery() },
  { key: "copy", label: "Copy", icon: "pi pi-copy", title: "Copy query", action: () => sql.copyQuery() },
  { key: "clear", label: "Clear", icon: "pi pi-eraser", title: "Xoá nội dung query", action: () => sql.clearQuery() },
];

// ── Left panel resize / collapse ────────────────────────────────────────────
const MIN_WIDTH = 240;
const MAX_WIDTH = 560;
const sidebarWidth = ref(320);
const collapsed = ref(false);
const container = ref<HTMLElement | null>(null);
let dragging = false;

function onDrag(event: MouseEvent) {
  if (!dragging || !container.value) return;
  const rect = container.value.getBoundingClientRect();
  const width = event.clientX - rect.left;
  sidebarWidth.value = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, width));
}

function stopDrag() {
  dragging = false;
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
  window.removeEventListener("mousemove", onDrag);
  window.removeEventListener("mouseup", stopDrag);
}

function startDrag() {
  dragging = true;
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
  window.addEventListener("mousemove", onDrag);
  window.addEventListener("mouseup", stopDrag);
}

// ── Autocomplete / suggestions ──────────────────────────────────────────────
const queryEl = ref<HTMLTextAreaElement | null>(null);
const suggestOpen = ref(false);
const suggestItems = ref<SuggestItem[]>([]);
const suggestIndex = ref(0);
const suggestTop = ref(0);
const suggestLeft = ref(0);

const SUGGEST_ICON: Record<SuggestItem["kind"], string> = {
  keyword: "pi pi-bolt",
  table: "pi pi-table",
  column: "pi pi-list",
};

function closeSuggest() {
  suggestOpen.value = false;
}

function moveSuggest(delta: number) {
  const count = suggestItems.value.length;
  if (count === 0) return;
  suggestIndex.value = (suggestIndex.value + delta + count) % count;
}

/**
 * Ngữ cảnh gõ ngay trước con trỏ.
 * - Nếu dạng `alias.word` → trả `qualifier = alias`, `word` là phần sau dấu `.`.
 * - Ngược lại → `qualifier = null`, `word` là định danh đang gõ.
 */
function contextBeforeCaret(el: HTMLTextAreaElement) {
  const pos = el.selectionStart;
  const before = el.value.slice(0, pos);
  const dotted = before.match(/([A-Za-z_][\w$]*)\.(\w*)$/);
  if (dotted) return { pos, qualifier: dotted[1], word: dotted[2] };
  const word = (before.match(/[\w$]*$/) ?? [""])[0];
  return { pos, qualifier: null as string | null, word };
}

function updateSuggest(force: boolean) {
  const el = queryEl.value;
  if (!el || !sql.activeTab.value) {
    closeSuggest();
    return;
  }
  const { pos, qualifier, word } = contextBeforeCaret(el);
  if (!force && word.length === 0 && qualifier === null) {
    closeSuggest();
    return;
  }
  const items = sql.getSuggestions(word, qualifier, el.value);
  if (items.length === 0) {
    closeSuggest();
    return;
  }
  const coords = getCaretCoordinates(el, pos);
  const rect = el.getBoundingClientRect();
  // Toạ độ viewport (dropdown dùng position: fixed, teleport ra body) để không bị
  // panel cha có overflow-hidden cắt mất.
  suggestTop.value = rect.top + coords.top - el.scrollTop + coords.height + 2;
  suggestLeft.value = rect.left + coords.left - el.scrollLeft;
  suggestItems.value = items;
  suggestIndex.value = 0;
  suggestOpen.value = true;
}

function acceptSuggest(item: SuggestItem | undefined) {
  const el = queryEl.value;
  const tab = sql.activeTab.value;
  if (!el || !tab || !item) return;
  const { pos, word } = contextBeforeCaret(el);
  const start = pos - word.length;
  const newValue = el.value.slice(0, start) + item.label + el.value.slice(pos);
  tab.query = newValue;
  closeSuggest();
  nextTick(() => {
    const caret = start + item.label.length;
    el.selectionStart = el.selectionEnd = caret;
    el.focus();
  });
}

function onQueryInput() {
  updateSuggest(false);
}

function onQueryKeydown(event: KeyboardEvent) {
  if (suggestOpen.value) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveSuggest(1);
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      moveSuggest(-1);
      return;
    }
    if (event.key === "Enter" || event.key === "Tab") {
      event.preventDefault();
      acceptSuggest(suggestItems.value[suggestIndex.value]);
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      closeSuggest();
      return;
    }
  }

  // Ctrl/⌘ + Space → mở gợi ý thủ công.
  if ((event.ctrlKey || event.metaKey) && event.code === "Space") {
    event.preventDefault();
    updateSuggest(true);
    return;
  }

  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
    event.preventDefault();
    if (!sql.activeTab.value?.running) sql.runQuery();
  }
}

function dbTypeLabel(value: string) {
  return sql.dbTypeOptions.find((o) => o.value === value)?.label ?? value;
}

onMounted(() => {
  sql.loadConnections();
});

onBeforeUnmount(stopDrag);
</script>

<template>
  <section ref="container" class="flex min-h-0 flex-1 overflow-hidden">
    <!-- ── Collapsed rail ─────────────────────────────────────────────── -->
    <div
      v-if="collapsed"
      class="flex w-12 flex-col items-center gap-3 border-r border-divider bg-panel py-3"
    >
      <button
        class="grid h-8 w-8 place-items-center rounded-md text-muted hover:bg-canvas hover:text-brand"
        title="Mở rộng danh sách kết nối"
        @click="collapsed = false"
      >
        <i class="pi pi-angle-double-right" />
      </button>
      <i class="pi pi-database text-lg text-muted" />
    </div>

    <!-- ── Left panel: connections ────────────────────────────────────── -->
    <aside
      v-else
      class="flex min-h-0 flex-col border-r border-divider bg-panel"
      :style="{ width: sidebarWidth + 'px' }"
    >
      <header class="flex items-center justify-between gap-2 border-b border-divider px-3 py-2.5">
        <div class="flex items-center gap-2">
          <i class="pi pi-database text-brand" />
          <h3 class="text-sm font-bold">Connections</h3>
        </div>
        <div class="flex items-center gap-1">
          <button
            class="grid h-7 w-7 place-items-center rounded-md text-muted hover:bg-canvas hover:text-brand"
            title="Thêm kết nối"
            @click="sql.openNewForm()"
          >
            <i class="pi pi-plus text-sm" />
          </button>
          <button
            class="grid h-7 w-7 place-items-center rounded-md text-muted hover:bg-canvas hover:text-brand"
            title="Thu gọn"
            @click="collapsed = true"
          >
            <i class="pi pi-angle-double-left text-sm" />
          </button>
        </div>
      </header>

      <!-- Connection list -->
      <div class="min-h-0 flex-1 overflow-y-auto p-2">
        <p v-if="sql.loading.value" class="px-2 py-3 text-sm text-muted">Đang tải…</p>
        <p
          v-else-if="sql.connections.value.length === 0"
          class="px-2 py-6 text-center text-sm text-muted"
        >
          Chưa có kết nối nào.<br />Nhấn <i class="pi pi-plus text-xs" /> để thêm.
        </p>

        <ul v-else class="grid gap-1">
          <li v-for="conn in sql.connections.value" :key="conn.id">
            <div
              class="group flex cursor-pointer items-center gap-2 rounded-md border px-2.5 py-2 transition-colors"
              :class="
                sql.activeId.value === conn.id
                  ? 'border-brand bg-canvas'
                  : 'border-transparent hover:bg-canvas'
              "
              @click="sql.selectConnection(conn.id)"
            >
              <i class="pi pi-server text-sm text-muted" />
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-semibold text-ink">{{ conn.name }}</p>
                <p class="truncate text-xs text-muted">
                  {{ dbTypeLabel(conn.db_type) }} · {{ conn.host }}:{{ conn.port }}/{{ conn.database }}
                </p>
              </div>
              <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100">
                <button
                  class="grid h-6 w-6 place-items-center rounded text-muted hover:text-brand"
                  title="Sửa"
                  @click.stop="sql.openEditForm(conn)"
                >
                  <i class="pi pi-pencil text-xs" />
                </button>
                <button
                  class="grid h-6 w-6 place-items-center rounded text-muted hover:text-red-500"
                  title="Xoá"
                  @click.stop="sql.removeConnection(conn.id)"
                >
                  <i class="pi pi-trash text-xs" />
                </button>
              </div>
            </div>
          </li>
        </ul>
      </div>

    </aside>

    <!-- ── Drag handle ────────────────────────────────────────────────── -->
    <div
      v-if="!collapsed"
      class="w-1 cursor-col-resize bg-transparent transition-colors hover:bg-brand"
      @mousedown.prevent="startDrag"
    />

    <!-- ── Right panel: query tabs + results ──────────────────────────── -->
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <!-- Tab bar -->
      <div class="flex items-stretch gap-1 border-b border-divider bg-canvas px-2 pt-1.5">
        <div class="flex min-w-0 flex-1 items-stretch gap-1 overflow-x-auto">
          <button
            v-for="tab in sql.tabs.value"
            :key="tab.id"
            class="group flex max-w-[200px] items-center gap-2 rounded-t-md border-b-2 px-3 py-1.5 text-sm transition-colors"
            :class="
              sql.activeTabId.value === tab.id
                ? 'border-brand bg-panel font-semibold text-ink'
                : 'border-transparent text-muted hover:bg-panel/60'
            "
            @click="sql.selectTab(tab.id)"
          >
            <i class="pi pi-file-edit text-xs" />
            <span class="truncate">{{ tab.title }}</span>
            <i
              class="pi pi-times shrink-0 rounded text-[10px] opacity-0 hover:text-red-500 group-hover:opacity-70"
              title="Đóng tab"
              @click.stop="sql.closeTab(tab.id)"
            />
          </button>
        </div>
        <button
          class="my-1 flex shrink-0 items-center gap-1.5 rounded-md px-2.5 py-1 text-sm text-muted hover:bg-panel hover:text-brand"
          title="Tạo query mới"
          @click="sql.newTab()"
        >
          <i class="pi pi-plus text-xs" />
          <span>New query</span>
        </button>
      </div>

      <!-- Editor -->
      <div v-if="sql.activeTab.value" class="flex flex-col gap-2 border-b border-divider bg-panel p-3">
        <div class="flex items-center justify-between gap-2">
          <div class="flex min-w-0 items-center gap-2 text-sm">
            <i class="pi pi-server text-muted" />
            <span v-if="sql.activeConnection.value" class="truncate text-muted">
              <span class="font-semibold text-ink">{{ sql.activeConnection.value.name }}</span>
              — {{ sql.activeConnection.value.host }}:{{ sql.activeConnection.value.port }}/{{ sql.activeConnection.value.database }}
            </span>
            <span v-else class="text-muted">Chọn một kết nối ở bên trái để bắt đầu.</span>
          </div>
          <Button
            icon="pi pi-play"
            label="Run"
            size="small"
            :loading="sql.activeTab.value.running"
            :disabled="sql.activeId.value === null"
            @click="sql.runQuery()"
          />
        </div>

        <!-- Toolbar -->
        <div class="flex flex-wrap items-center gap-1 rounded-md border border-divider bg-canvas px-1.5 py-1">
          <button
            v-for="tool in toolbarLeft"
            :key="tool.key"
            class="flex items-center gap-1.5 rounded px-2 py-1 text-xs text-muted transition-colors hover:bg-panel hover:text-brand"
            :title="tool.title"
            @click="tool.action"
          >
            <i :class="tool.icon" class="text-xs" />
            <span>{{ tool.label }}</span>
          </button>

          <span class="mx-1 h-4 w-px bg-divider" />

          <button
            class="flex items-center gap-1.5 rounded px-2 py-1 text-xs text-muted transition-colors hover:bg-panel hover:text-brand disabled:cursor-not-allowed disabled:opacity-40"
            title="Xem query plan (EXPLAIN)"
            :disabled="sql.activeId.value === null"
            @click="sql.explain(false)"
          >
            <i class="pi pi-sitemap text-xs" />
            <span>Explain</span>
          </button>
          <button
            class="flex items-center gap-1.5 rounded px-2 py-1 text-xs text-muted transition-colors hover:bg-panel hover:text-brand disabled:cursor-not-allowed disabled:opacity-40"
            title="Phân tích thực thi thật (EXPLAIN ANALYZE)"
            :disabled="sql.activeId.value === null"
            @click="sql.explain(true)"
          >
            <i class="pi pi-chart-bar text-xs" />
            <span>Analyze</span>
          </button>

          <span class="mx-1 h-4 w-px bg-divider" />

          <button
            class="flex items-center gap-1.5 rounded px-2 py-1 text-xs text-muted transition-colors hover:bg-panel hover:text-brand disabled:cursor-not-allowed disabled:opacity-40"
            title="Xuất kết quả ra CSV"
            :disabled="!activeResult?.has_result_set"
            @click="sql.exportCsv()"
          >
            <i class="pi pi-download text-xs" />
            <span>Export CSV</span>
          </button>
        </div>

        <div class="relative">
          <textarea
            ref="queryEl"
            v-model="sql.activeTab.value.query"
            spellcheck="false"
            placeholder="SELECT * FROM ..."
            class="min-h-[140px] w-full resize-y rounded-md border border-divider bg-canvas p-3 font-mono text-sm outline-none focus:border-brand"
            @keydown="onQueryKeydown"
            @input="onQueryInput"
            @blur="closeSuggest"
            @scroll="closeSuggest"
            @click="closeSuggest"
          />

          <!-- Suggestion dropdown (teleport để không bị panel cha cắt) -->
          <Teleport to="body">
            <ul
              v-if="suggestOpen"
              class="fixed z-[100] max-h-56 w-64 overflow-auto rounded-md border border-divider bg-panel py-1 shadow-xl"
              :style="{ top: suggestTop + 'px', left: suggestLeft + 'px' }"
            >
              <li
                v-for="(item, index) in suggestItems"
                :key="item.kind + ':' + item.label"
                class="flex cursor-pointer items-center gap-2 px-3 py-1.5 text-sm"
                :class="index === suggestIndex ? 'bg-canvas text-ink' : 'text-muted hover:bg-canvas'"
                @mousedown.prevent="acceptSuggest(item)"
                @mouseenter="suggestIndex = index"
              >
                <i :class="SUGGEST_ICON[item.kind]" class="text-xs text-brand" />
                <span class="flex-1 truncate font-mono">{{ item.label }}</span>
                <span v-if="item.detail" class="truncate text-[10px] text-muted">{{ item.detail }}</span>
              </li>
            </ul>
          </Teleport>
        </div>
        <p class="text-xs text-muted">
          <kbd class="rounded border border-divider px-1">Ctrl/⌘ + Enter</kbd> chạy ·
          <kbd class="rounded border border-divider px-1">Ctrl/⌘ + Space</kbd> gợi ý
        </p>
      </div>

      <!-- Messages -->
      <div
        v-if="sql.activeTab.value?.error"
        class="mx-3 mt-3 rounded-md border border-red-300 bg-red-50 px-3 py-2 text-sm text-red-600"
      >
        {{ sql.activeTab.value.error }}
      </div>
      <div
        v-else-if="sql.notice.value"
        class="mx-3 mt-3 rounded-md border border-emerald-300 bg-emerald-50 px-3 py-2 text-sm text-emerald-700"
      >
        {{ sql.notice.value }}
      </div>

      <!-- Results -->
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden p-3">
        <template v-if="activeResult">
          <div class="mb-2 flex items-center gap-3 text-xs text-muted">
            <span v-if="activeResult.has_result_set">
              <b class="text-ink">{{ activeResult.row_count }}</b> rows
            </span>
            <span v-else>
              <b class="text-ink">{{ activeResult.affected }}</b> rows affected
            </span>
            <span>·</span>
            <span>{{ activeResult.execution_ms }} ms</span>
          </div>

          <div
            v-if="activeResult.has_result_set"
            class="min-h-0 flex-1 overflow-auto rounded-md border border-divider"
          >
            <table class="w-full border-collapse text-sm">
              <thead class="sticky top-0 z-10 bg-panel">
                <tr>
                  <th class="border-b border-divider px-3 py-2 text-left font-semibold text-muted">#</th>
                  <th
                    v-for="col in activeResult.columns"
                    :key="col"
                    class="border-b border-divider px-3 py-2 text-left font-semibold text-ink"
                  >
                    {{ col }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(row, rowIndex) in activeResult.rows"
                  :key="rowIndex"
                  class="odd:bg-canvas/50 hover:bg-canvas"
                >
                  <td class="border-b border-divider px-3 py-1.5 text-xs text-muted">{{ rowIndex + 1 }}</td>
                  <td
                    v-for="(cell, cellIndex) in row"
                    :key="cellIndex"
                    class="max-w-[360px] truncate border-b border-divider px-3 py-1.5 font-mono text-xs"
                    :title="cell ?? 'NULL'"
                  >
                    <span v-if="cell === null" class="italic text-muted">NULL</span>
                    <span v-else>{{ cell }}</span>
                  </td>
                </tr>
                <tr v-if="activeResult.rows.length === 0">
                  <td
                    :colspan="activeResult.columns.length + 1"
                    class="px-3 py-6 text-center text-sm text-muted"
                  >
                    Query trả về 0 dòng.
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="flex flex-1 items-center justify-center text-sm text-muted">
            Câu lệnh đã chạy thành công.
          </div>
        </template>

        <div v-else class="flex flex-1 items-center justify-center text-sm text-muted">
          Chạy một query để xem kết quả.
        </div>
      </div>
    </div>

    <!-- ── Connection dialog ──────────────────────────────────────────── -->
    <Dialog
      :visible="sql.showForm.value"
      modal
      class="w-full max-w-lg rounded-lg bg-panel shadow-xl"
      @update:visible="(value: boolean) => { if (!value) sql.closeForm(); }"
    >
      <template #header>
        <div class="flex items-center gap-2">
          <i class="pi pi-database text-brand" />
          <h3 class="font-bold text-ink">{{ sql.isNew.value ? "Kết nối mới" : "Sửa kết nối" }}</h3>
        </div>
      </template>

      <div class="grid gap-3">
        <label class="grid gap-1">
          <span class="text-xs font-bold text-muted">Tên kết nối <span class="text-red-500">*</span></span>
          <input
            v-model="sql.form.name"
            placeholder="Ví dụ: Prod PostgreSQL"
            class="rounded-md border border-divider bg-panel px-3 py-2 text-sm outline-none focus:border-brand"
          />
        </label>

        <div class="grid grid-cols-2 gap-3">
          <label class="grid gap-1">
            <span class="text-xs font-bold text-muted">Loại DB</span>
            <select
              :value="sql.form.db_type"
              class="rounded-md border border-divider bg-panel px-3 py-2 text-sm outline-none focus:border-brand"
              @change="sql.onDbTypeChange(($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in sql.dbTypeOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}{{ opt.supported ? "" : " (sắp có)" }}
              </option>
            </select>
          </label>
          <label class="grid gap-1">
            <span class="text-xs font-bold text-muted">Port</span>
            <input
              v-model.number="sql.form.port"
              type="number"
              placeholder="5432"
              class="rounded-md border border-divider bg-panel px-3 py-2 text-sm outline-none focus:border-brand"
            />
          </label>
        </div>

        <label class="grid gap-1">
          <span class="text-xs font-bold text-muted">Host <span class="text-red-500">*</span></span>
          <input
            v-model="sql.form.host"
            placeholder="localhost"
            class="rounded-md border border-divider bg-panel px-3 py-2 text-sm outline-none focus:border-brand"
          />
        </label>

        <label class="grid gap-1">
          <span class="text-xs font-bold text-muted">Database <span class="text-red-500">*</span></span>
          <input
            v-model="sql.form.database"
            placeholder="Tên database"
            class="rounded-md border border-divider bg-panel px-3 py-2 text-sm outline-none focus:border-brand"
          />
        </label>

        <div class="grid grid-cols-2 gap-3">
          <label class="grid gap-1">
            <span class="text-xs font-bold text-muted">Username</span>
            <input
              v-model="sql.form.username"
              placeholder="postgres"
              class="rounded-md border border-divider bg-panel px-3 py-2 text-sm outline-none focus:border-brand"
            />
          </label>
          <label class="grid gap-1">
            <span class="text-xs font-bold text-muted">Password</span>
            <input
              v-model="sql.form.password"
              type="password"
              placeholder="••••••"
              class="rounded-md border border-divider bg-panel px-3 py-2 text-sm outline-none focus:border-brand"
            />
          </label>
        </div>

        <div
          v-if="sql.error.value"
          class="rounded-md border border-red-300 bg-red-50 px-3 py-2 text-sm text-red-600"
        >
          {{ sql.error.value }}
        </div>
        <div
          v-else-if="sql.notice.value"
          class="rounded-md border border-emerald-300 bg-emerald-50 px-3 py-2 text-sm text-emerald-700"
        >
          {{ sql.notice.value }}
        </div>
      </div>

      <template #footer>
        <div class="flex w-full items-center justify-between gap-2">
          <Button
            severity="secondary"
            outlined
            icon="pi pi-bolt"
            label="Test"
            :loading="sql.testing.value"
            @click="sql.testConnection()"
          />
          <div class="flex items-center gap-2">
            <Button severity="secondary" text label="Huỷ" @click="sql.closeForm()" />
            <Button
              icon="pi pi-check"
              label="Lưu"
              :loading="sql.saving.value"
              @click="sql.saveConnection()"
            />
          </div>
        </div>
      </template>
    </Dialog>
  </section>
</template>
