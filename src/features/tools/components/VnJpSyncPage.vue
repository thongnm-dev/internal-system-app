<script setup lang="ts">
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import Fieldset from "primevue/fieldset";
import InputGroup from "primevue/inputgroup";
import InputText from "primevue/inputtext";
import Tab from "primevue/tab";
import TabList from "primevue/tablist";
import TabPanel from "primevue/tabpanel";
import TabPanels from "primevue/tabpanels";
import Tabs from "primevue/tabs";
import Tag from "primevue/tag";
import { onMounted, onUnmounted, ref } from "vue";
import { explorerOpenFile } from "@/tauri/commands/explorer";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import { useVnJpSync } from "../composables/useVnJpSync";

const ctrl = useVnJpSync();

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unitIdx = 0;
  while (value >= 1024 && unitIdx < units.length - 1) {
    value /= 1024;
    unitIdx += 1;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)} ${units[unitIdx]}`;
}

function confirmClearTempFiles() {
  const count = ctrl.tempFiles.value.length;
  if (count === 0) return;
  if (window.confirm(`Xóa toàn bộ ${count} file trong thư mục Temp? Không thể hoàn tác.`)) {
    void ctrl.clearAllTempFiles();
  }
}

function confirmDeleteSelectedTempFiles() {
  const count = ctrl.selectedTempFiles.value.length;
  if (count === 0) return;
  if (window.confirm(`Xóa ${count} file đã chọn? Không thể hoàn tác.`)) {
    void ctrl.deleteSelectedTempFiles();
  }
}

const fieldsetCollapsed = ref(false);
const resultsFieldsetCollapsed = ref(false);

// Dialog copy file đã chọn sang thư mục đích do TL tự chọn.
const showCopyDialog = ref(false);
const copyDestPath = ref("");

function openCopyDialog() {
  if (!ctrl.hasSelectedTempFiles.value) return;
  copyDestPath.value = "";
  showCopyDialog.value = true;
}

async function chooseCopyDest() {
  const dir = await open({ directory: true, title: "Chọn thư mục đích" });
  if (dir) copyDestPath.value = dir as string;
}

async function handleCopyTempFiles() {
  try {
    await ctrl.copySelectedTempFiles(copyDestPath.value);
    showCopyDialog.value = false;
  } catch {
    // Giữ dialog mở khi copy lỗi — toast lỗi đã hiển thị từ composable, TL tự sửa đường dẫn rồi thử lại.
  }
}

// Kéo-thả file từ ngoài vào ô VN/JP — Tauri webview drag-drop chỉ báo 1 tọa độ chung cho cả cửa sổ
// (không phải HTML5 drop event trên từng element), nên phải tự so tọa độ đó với bounding rect của
// từng ô để biết thả vào VN hay JP. Tọa độ event là physical pixel, cần chia devicePixelRatio để về
// logical pixel khớp với getBoundingClientRect().
const boxVnRef = ref<HTMLElement | null>(null);
const boxJpRef = ref<HTMLElement | null>(null);
const dragOverSlot = ref<"vn" | "jp" | null>(null);
let unlistenDrop: (() => void) | null = null;

function slotAtPoint(physicalX: number, physicalY: number): "vn" | "jp" | null {
  const ratio = window.devicePixelRatio || 1;
  const x = physicalX / ratio;
  const y = physicalY / ratio;
  const inRect = (el: HTMLElement | null) => {
    if (!el) return false;
    const r = el.getBoundingClientRect();
    return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
  };
  if (inRect(boxVnRef.value)) return "vn";
  if (inRect(boxJpRef.value)) return "jp";
  return null;
}

onMounted(async () => {
  void ctrl.refreshTempFiles();
  try {
    unlistenDrop = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        const slot = slotAtPoint(event.payload.position.x, event.payload.position.y);
        dragOverSlot.value = null;
        const path = event.payload.paths[0];
        if (slot && path) ctrl.dropFile(slot, path);
      } else if (event.payload.type === "over") {
        dragOverSlot.value = slotAtPoint(event.payload.position.x, event.payload.position.y);
      } else {
        dragOverSlot.value = null;
      }
    });
  } catch {
    // Tauri drag-drop không khả dụng (vd chạy trong browser dev), nút "Chọn" vẫn dùng được.
  }
});

onUnmounted(() => {
  unlistenDrop?.();
  unlistenDrop = null;
});

function colLabel(col: number): string {
  let result = "";
  let c = col - 1;
  while (c >= 0) {
    result = String.fromCharCode(65 + (c % 26)) + result;
    c = Math.floor(c / 26) - 1;
  }
  return result;
}

function tabColorStyle(color: string | null | undefined): string {
  if (!color) return "";
  const hex = color.length === 8 ? color.slice(2) : color;
  return `background-color: #${hex};`;
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-3 overflow-hidden p-1">
    <!-- ═══════════ Chọn file ═══════════ -->
    <Fieldset
      v-model:collapsed="fieldsetCollapsed"
      class="shrink-0 rounded-lg border border-divider bg-panel shadow-sm fieldset-nested"
      toggleable
    >
      <template #legend="{ toggleCallback }">
        <div class="flex cursor-pointer items-center gap-2 px-6 py-3" @click="toggleCallback">
          <i :class="fieldsetCollapsed ? 'pi pi-plus' : 'pi pi-minus'" class="text-xs text-muted" />
          <h3 class="section-title">Đồng bộ tài liệu</h3>
        </div>
      </template>

      <div class="relative grid gap-3 md:grid-cols-2">
        <!-- File VN -->
        <div class="space-y-1.5">
          <span class="text-xs font-bold uppercase tracking-wide text-muted">
            File VN <span class="normal-case font-normal text-muted/70">(bản đã chỉnh sửa, có chữ đỏ)</span>
          </span>
          <div
            ref="boxVnRef"
            class="rounded-lg border-2 border-dashed transition-colors"
            :class="
              dragOverSlot === 'vn'
                ? 'border-brand bg-brand/5'
                : 'border-emerald-300 bg-emerald-50/70 dark:border-emerald-700 dark:bg-emerald-900/10'
            "
          >
            <div
              class="flex cursor-pointer flex-col items-center justify-center gap-1.5 px-4 py-6 text-center"
              @click="ctrl.pickFile('vn')"
            >
              <span class="flex h-10 w-10 items-center justify-center rounded-full bg-emerald-100 text-emerald-600 dark:bg-emerald-800/40">
                <i :class="ctrl.vnPath.value ? 'pi pi-file-excel' : 'pi pi-upload'" class="text-lg" />
              </span>
              <span v-if="ctrl.vnPath.value" class="block w-full truncate text-sm font-semibold text-ink">
                {{ ctrl.vnName.value }}
              </span>
              <span v-else class="text-sm font-semibold text-ink">Kéo &amp; thả file vào đây</span>
              <span v-if="ctrl.vnPath.value" class="text-xs text-muted">
                <span class="font-medium text-brand underline" @click.stop="explorerOpenFile(ctrl.vnPath.value)">Mở file</span>
                <span class="mx-1">·</span>
                <span class="font-medium text-red-500 underline" @click.stop="ctrl.clearFile('vn')">Xóa</span>
              </span>
              <span v-else class="text-xs text-muted">
                hoặc <span class="font-medium text-brand underline">bấm để chọn</span> từ máy
              </span>
              <span v-if="!ctrl.vnPath.value" class="text-[11px] text-muted/70">.xlsx · .xlsm</span>
            </div>
          </div>
        </div>

        <!-- File JP -->
        <div class="space-y-1.5">
          <span class="text-xs font-bold uppercase tracking-wide text-muted">
            File JP <span class="normal-case font-normal text-muted/70">(bản gốc tiếng Nhật, có strikethrough)</span>
          </span>
          <div
            ref="boxJpRef"
            class="rounded-lg border-2 border-dashed transition-colors"
            :class="
              dragOverSlot === 'jp'
                ? 'border-brand bg-brand/5'
                : 'border-blue-300 bg-blue-50/70 dark:border-blue-700 dark:bg-blue-900/10'
            "
          >
            <div
              class="flex cursor-pointer flex-col items-center justify-center gap-1.5 px-4 py-6 text-center"
              @click="ctrl.pickFile('jp')"
            >
              <span class="flex h-10 w-10 items-center justify-center rounded-full bg-blue-100 text-blue-600 dark:bg-blue-800/40">
                <i :class="ctrl.jpPath.value ? 'pi pi-file-excel' : 'pi pi-upload'" class="text-lg" />
              </span>
              <span v-if="ctrl.jpPath.value" class="block w-full truncate text-sm font-semibold text-ink">
                {{ ctrl.jpName.value }}
              </span>
              <span v-else class="text-sm font-semibold text-ink">Kéo &amp; thả file vào đây</span>
              <span v-if="ctrl.jpPath.value" class="text-xs text-muted">
                <span class="font-medium text-brand underline" @click.stop="explorerOpenFile(ctrl.jpPath.value)">Mở file</span>
                <span class="mx-1">·</span>
                <span class="font-medium text-red-500 underline" @click.stop="ctrl.clearFile('jp')">Xóa</span>
              </span>
              <span v-else class="text-xs text-muted">
                hoặc <span class="font-medium text-brand underline">bấm để chọn</span> từ máy
              </span>
              <span v-if="!ctrl.jpPath.value" class="text-[11px] text-muted/70">.xlsx · .xlsm</span>
            </div>
          </div>
        </div>

        <!-- Badge mũi tên VN → JP giữa 2 ô — absolute nên không chiếm cell của grid. -->
        <div class="pointer-events-none absolute inset-0 hidden items-center justify-center md:flex">
          <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand text-white shadow-sm ring-4 ring-panel">
            <i class="pi pi-arrow-right text-sm" />
          </span>
        </div>
      </div>

      <!-- Actions — chỉ hiện khi đã chọn ít nhất 1 trong 2 file, tránh 1 dàn nút disabled vô nghĩa. -->
      <div v-if="ctrl.vnPath.value || ctrl.jpPath.value" class="mt-3 flex flex-wrap items-center gap-2">
        <Button
          label="Phân tích & Áp dụng"
          icon="pi pi-sync"
          severity="warn"
          :loading="ctrl.analyzing.value"
          :disabled="!ctrl.canAnalyzeAndApply.value"
          @click="ctrl.analyzeAndApply()"
        />
        <Button
          label="Xuất báo cáo"
          icon="pi pi-file-excel"
          severity="primary"
          :loading="ctrl.exporting.value"
          :disabled="!ctrl.canExport.value"
          @click="ctrl.exportReport()"
        />
        <Button
          v-if="ctrl.analysis.value"
          label="Làm mới"
          icon="pi pi-eraser"
          outlined
          severity="danger"
          @click="ctrl.reset()"
        />
      </div>

      <!-- Apply result banner -->
      <div
        v-if="ctrl.applyResult.value"
        class="mt-2 flex items-center gap-2 rounded-md bg-emerald-50 px-3 py-1.5 text-sm text-emerald-700 dark:bg-emerald-900/20 dark:text-emerald-400"
      >
        <i class="pi pi-check-circle" />
        <span>
          Đã dọn dẹp (xóa <strong>{{ ctrl.applyResult.value.strikeRemovedCount }}</strong> ô strikethrough, tô đen
          <strong>{{ ctrl.applyResult.value.redBlackenedCount }}</strong> ô chữ đỏ cũ) rồi ghi
          <strong>{{ ctrl.applyResult.value.appliedCount }}</strong> ô VN vào
          <strong>{{ ctrl.applyResult.value.sheetsModified.length }}</strong> sheet.
          <span v-if="ctrl.applyResult.value.shapeAppliedCount > 0">
            (kèm <strong>{{ ctrl.applyResult.value.shapeAppliedCount }}</strong> đoạn văn trong textbox/shape)
          </span>
          <span v-if="ctrl.applyResult.value.clonedSheetCount > 0" class="text-emerald-800 dark:text-emerald-300">
            ({{ ctrl.applyResult.value.clonedSheetCount }} sheet mới clone từ VN sang JP)
          </span>
          <span v-if="ctrl.applyResult.value.delSheetCount > 0">
            ({{ ctrl.applyResult.value.delSheetCount }} sheet "(DEL)" chỉ bỏ màu chữ về đen)
          </span>
          <span v-if="ctrl.applyResult.value.columnCorrectedCount > 0" class="text-sky-600 dark:text-sky-400">
            ({{ ctrl.applyResult.value.columnCorrectedCount }} ô tự sửa lệch cột theo nội dung khớp cùng dòng)
          </span>
          <span v-if="ctrl.applyResult.value.rowsInserted > 0" class="text-sky-600 dark:text-sky-400">
            ({{ ctrl.applyResult.value.rowsInserted }} dòng tự động chèn để canh khớp lệch dòng)
          </span>
          <span
            v-if="ctrl.applyResult.value.skippedCount > 0 || ctrl.applyResult.value.cleanupSkippedCount > 0 || ctrl.applyResult.value.shapeSkippedCount > 0"
            class="text-amber-600 dark:text-amber-400"
          >
            ({{ ctrl.applyResult.value.skippedCount }} ô VN bỏ qua, {{ ctrl.applyResult.value.cleanupSkippedCount }} ô cần tự kiểm tra thủ công<span v-if="ctrl.applyResult.value.shapeSkippedCount > 0">, {{ ctrl.applyResult.value.shapeSkippedCount }} đoạn shape không tìm thấy shape cùng tên ở JP</span>)
          </span>
        </span>
      </div>
      <p v-if="ctrl.error.value" class="mt-2 text-sm text-red-500">
        <i class="pi pi-exclamation-triangle mr-1" />{{ ctrl.error.value }}
      </p>
    </Fieldset>

    <!-- ═══════════ Danh sách file trong thư mục Temp ═══════════ -->
    <section class="shrink-0 rounded-lg border border-divider bg-panel px-4 py-3 shadow-sm">
      <div class="mb-2 flex items-center gap-2">
        <i class="pi pi-folder text-lg text-brand" />
        <h3 class="section-title">Thư mục Temp</h3>
        <span class="text-xs text-muted">File kết quả các lượt Áp dụng — mở hoặc copy sang thư mục làm việc khác</span>
        <div class="ml-auto flex items-center gap-2">
          <Button
            label="Copy"
            icon="pi pi-copy"
            size="small"
            text
            :disabled="!ctrl.hasSelectedTempFiles.value"
            @click="openCopyDialog"
          />
          <Button
            v-if="ctrl.hasSelectedTempFiles.value"
            label="Xóa file đã chọn"
            icon="pi pi-trash"
            size="small"
            text
            severity="danger"
            :loading="ctrl.clearingTempFiles.value"
            @click="confirmDeleteSelectedTempFiles"
          />
          <Button
            label="Xóa toàn bộ"
            icon="pi pi-eraser"
            size="small"
            text
            severity="danger"
            :loading="ctrl.clearingTempFiles.value"
            :disabled="ctrl.tempFiles.value.length === 0"
            @click="confirmClearTempFiles"
          />
          <Button
            icon="pi pi-refresh"
            size="small"
            text
            severity="secondary"
            :loading="ctrl.loadingTempFiles.value"
            v-tooltip.top="'Làm mới danh sách'"
            @click="ctrl.refreshTempFiles()"
          />
        </div>
      </div>

      <div v-if="ctrl.tempFiles.value.length > 0" class="max-h-56 overflow-auto rounded-md border border-divider">
        <table class="w-full text-sm">
          <thead>
            <tr class="sticky top-0 z-10 border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
              <th class="px-3 py-2 text-center" style="width: 40px"></th>
              <th class="px-3 py-2 text-left">Tên file</th>
              <th class="px-3 py-2 text-center">Kích thước</th>
              <th class="px-3 py-2 text-center">Thời gian</th>
              <th class="px-3 py-2 text-center">Thao tác</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="f in ctrl.tempFiles.value"
              :key="f.path"
              class="border-b border-divider/50 hover:bg-canvas/50"
            >
              <td class="px-3 py-2 text-center">
                <Checkbox
                  :model-value="ctrl.isTempFileSelected(f.path)"
                  binary
                  @change="ctrl.toggleTempFileSelection(f.path)"
                />
              </td>
              <td class="max-w-[280px] truncate px-3 py-2 font-medium" v-tooltip.top="f.name">{{ f.name }}</td>
              <td class="px-3 py-2 text-center text-xs text-muted">{{ formatFileSize(f.size) }}</td>
              <td class="px-3 py-2 text-center text-xs text-muted">{{ f.modified }}</td>
              <td class="px-3 py-2 text-center text-xs">
                <Button
                  icon="pi pi-eye"
                  size="small"
                  text
                  rounded
                  v-tooltip.top="'Mở file'"
                  @click="explorerOpenFile(f.path)"
                />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div v-else class="rounded-md border border-dashed border-divider px-3 py-4 text-center text-xs text-muted">
        <span v-if="ctrl.loadingTempFiles.value">Đang tải…</span>
        <span v-else>Chưa có file kết quả nào trong thư mục Temp.</span>
      </div>
    </section>

    <!-- ═══════════ Kết quả phân tích ═══════════ -->
    <Fieldset
      v-if="ctrl.analysis.value"
      v-model:collapsed="resultsFieldsetCollapsed"
      :class="[
        'flex min-h-0 flex-col rounded-lg border border-divider bg-panel shadow-sm fieldset-nested',
        resultsFieldsetCollapsed ? 'shrink-0' : 'flex-1',
      ]"
      :pt="{
        contentContainer: { class: 'flex min-h-0 flex-1 flex-col' },
        contentWrapper: { class: 'flex min-h-0 flex-1 flex-col' },
        content: { class: 'flex h-full min-h-0 flex-1 flex-col p-0' },
      }"
    >
      <template #legend="{ toggleCallback }">
        <div class="flex cursor-pointer items-center gap-2 px-6 py-3" @click="toggleCallback">
          <i :class="resultsFieldsetCollapsed ? 'pi pi-plus' : 'pi pi-minus'" class="text-xs text-muted" />
          <h3 class="section-title">Kết quả phân tích</h3>
        </div>
      </template>

      <Tabs
        v-model:value="ctrl.activeTab.value"
        class="flex min-h-0 flex-1 flex-col overflow-hidden"
      >
        <TabList>
          <Tab value="overview">Tổng quan</Tab>
          <Tab value="red-cells">Chi tiết</Tab>
          <Tab value="quality">Quality Issues</Tab>
        </TabList>
        <TabPanels class="min-h-0 flex-1 overflow-hidden p-0">
          <!-- Tab: Overview -->
          <TabPanel value="overview" class="flex h-full min-h-0 flex-col">
            <div class="shrink-0 border-b border-divider px-4 py-2">
              <h4 class="font-semibold text-ink">So sánh Sheet (VN vs JP)</h4>
            </div>
            <div class="min-h-0 flex-1 overflow-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="sticky top-0 z-10 border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
                    <th class="px-4 py-2 text-left">Tên Sheet</th>
                    <th class="px-4 py-2 text-center">Tab VN</th>
                    <th class="px-4 py-2 text-center">Tab JP</th>
                    <th class="px-4 py-2 text-center">VN (rows)</th>
                    <th class="px-4 py-2 text-center">JP (rows)</th>
                    <th class="px-4 py-2 text-center">Ô đỏ</th>
                    <th class="px-4 py-2 text-center">Strikethrough</th>
                    <th class="px-4 py-2 text-center">Trạng thái</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="sc in ctrl.analysis.value.sheetCompare"
                    :key="sc.name"
                    class="border-b border-divider/50 hover:bg-canvas/50"
                  >
                    <td class="px-4 py-2 font-medium">{{ sc.name }}</td>
                    <td class="px-4 py-2 text-center">
                      <span
                        v-if="sc.vnTabColor"
                        class="inline-block h-4 w-8 rounded border border-divider/50"
                        :style="tabColorStyle(sc.vnTabColor)"
                        v-tooltip.top="sc.vnTabColor"
                      />
                      <span v-else class="text-xs text-muted">—</span>
                    </td>
                    <td class="px-4 py-2 text-center">
                      <span
                        v-if="sc.jpTabColor"
                        class="inline-block h-4 w-8 rounded border border-divider/50"
                        :style="tabColorStyle(sc.jpTabColor)"
                        v-tooltip.top="sc.jpTabColor"
                      />
                      <span v-else class="text-xs text-muted">—</span>
                    </td>
                    <td class="px-4 py-2 text-center">
                      <span v-if="sc.inVn">{{ sc.vnRows }}</span>
                      <span v-else class="text-xs text-muted">—</span>
                    </td>
                    <td class="px-4 py-2 text-center">
                      <span v-if="sc.inJp">{{ sc.jpRows }}</span>
                      <span v-else class="text-xs text-muted">—</span>
                    </td>
                    <td class="px-4 py-2 text-center">
                      <span v-if="sc.vnRedCount > 0" class="font-bold text-red-500">{{ sc.vnRedCount }}</span>
                      <span v-else class="text-xs text-muted">0</span>
                    </td>
                    <td class="px-4 py-2 text-center">
                      <span v-if="sc.jpStrikeCount > 0" class="font-bold text-amber-600">{{ sc.jpStrikeCount }}</span>
                      <span v-else class="text-xs text-muted">0</span>
                    </td>
                    <td class="px-4 py-2 text-center">
                      <Tag v-if="!sc.inVn" value="JP only" severity="warn" class="text-xs" />
                      <Tag v-else-if="!sc.inJp" value="VN only" severity="danger" class="text-xs" />
                      <Tag v-else-if="sc.vnTabColor !== sc.jpTabColor" value="Màu tab khác" severity="warn" class="text-xs" />
                      <Tag v-else value="OK" severity="success" class="text-xs" />
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </TabPanel>

          <!-- Tab: Red Cells (VN) -->
          <TabPanel value="red-cells" class="flex h-full min-h-0 flex-col">
            <div class="shrink-0 border-b border-divider px-4 py-2">
              <h4 class="font-semibold text-ink">
                Ô đỏ cần phản ánh sang JP
                <span class="ml-1 text-sm font-normal text-muted">({{ ctrl.totalRedCells.value }} ô)</span>
                <span v-if="ctrl.verifyingAi.value" class="ml-2 text-xs font-normal text-muted">
                  <i class="pi pi-spin pi-spinner mr-1" />đang kiểm tra AI…
                </span>
              </h4>
            </div>
            <div class="min-h-0 flex-1 overflow-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="sticky top-0 z-10 border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
                    <th class="px-3 py-2 text-left">#</th>
                    <th class="px-3 py-2 text-left">Sheet</th>
                    <th class="px-3 py-2 text-center">Vị trí</th>
                    <th class="px-3 py-2 text-left">Nội dung VN (đỏ)</th>
                    <th class="px-3 py-2 text-left">JP hiện tại</th>
                    <th class="px-3 py-2 text-left">
                      Kiểm tra AI
                      <span class="normal-case font-normal text-muted/70">(VN→JP chỉ để so sánh)</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="(cell, idx) in ctrl.analysis.value.redCells"
                    :key="`${cell.sheet}-${cell.row}-${cell.col}-${idx}`"
                    class="border-b border-divider/50 hover:bg-canvas/50"
                  >
                    <td class="px-3 py-2 text-xs text-muted">{{ idx + 1 }}</td>
                    <td class="px-3 py-2 text-xs font-medium">{{ cell.sheet }}</td>
                    <td class="px-3 py-2 text-center text-xs font-mono text-muted">
                      {{ colLabel(cell.col) }}{{ cell.row }}
                      <Tag v-if="cell.isShape" value="Shape" severity="info" class="ml-1 text-xs" v-tooltip.top="'Textbox nổi, neo tại vị trí này'" />
                    </td>
                    <td class="max-w-[260px] px-3 py-2">
                      <span class="line-clamp-3 whitespace-pre-wrap break-words text-red-600">{{ cell.vnText }}</span>
                    </td>
                    <td class="max-w-[260px] px-3 py-2 text-muted">
                      <span class="line-clamp-3 whitespace-pre-wrap break-words text-xs">{{ cell.jpText || "—" }}</span>
                    </td>
                    <td class="max-w-[220px] px-3 py-2">
                      <template v-if="ctrl.getVerification(cell.sheet, cell.row, cell.col)">
                        <div class="flex flex-col gap-1">
                          <Tag
                            :value="`Giống JP hiện tại: ${Math.round(ctrl.getVerification(cell.sheet, cell.row, cell.col)!.similaritySamePos)}%`"
                            :severity="ctrl.getVerification(cell.sheet, cell.row, cell.col)!.similaritySamePos >= 70 ? 'warn' : 'secondary'"
                            class="w-fit text-xs"
                            v-tooltip.top="ctrl.getVerification(cell.sheet, cell.row, cell.col)!.aiTranslation"
                          />
                          <Tag
                            v-if="ctrl.getVerification(cell.sheet, cell.row, cell.col)!.betterMatch"
                            :value="`⚠ Có thể lệch dòng — giống ${colLabel(ctrl.getVerification(cell.sheet, cell.row, cell.col)!.betterMatch!.col)}${ctrl.getVerification(cell.sheet, cell.row, cell.col)!.betterMatch!.row} hơn (${Math.round(ctrl.getVerification(cell.sheet, cell.row, cell.col)!.betterMatch!.similarity)}%)`"
                            severity="danger"
                            class="w-fit text-xs"
                          />
                        </div>
                      </template>
                      <span v-else-if="ctrl.verifyingAi.value" class="text-xs text-muted">
                        <i class="pi pi-spin pi-spinner" />
                      </span>
                      <span v-else class="text-xs text-muted">—</span>
                    </td>
                  </tr>
                  <tr v-if="ctrl.analysis.value.redCells.length === 0">
                    <td colspan="6" class="px-4 py-8 text-center text-muted">
                      <i class="pi pi-check-circle mb-2 block text-3xl text-emerald-400" />
                      Không có ô đỏ nào trong file VN
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </TabPanel>

          <!-- Tab: Quality Check -->
          <TabPanel value="quality" class="flex h-full min-h-0 flex-col">
            <div class="shrink-0 border-b border-divider px-4 py-2">
              <h4 class="font-semibold text-ink">
                Kiểm tra chất lượng (JP)
                <span class="ml-1 text-sm font-normal text-muted">({{ ctrl.totalQualityIssues.value }} vấn đề)</span>
              </h4>
            </div>
            <div class="min-h-0 flex-1 overflow-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="sticky top-0 z-10 border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
                    <th class="px-4 py-2 text-left">#</th>
                    <th class="px-4 py-2 text-left">Sheet</th>
                    <th class="px-4 py-2 text-center">Vị trí</th>
                    <th class="px-4 py-2 text-left">Loại</th>
                    <th class="px-4 py-2 text-left">Nội dung</th>
                    <th class="px-4 py-2 text-left">Mô tả</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="(issue, idx) in ctrl.analysis.value.qualityIssues"
                    :key="idx"
                    class="border-b border-divider/50 hover:bg-canvas/50"
                  >
                    <td class="px-4 py-2 text-xs text-muted">{{ idx + 1 }}</td>
                    <td class="px-4 py-2 text-xs font-medium">{{ issue.sheet }}</td>
                    <td class="px-4 py-2 text-center text-xs font-mono text-muted">
                      {{ colLabel(issue.col) }}{{ issue.row }}
                      <Tag v-if="issue.isShape" value="Shape" severity="info" class="ml-1 text-xs" v-tooltip.top="'Textbox nổi, neo tại vị trí này'" />
                    </td>
                    <td class="px-4 py-2">
                      <Tag
                        :value="issue.issueType === 'vn_char' ? '文字' : issue.issueType"
                        :severity="issue.issueType === 'vn_char' ? 'danger' : 'warn'"
                        class="text-xs"
                      />
                    </td>
                    <td class="max-w-xs px-4 py-2">
                      <span class="line-clamp-2 break-words text-orange-600">{{ issue.content }}</span>
                    </td>
                    <td class="px-4 py-2 text-xs text-muted">{{ issue.description }}</td>
                  </tr>
                  <tr v-if="ctrl.analysis.value.qualityIssues.length === 0">
                    <td colspan="6" class="px-4 py-8 text-center text-muted">
                      <i class="pi pi-check-circle mb-2 block text-3xl text-emerald-400" />
                      Không phát hiện vấn đề chất lượng nào
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </TabPanel>
        </TabPanels>
      </Tabs>
    </Fieldset>

    <!-- Empty state -->
    <div
      v-if="!ctrl.analysis.value && !ctrl.analyzing.value"
      class="flex flex-1 flex-col items-center justify-center gap-4 text-center text-muted"
    >
      <div>
        <p class="text-base font-medium">VN → JP 同期ツール</p>
        <p class="mt-1 text-sm">Chọn file VN và JP rồi nhấn <strong>Phân tích &amp; Áp dụng</strong> để bắt đầu</p>
      </div>
      <div class="max-w-lg rounded-lg border border-divider bg-panel p-4 text-left text-xs">
        <p class="mb-2 font-semibold text-ink">Quy trình:</p>
        <ol class="list-inside list-decimal space-y-1">
          <li>Chọn file VN (bản đã chỉnh sửa, nội dung mới được đánh dấu chữ <span class="font-bold text-red-500">đỏ</span>)</li>
          <li>Chọn file JP (bản gốc tiếng Nhật, nội dung cần xóa có <span class="line-through">gạch ngang</span>)</li>
          <li>
            Nhấn <strong>Phân tích &amp; Áp dụng</strong> → tool tự phân tích khác biệt (danh sách ô đỏ,
            strikethrough, quality issues) rồi ghi luôn VN text vào file JP mới
          </li>
          <li>Mở file JP mới, dùng <strong>skill dịch thuật</strong> để dịch từng ô đỏ sang tiếng Nhật</li>
        </ol>
      </div>
    </div>

    <!-- Dialog copy file Temp đã chọn sang thư mục đích -->
    <Dialog
      v-model:visible="showCopyDialog"
      header="Copy file"
      :modal="true"
      :closable="true"
      :style="{ width: '32rem' }"
    >
      <div class="space-y-3">
        <div>
          <h4 class="mb-1.5 text-sm font-semibold text-ink">
            File đã chọn <span class="font-normal text-muted">({{ ctrl.selectedTempFiles.value.length }})</span>
          </h4>
          <div class="max-h-40 overflow-auto rounded-md border border-divider">
            <div
              v-for="f in ctrl.selectedTempFiles.value"
              :key="f.path"
              class="truncate border-b border-divider/50 px-3 py-1.5 text-sm last:border-b-0"
              v-tooltip.top="f.name"
            >
              {{ f.name }}
            </div>
          </div>
        </div>

        <div>
          <h4 class="mb-1.5 text-sm font-semibold text-ink">Đường dẫn đích:</h4>
          <InputGroup>
            <InputText v-model="copyDestPath" placeholder="Chưa chọn thư mục đích" readonly />
            <Button icon="pi pi-folder-open" severity="secondary" outlined title="Chọn thư mục đích" @click="chooseCopyDest" />
          </InputGroup>
        </div>
      </div>
      <template #footer>
        <DialogFooter
          confirm-label="Copy"
          confirm-icon="pi pi-copy"
          :confirm-disabled="!copyDestPath"
          :busy="ctrl.copyingTempFiles.value"
          @cancel="showCopyDialog = false"
          @confirm="handleCopyTempFiles"
        />
      </template>
    </Dialog>
  </section>
</template>
