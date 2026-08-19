<script setup lang="ts">
import Button from "primevue/button";
import ToggleSwitch from "primevue/toggleswitch";
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
  { label: "Side by Side", value: "split" },
  { label: "Inline", value: "inline" },
];
const viewMode = ref<"split" | "inline">("split");
const diffOnly = ref(false);

const result = computed(() => props.result);
const isExcel = computed(() => result.value?.kind === "excel");
const textDiff = computed(() => result.value?.textDiff ?? null);

const allOldSide = computed(() => textDiff.value?.lines.filter((l) => l.tag !== "insert") ?? []);
const allNewSide = computed(() => textDiff.value?.lines.filter((l) => l.tag !== "delete") ?? []);
const oldSide = computed(() => diffOnly.value ? allOldSide.value.filter((l) => l.tag !== "equal") : allOldSide.value);
const newSide = computed(() => diffOnly.value ? allNewSide.value.filter((l) => l.tag !== "equal") : allNewSide.value);
const inlineLines = computed(() => {
  const lines = textDiff.value?.lines ?? [];
  return diffOnly.value ? lines.filter((l) => l.tag !== "equal") : lines;
});

const sheetOptions = computed(
  () => result.value?.excelDiff?.sheets.map((s) => ({ label: sheetLabel(s), value: s.name })) ?? [],
);
function sheetLabel(s: SheetDiff) {
  const diff = s.changed + s.added + s.removed + s.rowStrikethrough;
  return diff > 0 ? `${s.name} (${diff})` : s.name;
}

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

interface ExcelCellEntry {
  position: string;
  content: string;
  tag: string;
}

const excelGroupA = computed<ExcelCellEntry[]>(() => {
  const sheet = activeSheetData.value;
  if (!sheet) return [];
  const entries: ExcelCellEntry[] = [];
  sheet.cells.forEach((row, r) => {
    const rowMeta = sheet.rowsMeta[r];
    row.forEach((cell, c) => {
      const col = sheet.columns[c];
      if (!col) return;
      const position = `${col.label}${rowMeta.label}`;

      if (rowMeta.tag === "added") return;
      if (col.tag === "added") return;

      let content = "";
      let tag = "";

      if (rowMeta.strikethrough || rowMeta.tag === "removed") {
        content = cell.old || cell.new;
        tag = "removed";
      } else if (col.tag === "removed") {
        content = cell.old;
        tag = "removed";
      } else if (cell.tag === "changed") {
        content = cell.old;
        tag = "changed";
      } else if (cell.tag === "removed") {
        content = cell.old;
        tag = "removed";
      } else {
        return;
      }

      if (content) entries.push({ position, content, tag });
    });
  });
  return entries;
});

const excelGroupB = computed<ExcelCellEntry[]>(() => {
  const sheet = activeSheetData.value;
  if (!sheet) return [];
  const entries: ExcelCellEntry[] = [];
  sheet.cells.forEach((row, r) => {
    const rowMeta = sheet.rowsMeta[r];
    row.forEach((cell, c) => {
      const col = sheet.columns[c];
      if (!col) return;
      const position = `${col.label}${rowMeta.label}`;

      if (rowMeta.strikethrough || rowMeta.tag === "removed") return;
      if (col.tag === "removed") return;

      let content = "";
      let tag = "";

      if (rowMeta.tag === "added") {
        content = cell.new;
        tag = "added";
      } else if (col.tag === "added") {
        content = cell.new;
        tag = "added";
      } else if (cell.tag === "changed") {
        content = cell.new;
        tag = "changed";
      } else if (cell.tag === "added") {
        content = cell.new;
        tag = "added";
      } else {
        return;
      }

      if (content) entries.push({ position, content, tag });
    });
  });
  return entries;
});
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
          <template v-if="activeSheetData.rowStrikethrough">
            · {{ activeSheetData.rowStrikethrough }} strikethrough
          </template>
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
          v-if="!isExcel && result"
          v-model="viewMode"
          :options="viewModeOptions"
          option-label="label"
          option-value="value"
          :allow-empty="false"
          size="small"
        />
        <label v-if="result && !isExcel" class="flex items-center gap-1.5 text-[11px] font-medium text-muted">
          <ToggleSwitch v-model="diffOnly" />
          Chỉ diff
        </label>
        <div v-if="result && !isExcel" class="flex items-center gap-3 text-[11px] font-medium">
          <span class="flex items-center gap-1"><span class="inline-block h-3 w-3 rounded-sm bg-emerald-200" /> Thêm</span>
          <span class="flex items-center gap-1"><span class="inline-block h-3 w-3 rounded-sm bg-rose-200" /> Xóa</span>
        </div>
        <div v-if="result && isExcel" class="flex items-center gap-3 text-[11px] font-medium">
          <span class="flex items-center gap-1"><span class="inline-block h-3 w-3 rounded-sm bg-amber-200" /> Đổi</span>
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
      <!-- Raw diff — 2 cột -->
      <div v-if="viewMode === 'split'" class="mt-3 grid min-h-0 flex-1 grid-cols-2 gap-3">
        <div class="flex flex-col overflow-hidden rounded-md border border-divider">
          <div class="border-b border-divider bg-canvas px-3 py-1.5 text-xs font-bold text-muted">File A</div>
          <div class="min-h-0 flex-1 overflow-auto py-1 font-mono text-xs leading-relaxed">
            <div
              v-for="(l, i) in oldSide"
              :key="`a${i}`"
              class="flex px-2"
              :class="l.tag === 'delete' ? 'bg-rose-100 dark:bg-rose-500/20' : ''"
            >
              <span class="mr-3 inline-block w-10 shrink-0 select-none text-right text-muted/60">{{ lineNo(l.oldLine) }}</span>
              <span class="whitespace-pre-wrap break-all" :class="l.tag === 'delete' ? 'text-rose-900 dark:text-rose-300' : 'text-ink'">{{ l.content }}</span>
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
              :class="l.tag === 'insert' ? 'bg-emerald-100 dark:bg-emerald-500/20' : ''"
            >
              <span class="mr-3 inline-block w-10 shrink-0 select-none text-right text-muted/60">{{ lineNo(l.newLine) }}</span>
              <span class="whitespace-pre-wrap break-all" :class="l.tag === 'insert' ? 'text-emerald-900 dark:text-emerald-300' : 'text-ink'">{{ l.content }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Raw diff — 1 cột inline -->
      <div v-else class="mt-3 flex min-h-0 flex-1 flex-col overflow-hidden rounded-md border border-divider">
        <div class="min-h-0 flex-1 overflow-auto py-1 font-mono text-xs leading-relaxed">
          <div
            v-for="(l, i) in inlineLines"
            :key="`i${i}`"
            class="flex px-2"
            :class="l.tag === 'insert' ? 'bg-emerald-100 dark:bg-emerald-500/20' : l.tag === 'delete' ? 'bg-rose-100 dark:bg-rose-500/20' : ''"
          >
            <span class="mr-2 inline-block w-4 shrink-0 select-none text-center font-bold"
              :class="l.tag === 'insert' ? 'text-emerald-600 dark:text-emerald-400' : l.tag === 'delete' ? 'text-rose-600 dark:text-rose-400' : 'text-muted/40'">
              {{ l.tag === "insert" ? "+" : l.tag === "delete" ? "−" : "" }}
            </span>
            <span class="whitespace-pre-wrap break-all" :class="l.tag === 'insert' ? 'text-emerald-900 dark:text-emerald-300' : l.tag === 'delete' ? 'text-rose-900 dark:text-rose-300' : 'text-ink'">{{ l.content }}</span>
          </div>
        </div>
      </div>
    </template>

    <!-- ── Excel (2 group: file gốc / file so sánh) ── -->
    <template v-else-if="isExcel && activeSheetData">
      <div
        v-if="activeSheetData.cells.length === 0 || activeSheetData.columns.length === 0"
        class="mt-3 flex flex-1 items-center justify-center text-sm text-muted"
      >
        Sheet trống.
      </div>
      <div
        v-else-if="excelGroupA.length === 0 && excelGroupB.length === 0"
        class="mt-3 flex flex-1 items-center justify-center text-sm text-muted"
      >
        Không có sự khác biệt.
      </div>
      <div v-else class="mt-3 grid min-h-0 flex-1 grid-cols-2 gap-3">
        <!-- File gốc (A) -->
        <div class="flex flex-col overflow-hidden rounded-md border border-divider">
          <div class="border-b border-divider bg-canvas px-3 py-1.5 text-xs font-bold text-muted">
            File gốc (A)
            <span class="ml-2 font-normal text-muted/70">{{ excelGroupA.length }} cell</span>
          </div>
          <div class="min-h-0 flex-1 overflow-auto">
            <table v-if="excelGroupA.length" class="w-full border-collapse text-xs">
              <thead class="sticky top-0 z-10 bg-canvas">
                <tr>
                  <th class="w-20 border-b border-divider px-3 py-1.5 text-left font-semibold text-muted">Vị trí</th>
                  <th class="border-b border-divider px-3 py-1.5 text-left font-semibold text-muted">Nội dung</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(entry, i) in excelGroupA"
                  :key="`ea${i}`"
                  :class="entry.tag === 'removed'
                    ? 'bg-rose-50 dark:bg-rose-500/10'
                    : entry.tag === 'changed'
                      ? 'bg-amber-50 dark:bg-amber-500/10'
                      : ''"
                >
                  <td class="border-b border-divider px-3 py-1 font-mono font-semibold text-muted">{{ entry.position }}</td>
                  <td
                    class="border-b border-divider px-3 py-1"
                    :class="entry.tag === 'removed'
                      ? 'text-rose-700 dark:text-rose-300'
                      : entry.tag === 'changed'
                        ? 'text-amber-800 dark:text-amber-300'
                        : ''"
                  >
                    {{ entry.content }}
                  </td>
                </tr>
              </tbody>
            </table>
            <div v-else class="flex items-center justify-center py-8 text-xs text-muted">Không có</div>
          </div>
        </div>

        <!-- File so sánh (B) -->
        <div class="flex flex-col overflow-hidden rounded-md border border-divider">
          <div class="border-b border-divider bg-canvas px-3 py-1.5 text-xs font-bold text-muted">
            File so sánh (B)
            <span class="ml-2 font-normal text-muted/70">{{ excelGroupB.length }} cell</span>
          </div>
          <div class="min-h-0 flex-1 overflow-auto">
            <table v-if="excelGroupB.length" class="w-full border-collapse text-xs">
              <thead class="sticky top-0 z-10 bg-canvas">
                <tr>
                  <th class="w-20 border-b border-divider px-3 py-1.5 text-left font-semibold text-muted">Vị trí</th>
                  <th class="border-b border-divider px-3 py-1.5 text-left font-semibold text-muted">Nội dung</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(entry, i) in excelGroupB"
                  :key="`eb${i}`"
                  :class="entry.tag === 'added'
                    ? 'bg-emerald-50 dark:bg-emerald-500/10'
                    : entry.tag === 'changed'
                      ? 'bg-amber-50 dark:bg-amber-500/10'
                      : ''"
                >
                  <td class="border-b border-divider px-3 py-1 font-mono font-semibold text-muted">{{ entry.position }}</td>
                  <td
                    class="border-b border-divider px-3 py-1"
                    :class="entry.tag === 'added'
                      ? 'text-emerald-700 dark:text-emerald-300'
                      : entry.tag === 'changed'
                        ? 'text-amber-800 dark:text-amber-300'
                        : ''"
                  >
                    {{ entry.content }}
                  </td>
                </tr>
              </tbody>
            </table>
            <div v-else class="flex items-center justify-center py-8 text-xs text-muted">Không có</div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
