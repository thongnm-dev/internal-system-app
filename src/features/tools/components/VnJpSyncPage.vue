<script setup lang="ts">
import Button from "primevue/button";
import Tag from "primevue/tag";
import { explorerOpenFile } from "@/tauri/commands/explorer";
import { useVnJpSync } from "../composables/useVnJpSync";

const ctrl = useVnJpSync();

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
          <div class="flex items-center gap-2 rounded-md border border-divider bg-canvas px-3 py-2">
            <i class="pi pi-file-excel text-emerald-500" />
            <div class="min-w-0 flex-1 truncate text-sm">
              <span v-if="ctrl.vnName.value" class="font-medium">{{ ctrl.vnName.value }}</span>
              <span v-else class="text-muted">Chưa chọn file…</span>
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
          <div class="flex items-center gap-2 rounded-md border border-divider bg-canvas px-3 py-2">
            <i class="pi pi-file-excel text-blue-500" />
            <div class="min-w-0 flex-1 truncate text-sm">
              <span v-if="ctrl.jpName.value" class="font-medium">{{ ctrl.jpName.value }}</span>
              <span v-else class="text-muted">Chưa chọn file…</span>
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

      <div class="mt-3 flex items-center gap-2">
        <Button
          label="Phân tích"
          icon="pi pi-search"
          :loading="ctrl.analyzing.value"
          :disabled="!ctrl.canAnalyze.value"
          @click="ctrl.analyze()"
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

      <p v-if="ctrl.error.value" class="mt-2 text-sm text-red-500">
        <i class="pi pi-exclamation-triangle mr-1" />{{ ctrl.error.value }}
      </p>
    </section>

    <!-- ═══════════ Kết quả phân tích ═══════════ -->
    <template v-if="ctrl.analysis.value">
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
      <section class="min-h-0 flex-1 rounded-lg border border-divider bg-panel shadow-sm">
        <!-- Tab: Overview -->
        <template v-if="ctrl.activeTab.value === 'overview'">
          <div class="border-b border-divider px-4 py-2">
            <h4 class="font-semibold text-ink">So sánh Sheet (VN vs JP)</h4>
          </div>
          <div class="overflow-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
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
          <div class="border-b border-divider px-4 py-2">
            <h4 class="font-semibold text-ink">
              Ô đỏ cần phản ánh sang JP
              <span class="ml-1 text-sm font-normal text-muted">({{ ctrl.totalRedCells.value }} ô)</span>
            </h4>
          </div>
          <div class="overflow-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
                  <th class="px-3 py-2 text-left">#</th>
                  <th class="px-3 py-2 text-left">Sheet</th>
                  <th class="px-3 py-2 text-center">Vị trí</th>
                  <th class="px-3 py-2 text-left">Nội dung VN (đỏ)</th>
                  <th class="px-3 py-2 text-left">JP hiện tại</th>
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
                </tr>
                <tr v-if="ctrl.analysis.value.redCells.length === 0">
                  <td colspan="5" class="px-4 py-8 text-center text-muted">
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
          <div class="border-b border-divider px-4 py-2">
            <h4 class="font-semibold text-ink">
              Ô Strikethrough cần xóa (JP)
              <span class="ml-1 text-sm font-normal text-muted">({{ ctrl.totalStrikeCells.value }} ô)</span>
            </h4>
          </div>
          <div class="overflow-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
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
          <div class="border-b border-divider px-4 py-2">
            <h4 class="font-semibold text-ink">
              Kiểm tra chất lượng (JP)
              <span class="ml-1 text-sm font-normal text-muted">({{ ctrl.totalQualityIssues.value }} vấn đề)</span>
            </h4>
          </div>
          <div class="overflow-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-divider bg-canvas text-xs uppercase tracking-wide text-muted">
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
              Đã ghi <strong>{{ ctrl.applyResult.value.appliedCount }}</strong> ô vào
              <strong>{{ ctrl.applyResult.value.sheetsModified.length }}</strong> sheet.
              <span v-if="ctrl.applyResult.value.skippedCount > 0" class="text-amber-600 dark:text-amber-400">
                ({{ ctrl.applyResult.value.skippedCount }} ô bỏ qua)
              </span>
            </span>
          </div>
        </div>
        <p class="mt-1.5 text-xs text-muted">
          <strong>Áp dụng:</strong> tạo file JP mới với nội dung VN (màu đỏ) ở các vị trí thay đổi — mở file mới rồi dùng skill dịch thuật để dịch từng ô đỏ.
        </p>
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
