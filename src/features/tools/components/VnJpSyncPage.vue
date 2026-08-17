<script setup lang="ts">
import { getCurrentWebview } from "@tauri-apps/api/webview";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Tag from "primevue/tag";
import { onMounted, onUnmounted, ref } from "vue";
import { explorerOpenFile } from "@/tauri/commands/explorer";
import { useVnJpSync } from "../composables/useVnJpSync";

const ctrl = useVnJpSync();

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
  <section class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto p-1">
    <!-- ═══════════ Chọn file ═══════════ -->
    <section class="shrink-0 rounded-lg border border-divider bg-panel px-4 py-3 shadow-sm">
      <div class="mb-2 flex items-center gap-2">
        <i class="pi pi-sync text-xl text-brand" />
        <h3 class="section-title">Chọn tài liệu cần đồng bộ</h3>
        <span class="text-xs text-muted">VN → JP · Excel (.xlsx / .xlsm)</span>
      </div>

      <div class="grid gap-3 md:grid-cols-2">
        <!-- File VN -->
        <div class="space-y-1.5">
          <span class="text-xs font-bold uppercase tracking-wide text-muted">
            File VN <span class="normal-case font-normal text-muted/70">(bản đã chỉnh sửa, có chữ đỏ)</span>
          </span>
          <div
            ref="boxVnRef"
            class="flex items-center gap-2 rounded-md border px-3 py-2 transition-colors"
            :class="dragOverSlot === 'vn' ? 'border-brand bg-brand/5' : 'border-divider bg-canvas'"
          >
            <i class="pi pi-file-excel text-emerald-500" />
            <div class="min-w-0 flex-1 truncate text-sm">
              <span v-if="ctrl.vnName.value" class="font-medium">{{ ctrl.vnName.value }}</span>
              <span v-else class="text-muted">Chưa chọn file… (kéo thả vào đây)</span>
            </div>
            <Button
              v-if="ctrl.vnPath.value"
              icon="pi pi-eye"
              size="small"
              text
              severity="info"
              v-tooltip.top="'Mở file'"
              @click="explorerOpenFile(ctrl.vnPath.value)"
            />
            <Button
              v-if="ctrl.vnPath.value"
              icon="pi pi-times"
              size="small"
              text
              severity="secondary"
              @click="ctrl.clearFile('vn')"
            />
            <Button
              icon="pi pi-folder-open"
              size="small"
              text
              severity="secondary"
              label="Chọn"
              @click="ctrl.pickFile('vn')"
            />
          </div>
        </div>

        <!-- File JP -->
        <div class="space-y-1.5">
          <span class="text-xs font-bold uppercase tracking-wide text-muted">
            File JP <span class="normal-case font-normal text-muted/70">(bản gốc tiếng Nhật, có strikethrough)</span>
          </span>
          <div
            ref="boxJpRef"
            class="flex items-center gap-2 rounded-md border px-3 py-2 transition-colors"
            :class="dragOverSlot === 'jp' ? 'border-brand bg-brand/5' : 'border-divider bg-canvas'"
          >
            <i class="pi pi-file-excel text-blue-500" />
            <div class="min-w-0 flex-1 truncate text-sm">
              <span v-if="ctrl.jpName.value" class="font-medium">{{ ctrl.jpName.value }}</span>
              <span v-else class="text-muted">Chưa chọn file… (kéo thả vào đây)</span>
            </div>
            <Button
              v-if="ctrl.jpPath.value"
              icon="pi pi-eye"
              size="small"
              text
              severity="info"
              v-tooltip.top="'Mở file'"
              @click="explorerOpenFile(ctrl.jpPath.value)"
            />
            <Button
              v-if="ctrl.jpPath.value"
              icon="pi pi-times"
              size="small"
              text
              severity="secondary"
              @click="ctrl.clearFile('jp')"
            />
            <Button
              icon="pi pi-folder-open"
              size="small"
              text
              severity="secondary"
              label="Chọn"
              @click="ctrl.pickFile('jp')"
            />
          </div>
        </div>
      </div>

      <div class="mt-3 flex flex-wrap items-center gap-2">
        <Button
          label="Phân tích"
          icon="pi pi-search"
          :loading="ctrl.analyzing.value"
          :disabled="!ctrl.canAnalyze.value"
          @click="ctrl.analyze()"
        />
        <Button
          label="Dọn dẹp file JP"
          icon="pi pi-eraser"
          severity="secondary"
          outlined
          :loading="ctrl.cleaning.value"
          :disabled="!ctrl.canCleanup.value"
          v-tooltip.top="'Xóa hẳn strikethrough cũ + tô đen chữ đỏ cũ tồn đọng từ bản tablet cũ trong file JP — xuất ra file riêng để xem trước, chưa phản ánh chữ đỏ VN'"
          @click="ctrl.cleanupJp()"
        />
        <Button
          label="Kiểm tra khớp dòng"
          icon="pi pi-align-justify"
          severity="secondary"
          outlined
          :loading="ctrl.checkingAlignment.value"
          :disabled="!ctrl.canCheckAlignment.value"
          v-tooltip.top="'Phát hiện dòng VN có mà JP chưa có (dựa trên ô số/mã dùng làm điểm neo) — chỉ phát hiện, TL tự xác nhận từng vị trí trước khi chèn'"
          @click="ctrl.checkRowAlignment()"
        />
        <Button
          v-if="ctrl.analysis.value"
          label="Làm mới"
          icon="pi pi-refresh"
          text
          severity="secondary"
          @click="ctrl.reset()"
        />
      </div>

      <!-- Cleanup result banner -->
      <div
        v-if="ctrl.cleanupResult.value"
        class="mt-2 flex items-center gap-2 rounded-md bg-sky-50 px-3 py-1.5 text-sm text-sky-700 dark:bg-sky-900/20 dark:text-sky-400"
      >
        <i class="pi pi-check-circle" />
        <span>
          Đã dọn dẹp <strong>{{ ctrl.cleanupResult.value.sheetsModified.length }}</strong> sheet: xóa
          <strong>{{ ctrl.cleanupResult.value.strikeRemovedCount }}</strong> ô strikethrough cũ, tô đen
          <strong>{{ ctrl.cleanupResult.value.redBlackenedCount }}</strong> ô chữ đỏ cũ.
          <span v-if="ctrl.cleanupResult.value.skippedCount > 0" class="text-amber-600 dark:text-amber-400">
            ({{ ctrl.cleanupResult.value.skippedCount }} ô cần tự kiểm tra thủ công)
          </span>
        </span>
      </div>

      <p v-if="ctrl.error.value" class="mt-2 text-sm text-red-500">
        <i class="pi pi-exclamation-triangle mr-1" />{{ ctrl.error.value }}
      </p>
    </section>

    <!-- ═══════════ Đề xuất canh dòng VN↔JP ═══════════ -->
    <section
      v-if="ctrl.rowAlignment.value && ctrl.rowAlignment.value.suggestions.length > 0"
      class="shrink-0 rounded-lg border border-amber-300 bg-amber-50 px-4 py-3 shadow-sm dark:border-amber-700 dark:bg-amber-900/10"
    >
      <div class="mb-2 flex items-center gap-2">
        <i class="pi pi-exclamation-triangle text-lg text-amber-600" />
        <h3 class="section-title">
          Phát hiện {{ ctrl.rowAlignment.value.suggestions.length }} vị trí lệch dòng
        </h3>
        <span class="text-xs text-muted">Xem lại từng vị trí rồi tick xác nhận trước khi chèn</span>
      </div>

      <div class="overflow-auto rounded-md border border-divider">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
              <th class="px-3 py-2 text-center" style="width: 40px"></th>
              <th class="px-3 py-2 text-left">Sheet</th>
              <th class="px-3 py-2 text-center">Dòng VN</th>
              <th class="px-3 py-2 text-center">Chèn sau dòng JP</th>
              <th class="px-3 py-2 text-center">Số dòng</th>
              <th class="px-3 py-2 text-left">Loại</th>
              <th class="px-3 py-2 text-left">Nội dung VN (xem trước)</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(s, idx) in ctrl.rowAlignment.value.suggestions"
              :key="`${s.sheet}-${s.jpInsertAfterRow}-${idx}`"
              class="border-b border-divider/50 hover:bg-canvas/50"
            >
              <td class="px-3 py-2 text-center">
                <Checkbox :model-value="ctrl.isConfirmed(s)" binary @change="ctrl.toggleConfirm(s)" />
              </td>
              <td class="px-3 py-2 text-xs font-medium">{{ s.sheet }}</td>
              <td class="px-3 py-2 text-center text-xs font-mono text-muted">
                {{ s.vnRowStart === s.vnRowEnd ? s.vnRowStart : `${s.vnRowStart}-${s.vnRowEnd}` }}
              </td>
              <td class="px-3 py-2 text-center text-xs font-mono text-muted">
                {{ s.jpInsertAfterRow === 0 ? "Đầu sheet" : s.jpInsertAfterRow }}
              </td>
              <td class="px-3 py-2 text-center text-xs">{{ s.insertCount }}</td>
              <td class="px-3 py-2">
                <Tag v-if="s.hasRed" value="Đỏ (mới)" severity="danger" class="mr-1 text-xs" />
                <Tag v-if="s.hasStrike" value="Gạch bỏ (xóa)" severity="warn" class="text-xs" />
              </td>
              <td class="max-w-[280px] px-3 py-2">
                <span
                  v-for="(t, i) in s.sampleVnText"
                  :key="i"
                  class="line-clamp-1 block truncate whitespace-pre-wrap break-words text-xs text-muted"
                >
                  {{ t }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="mt-3 flex items-center gap-2">
        <Button
          label="Chèn dòng đã xác nhận"
          icon="pi pi-plus"
          severity="warn"
          :loading="ctrl.insertingRows.value"
          :disabled="!ctrl.hasConfirmedInserts.value"
          @click="ctrl.insertConfirmedRows()"
        />
        <span class="text-xs text-muted">
          Tool sẽ tự đánh số lại dòng/ô/vùng gộp bên dưới vị trí chèn trong file JP, lưu ra file mới.
        </span>
      </div>
    </section>

    <!-- ═══════════ Kết quả phân tích ═══════════ -->
    <template v-if="ctrl.analysis.value">
    

      <!-- ═══════════ Actions ═══════════ -->
      <section class="shrink-0 rounded-lg border border-divider bg-panel px-4 py-3 shadow-sm">
        <div class="flex flex-wrap items-center gap-3">
          <Button
            label="Áp dụng vào file JP"
            icon="pi pi-file-import"
            severity="primary"
            :loading="ctrl.applying.value"
            :disabled="!ctrl.canApply.value"
            v-tooltip.top="'Ghi VN text (đỏ) vào đúng vị trí trong file JP, lưu file mới'"
            @click="ctrl.applyChanges()"
          />
          <Button
            label="Xuất báo cáo"
            icon="pi pi-file-excel"
            severity="secondary"
            :loading="ctrl.exporting.value"
            :disabled="!ctrl.canExport.value"
            @click="ctrl.exportReport()"
          />
          <!-- Apply result banner -->
          <div
            v-if="ctrl.applyResult.value"
            class="ml-auto flex items-center gap-2 rounded-md bg-emerald-50 px-3 py-1.5 text-sm text-emerald-700 dark:bg-emerald-900/20 dark:text-emerald-400"
          >
            <i class="pi pi-check-circle" />
            <span>
              Đã dọn dẹp (xóa <strong>{{ ctrl.applyResult.value.strikeRemovedCount }}</strong> ô strikethrough, tô đen
              <strong>{{ ctrl.applyResult.value.redBlackenedCount }}</strong> ô chữ đỏ cũ) rồi ghi
              <strong>{{ ctrl.applyResult.value.appliedCount }}</strong> ô VN vào
              <strong>{{ ctrl.applyResult.value.sheetsModified.length }}</strong> sheet.
              <span v-if="ctrl.applyResult.value.columnCorrectedCount > 0" class="text-sky-600 dark:text-sky-400">
                ({{ ctrl.applyResult.value.columnCorrectedCount }} ô tự sửa lệch cột theo nội dung khớp cùng dòng)
              </span>
              <span
                v-if="ctrl.applyResult.value.skippedCount > 0 || ctrl.applyResult.value.cleanupSkippedCount > 0"
                class="text-amber-600 dark:text-amber-400"
              >
                ({{ ctrl.applyResult.value.skippedCount }} ô VN bỏ qua, {{ ctrl.applyResult.value.cleanupSkippedCount }} ô cần tự kiểm tra thủ công)
              </span>
            </span>
          </div>
        </div>
        <p class="mt-1.5 text-xs text-muted">
          <strong>Áp dụng:</strong> dọn dẹp strikethrough/chữ đỏ cũ tồn đọng trong file JP, rồi tạo file JP mới với nội dung VN (màu đỏ) ở các vị trí thay đổi — mở file mới rồi dùng skill dịch thuật để dịch từng ô đỏ.
        </p>
      </section>
      <!-- Summary cards -->
      <div class="grid shrink-0 grid-cols-2 gap-3 md:grid-cols-4">
        <div
          class="flex cursor-pointer flex-col gap-1 rounded-lg border bg-panel px-4 py-3 shadow-sm transition hover:border-brand"
          :class="ctrl.activeTab.value === 'overview' ? 'border-brand' : 'border-divider'"
          @click="ctrl.activeTab.value = 'overview'"
        >
          <span class="text-xs text-muted">Sheet</span>
          <span class="text-2xl font-bold text-ink">{{ ctrl.analysis.value.sheetCompare.length }}</span>
          <span class="text-xs text-muted">tổng cộng</span>
        </div>
        <div
          class="flex cursor-pointer flex-col gap-1 rounded-lg border bg-panel px-4 py-3 shadow-sm transition hover:border-red-400"
          :class="ctrl.activeTab.value === 'red-cells' ? 'border-red-400' : 'border-divider'"
          @click="ctrl.activeTab.value = 'red-cells'"
        >
          <span class="text-xs text-muted">Ô đỏ (VN)</span>
          <span class="text-2xl font-bold text-red-500">{{ ctrl.totalRedCells.value }}</span>
          <span class="text-xs text-muted">cần phản ánh sang JP</span>
        </div>
        <div
          class="flex cursor-pointer flex-col gap-1 rounded-lg border bg-panel px-4 py-3 shadow-sm transition hover:border-amber-400"
          :class="ctrl.activeTab.value === 'strike-cells' ? 'border-amber-400' : 'border-divider'"
          @click="ctrl.activeTab.value = 'strike-cells'"
        >
          <span class="text-xs text-muted">Strikethrough (JP)</span>
          <span class="text-2xl font-bold text-amber-600">{{ ctrl.totalStrikeCells.value }}</span>
          <span class="text-xs text-muted">cần xóa</span>
        </div>
        <div
          class="flex cursor-pointer flex-col gap-1 rounded-lg border bg-panel px-4 py-3 shadow-sm transition hover:border-orange-400"
          :class="ctrl.activeTab.value === 'quality' ? 'border-orange-400' : 'border-divider'"
          @click="ctrl.activeTab.value = 'quality'"
        >
          <span class="text-xs text-muted">Quality Issues</span>
          <span class="text-2xl font-bold text-orange-500">{{ ctrl.totalQualityIssues.value }}</span>
          <span class="text-xs text-muted">vấn đề chất lượng</span>
        </div>
      </div>

      <!-- Tab content -->
      <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
        <!-- Tab: Overview -->
        <template v-if="ctrl.activeTab.value === 'overview'">
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
        </template>

        <!-- Tab: Red Cells (VN) -->
        <template v-else-if="ctrl.activeTab.value === 'red-cells'">
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
                  :key="`${cell.sheet}-${cell.row}-${cell.col}`"
                  class="border-b border-divider/50 hover:bg-canvas/50"
                >
                  <td class="px-3 py-2 text-xs text-muted">{{ idx + 1 }}</td>
                  <td class="px-3 py-2 text-xs font-medium">{{ cell.sheet }}</td>
                  <td class="px-3 py-2 text-center text-xs font-mono text-muted">
                    {{ colLabel(cell.col) }}{{ cell.row }}
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
        </template>

        <!-- Tab: Strikethrough cells (JP) -->
        <template v-else-if="ctrl.activeTab.value === 'strike-cells'">
          <div class="shrink-0 border-b border-divider px-4 py-2">
            <h4 class="font-semibold text-ink">
              Ô Strikethrough cần xóa (JP)
              <span class="ml-1 text-sm font-normal text-muted">({{ ctrl.totalStrikeCells.value }} ô)</span>
            </h4>
          </div>
          <div class="min-h-0 flex-1 overflow-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="sticky top-0 z-10 border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
                  <th class="px-4 py-2 text-left">#</th>
                  <th class="px-4 py-2 text-left">Sheet</th>
                  <th class="px-4 py-2 text-center">Vị trí</th>
                  <th class="px-4 py-2 text-left">Nội dung (cần xóa)</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(cell, idx) in ctrl.analysis.value.strikeCells"
                  :key="`${cell.sheet}-${cell.row}-${cell.col}`"
                  class="border-b border-divider/50 hover:bg-canvas/50"
                >
                  <td class="px-4 py-2 text-xs text-muted">{{ idx + 1 }}</td>
                  <td class="px-4 py-2 text-xs font-medium">{{ cell.sheet }}</td>
                  <td class="px-4 py-2 text-center text-xs font-mono text-muted">
                    {{ colLabel(cell.col) }}{{ cell.row }}
                  </td>
                  <td class="max-w-xs px-4 py-2">
                    <span class="line-through text-muted">{{ cell.text }}</span>
                  </td>
                </tr>
                <tr v-if="ctrl.analysis.value.strikeCells.length === 0">
                  <td colspan="4" class="px-4 py-8 text-center text-muted">
                    <i class="pi pi-check-circle mb-2 block text-3xl text-emerald-400" />
                    Không có ô strikethrough nào trong file JP
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>

        <!-- Tab: Quality Check -->
        <template v-else-if="ctrl.activeTab.value === 'quality'">
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
        </template>
      </section>
    </template>

    <!-- Empty state -->
    <div
      v-if="!ctrl.analysis.value && !ctrl.analyzing.value"
      class="flex flex-1 flex-col items-center justify-center gap-4 py-12 text-center text-muted"
    >
      <i class="pi pi-sync text-6xl opacity-20" />
      <div>
        <p class="text-base font-medium">VN → JP 同期ツール</p>
        <p class="mt-1 text-sm">Chọn file VN và JP rồi nhấn <strong>Phân tích</strong> để bắt đầu</p>
      </div>
      <div class="max-w-lg rounded-lg border border-divider bg-panel p-4 text-left text-xs">
        <p class="mb-2 font-semibold text-ink">Quy trình:</p>
        <ol class="list-inside list-decimal space-y-1">
          <li>Chọn file VN (bản đã chỉnh sửa, nội dung mới được đánh dấu chữ <span class="font-bold text-red-500">đỏ</span>)</li>
          <li>Chọn file JP (bản gốc tiếng Nhật, nội dung cần xóa có <span class="line-through">gạch ngang</span>)</li>
          <li>Nhấn <strong>Phân tích</strong> → xem danh sách ô đỏ, strikethrough, quality issues</li>
          <li>Nhấn <strong>Áp dụng vào file JP</strong> → tạo file JP mới với VN text (đỏ) đúng vị trí</li>
          <li>Mở file JP mới, dùng <strong>skill dịch thuật</strong> để dịch từng ô đỏ sang tiếng Nhật</li>
        </ol>
      </div>
    </div>
  </section>
</template>
