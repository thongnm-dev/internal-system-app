<script setup lang="ts">
import DOMPurify from "dompurify";
import { marked } from "marked";
import Button from "primevue/button";
import ProgressSpinner from "primevue/progressspinner";
import Select from "primevue/select";
import SelectButton from "primevue/selectbutton";
import { computed, ref, watch } from "vue";
import type { CompareResult, SheetDiff } from "@/_/types/file-compare";

const props = withDefaults(
  defineProps<{
    result: CompareResult | null;
    loading?: boolean;
    exporting?: boolean;
    /** Hiện nút phóng to (chỉ dùng ở bản inline, không dùng trong dialog). */
    expandable?: boolean;
  }>(),
  { loading: false, exporting: false, expandable: false },
);

const emit = defineEmits<{ (e: "expand"): void; (e: "export"): void }>();

const viewModeOptions = [
  { label: "2 cột", value: "split" },
  { label: "1 cột", value: "inline" },
];
const viewMode = ref<"split" | "inline">("split");

const mdRenderOptions = [
  { label: "Raw diff", value: "raw" },
  { label: "Rendered", value: "rendered" },
];
const mdRender = ref<"raw" | "rendered">("raw");

const result = computed(() => props.result);
const isExcel = computed(() => result.value?.kind === "excel");
const isMarkdown = computed(() => result.value?.kind === "markdown");
const textDiff = computed(() => result.value?.textDiff ?? null);

// Side-by-side: cột trái = equal+delete, cột phải = equal+insert.
const oldSide = computed(() => textDiff.value?.lines.filter((l) => l.tag !== "insert") ?? []);
const newSide = computed(() => textDiff.value?.lines.filter((l) => l.tag !== "delete") ?? []);

function renderMarkdown(lines: { content: string }[]): string {
  const raw = lines.map((l) => l.content).join("\n");
  return DOMPurify.sanitize(marked.parse(raw, { async: false }) as string);
}
const oldRendered = computed(() => renderMarkdown(oldSide.value));
const newRendered = computed(() => renderMarkdown(newSide.value));

const sheetOptions = computed(
  () => result.value?.excelDiff?.sheets.map((s) => ({ label: sheetLabel(s), value: s.name })) ?? [],
);
function sheetLabel(s: SheetDiff) {
  const diff = s.changed + s.added + s.removed;
  return diff > 0 ? `${s.name} (${diff})` : s.name;
}

const axisClass: Record<string, string> = {
  equal: "",
  added: "bg-emerald-50 text-emerald-700",
  removed: "bg-rose-50 text-rose-700",
};
const cellClass: Record<string, string> = {
  equal: "",
  changed: "bg-amber-100 text-amber-900",
  added: "bg-emerald-100 text-emerald-900",
  removed: "bg-rose-100 text-rose-900",
};

const activeSheet = ref<string | null>(null);
const activeSheetData = computed(
  () => result.value?.excelDiff?.sheets.find((s) => s.name === activeSheet.value) ?? null,
);

watch(
  result,
  (r) => {
    if (r?.excelDiff?.sheets.length) {
      const firstChanged = r.excelDiff.sheets.find((s) => s.changed + s.added + s.removed > 0);
      activeSheet.value = (firstChanged ?? r.excelDiff.sheets[0]).name;
    } else {
      activeSheet.value = null;
    }
  },
  { immediate: true },
);

const lineNo = (n: number | null) => (n === null ? "" : String(n));
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between gap-2">
      <div class="flex items-center gap-2">
        <i class="pi pi-list text-xl text-brand" />
        <h3 class="section-title">Kết quả so sánh</h3>
        <span v-if="textDiff" class="text-xs font-semibold text-muted">
          +{{ textDiff.added }} / −{{ textDiff.removed }}
        </span>
        <span v-else-if="activeSheetData" class="text-xs font-semibold text-muted">
          {{ activeSheetData.changed }} đổi · {{ activeSheetData.added }} thêm · {{ activeSheetData.removed }} xóa
          <template v-if="activeSheetData.colRemoved || activeSheetData.colAdded">
            · cột +{{ activeSheetData.colAdded }}/−{{ activeSheetData.colRemoved }}
          </template>
          <template v-if="activeSheetData.rowRemoved || activeSheetData.rowAdded">
            · dòng +{{ activeSheetData.rowAdded }}/−{{ activeSheetData.rowRemoved }}
          </template>
        </span>
      </div>

      <div class="flex items-center gap-2">
        <SelectButton
          v-if="isMarkdown && result"
          v-model="mdRender"
          :options="mdRenderOptions"
          option-label="label"
          option-value="value"
          :allow-empty="false"
          size="small"
        />
        <SelectButton
          v-if="!isExcel && result"
          v-model="viewMode"
          :options="viewModeOptions"
          option-label="label"
          option-value="value"
          :allow-empty="false"
          size="small"
        />
        <div v-if="result && !isExcel" class="flex items-center gap-3 text-[11px] font-medium">
          <span class="flex items-center gap-1"><span class="inline-block h-3 w-3 rounded-sm bg-emerald-200" /> Thêm</span>
          <span class="flex items-center gap-1"><span class="inline-block h-3 w-3 rounded-sm bg-rose-200" /> Xóa</span>
        </div>
        <Select
          v-if="result && isExcel"
          v-model="activeSheet"
          :options="sheetOptions"
          option-label="label"
          option-value="value"
          placeholder="Chọn sheet"
          size="small"
          class="w-56"
        />
        <Button
          v-if="result"
          icon="pi pi-file-excel"
          label="Xuất Excel"
          size="small"
          severity="success"
          outlined
          :loading="exporting"
          title="Xuất kết quả so sánh ra file Excel"
          @click="emit('export')"
        />
        <Button
          v-if="expandable && result"
          icon="pi pi-window-maximize"
          size="small"
          text
          severity="secondary"
          title="Phóng to toàn màn hình"
          @click="emit('expand')"
        />
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex flex-1 items-center justify-center">
      <ProgressSpinner style="width: 40px; height: 40px" stroke-width="4" />
    </div>

    <!-- Empty state -->
    <div
      v-else-if="!result"
      class="flex flex-1 flex-col items-center justify-center gap-2 rounded-md border border-dashed border-divider py-12 text-center text-sm text-muted"
    >
      <i class="pi pi-arrow-right-arrow-left text-3xl opacity-50" />
      <span>Chọn 2 file cùng loại rồi bấm “So sánh” để xem khác biệt.</span>
    </div>

    <!-- ── Text / Markdown / Word ── -->
    <template v-else-if="!isExcel && textDiff">
      <!-- Markdown rendered -->
      <div
        v-if="isMarkdown && mdRender === 'rendered'"
        class="mt-3 grid min-h-0 flex-1 gap-3"
        :class="viewMode === 'split' ? 'grid-cols-2' : 'grid-cols-1'"
      >
        <div class="flex flex-col overflow-hidden rounded-md border border-divider">
          <div class="border-b border-divider bg-canvas px-3 py-1.5 text-xs font-bold text-muted">File A</div>
          <div class="markdown-body min-h-0 flex-1 overflow-auto p-4 text-sm" v-html="oldRendered" />
        </div>
        <div v-if="viewMode === 'split'" class="flex flex-col overflow-hidden rounded-md border border-divider">
          <div class="border-b border-divider bg-canvas px-3 py-1.5 text-xs font-bold text-muted">File B</div>
          <div class="markdown-body min-h-0 flex-1 overflow-auto p-4 text-sm" v-html="newRendered" />
        </div>
      </div>

      <!-- Raw diff — 2 cột -->
      <div v-else-if="viewMode === 'split'" class="mt-3 grid min-h-0 flex-1 grid-cols-2 gap-3">
        <div class="flex flex-col overflow-hidden rounded-md border border-divider">
          <div class="border-b border-divider bg-canvas px-3 py-1.5 text-xs font-bold text-muted">File A</div>
          <div class="min-h-0 flex-1 overflow-auto py-1 font-mono text-xs leading-relaxed">
            <div
              v-for="(l, i) in oldSide"
              :key="`a${i}`"
              class="flex px-2"
              :class="l.tag === 'delete' ? 'bg-rose-100' : ''"
            >
              <span class="mr-3 inline-block w-10 shrink-0 select-none text-right text-muted/60">{{ lineNo(l.oldLine) }}</span>
              <span class="whitespace-pre-wrap break-all text-ink">{{ l.content }}</span>
            </div>
          </div>
        </div>
        <div class="flex flex-col overflow-hidden rounded-md border border-divider">
          <div class="border-b border-divider bg-canvas px-3 py-1.5 text-xs font-bold text-muted">File B</div>
          <div class="min-h-0 flex-1 overflow-auto py-1 font-mono text-xs leading-relaxed">
            <div
              v-for="(l, i) in newSide"
              :key="`b${i}`"
              class="flex px-2"
              :class="l.tag === 'insert' ? 'bg-emerald-100' : ''"
            >
              <span class="mr-3 inline-block w-10 shrink-0 select-none text-right text-muted/60">{{ lineNo(l.newLine) }}</span>
              <span class="whitespace-pre-wrap break-all text-ink">{{ l.content }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Raw diff — 1 cột inline -->
      <div v-else class="mt-3 flex min-h-0 flex-1 flex-col overflow-hidden rounded-md border border-divider">
        <div class="min-h-0 flex-1 overflow-auto py-1 font-mono text-xs leading-relaxed">
          <div
            v-for="(l, i) in textDiff.lines"
            :key="`i${i}`"
            class="flex px-2"
            :class="l.tag === 'insert' ? 'bg-emerald-100' : l.tag === 'delete' ? 'bg-rose-100' : ''"
          >
            <span class="mr-2 inline-block w-4 shrink-0 select-none text-center font-bold"
              :class="l.tag === 'insert' ? 'text-emerald-600' : l.tag === 'delete' ? 'text-rose-600' : 'text-muted/40'">
              {{ l.tag === "insert" ? "+" : l.tag === "delete" ? "−" : "" }}
            </span>
            <span class="whitespace-pre-wrap break-all text-ink">{{ l.content }}</span>
          </div>
        </div>
      </div>
    </template>

    <!-- ── Excel (cell-by-cell, đã căn cột + dòng) ── -->
    <template v-else-if="isExcel && activeSheetData">
      <div
        v-if="activeSheetData.cells.length === 0 || activeSheetData.columns.length === 0"
        class="mt-3 flex flex-1 items-center justify-center text-sm text-muted"
      >
        Sheet trống.
      </div>
      <div v-else class="mt-3 min-h-0 flex-1 overflow-auto rounded-md border border-divider">
        <table class="border-collapse text-xs">
          <thead class="sticky top-0 z-10 bg-canvas">
            <tr>
              <th class="sticky left-0 z-20 border border-divider bg-canvas px-2 py-1 text-muted">#</th>
              <th
                v-for="(col, c) in activeSheetData.columns"
                :key="`h${c}`"
                class="border border-divider px-2 py-1 text-center font-semibold"
                :class="axisClass[col.tag]"
                :title="col.tag === 'added' ? 'Cột thêm mới' : col.tag === 'removed' ? 'Cột bị xóa' : ''"
              >
                {{ col.label }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(row, r) in activeSheetData.cells" :key="`r${r}`">
              <td
                class="sticky left-0 z-10 border border-divider px-2 py-1 text-center"
                :class="axisClass[activeSheetData.rowsMeta[r].tag] || 'bg-canvas text-muted'"
                :title="activeSheetData.rowsMeta[r].tag === 'added' ? 'Dòng thêm mới' : activeSheetData.rowsMeta[r].tag === 'removed' ? 'Dòng bị xóa' : ''"
              >
                {{ activeSheetData.rowsMeta[r].label }}
              </td>
              <td
                v-for="(cell, c) in row"
                :key="`c${r}-${c}`"
                class="max-w-[220px] truncate border border-divider px-2 py-1"
                :class="cellClass[cell.tag]"
                :title="cell.tag === 'changed' ? `${cell.old} → ${cell.new}` : cell.tag === 'removed' ? cell.old : cell.new"
              >
                {{ cell.tag === "removed" ? cell.old : cell.new }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>
