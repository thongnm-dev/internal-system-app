<script setup lang="ts">
import { getCurrentWebview } from "@tauri-apps/api/webview";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import { onMounted, onUnmounted, ref } from "vue";
import type { FileCompareKind } from "@/_/types/file-compare";
import { explorerOpen, explorerOpenFile } from "@/tauri/commands/explorer";
import { useFileCompare } from "../composables/useFileCompare";
import FileCompareResult from "./FileCompareResult.vue";

const ctrl = useFileCompare();

const kindMeta: Record<FileCompareKind, { label: string; icon: string; color: string }> = {
  text: { label: "Text", icon: "pi pi-file", color: "text-slate-500" },
  markdown: { label: "Markdown", icon: "pi pi-hashtag", color: "text-sky-500" },
  word: { label: "Word", icon: "pi pi-file-word", color: "text-blue-600" },
  excel: { label: "Excel", icon: "pi pi-file-excel", color: "text-emerald-600" },
};

const fullscreen = ref(false);

// Kéo-thả file từ ngoài vào ô A/B — Tauri webview drag-drop chỉ báo 1 tọa độ chung cho cả cửa sổ
// (không phải HTML5 drop event trên từng element), nên phải tự so tọa độ đó với bounding rect của
// từng ô để biết thả vào A hay B. Tọa độ event là physical pixel, cần chia devicePixelRatio để về
// logical pixel khớp với getBoundingClientRect().
const boxARef = ref<HTMLElement | null>(null);
const boxBRef = ref<HTMLElement | null>(null);
const dragOverSlot = ref<"a" | "b" | null>(null);
let unlistenDrop: (() => void) | null = null;

function slotAtPoint(physicalX: number, physicalY: number): "a" | "b" | null {
  const ratio = window.devicePixelRatio || 1;
  const x = physicalX / ratio;
  const y = physicalY / ratio;
  const inRect = (el: HTMLElement | null) => {
    if (!el) return false;
    const r = el.getBoundingClientRect();
    return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
  };
  if (inRect(boxARef.value)) return "a";
  if (inRect(boxBRef.value)) return "b";
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
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div class="flex min-h-0 flex-1 flex-col gap-2">
        <!-- ═══════════ Section 1: Chọn 2 file ═══════════ -->
        <section class="shrink-0 rounded-lg border border-divider bg-panel px-4 py-2 shadow-sm">
          <div class="flex items-center gap-2">
            <i class="pi pi-folder-open text-xl text-brand" />
            <h3 class="section-title">So sánh file</h3>
            <span class="text-xs text-muted">Markdown · Excel · Word · Text</span>
          </div>

          <div class="mt-2 grid gap-3 md:grid-cols-2">
            <!-- File A -->
            <div class="grid gap-1.5">
              <span class="text-xs font-bold uppercase tracking-wide text-muted">File gốc (A)</span>
              <div
                ref="boxARef"
                class="flex items-center gap-3 rounded-md border px-3 py-2.5 transition-colors"
                :class="dragOverSlot === 'a' ? 'border-brand bg-brand/5' : 'border-divider bg-canvas'"
              >
                <template v-if="ctrl.fileA.value">
                  <i :class="[kindMeta[ctrl.fileA.value.kind].icon, kindMeta[ctrl.fileA.value.kind].color, 'text-lg']" />
                  <div class="min-w-0 flex-1">
                    <div class="truncate text-sm font-medium">{{ ctrl.fileA.value.name }}</div>
                  </div>
                  <span class="rounded bg-panel px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-muted">
                    {{ kindMeta[ctrl.fileA.value.kind].label }}
                  </span>
                  <Button icon="pi pi-eye" size="small" text severity="info" v-tooltip.top="'Mở file'" @click="explorerOpenFile(ctrl.fileA.value!.path)" />
                  <Button icon="pi pi-folder-open" size="small" text severity="secondary" v-tooltip.top="'Show in folder'" @click="explorerOpen(ctrl.fileA.value!.path)" />
                  <Button icon="pi pi-times" size="small" text severity="secondary" @click="ctrl.clearFile('a')" />
                </template>
                <template v-else>
                  <div class="flex flex-1 items-center gap-2 text-sm text-muted">
                    <i class="pi pi-file opacity-60" />
                    <span>Chưa chọn file gốc — kéo thả vào đây</span>
                  </div>
                  <Button icon="pi pi-plus" label="Chọn" size="small" severity="secondary" outlined @click="ctrl.pickFile('a')" />
                </template>
              </div>
            </div>

            <!-- File B -->
            <div class="grid gap-1.5">
              <span class="text-xs font-bold uppercase tracking-wide text-muted">File so sánh (B)</span>
              <div
                ref="boxBRef"
                class="flex items-center gap-3 rounded-md border px-3 py-2.5 transition-colors"
                :class="dragOverSlot === 'b' ? 'border-brand bg-brand/5' : 'border-divider bg-canvas'"
              >
                <template v-if="ctrl.fileB.value">
                  <i :class="[kindMeta[ctrl.fileB.value.kind].icon, kindMeta[ctrl.fileB.value.kind].color, 'text-lg']" />
                  <div class="min-w-0 flex-1">
                    <div class="truncate text-sm font-medium">{{ ctrl.fileB.value.name }}</div>
                  </div>
                  <span class="rounded bg-panel px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-muted">
                    {{ kindMeta[ctrl.fileB.value.kind].label }}
                  </span>
                  <Button icon="pi pi-eye" size="small" text severity="info" v-tooltip.top="'Mở file'" @click="explorerOpenFile(ctrl.fileB.value!.path)" />
                  <Button icon="pi pi-folder-open" size="small" text severity="secondary" v-tooltip.top="'Show in folder'" @click="explorerOpen(ctrl.fileB.value!.path)" />
                  <Button icon="pi pi-times" size="small" text severity="secondary" @click="ctrl.clearFile('b')" />
                </template>
                <template v-else>
                  <div class="flex flex-1 items-center gap-2 text-sm text-muted">
                    <i class="pi pi-file opacity-60" />
                    <span>Chưa chọn file so sánh — kéo thả vào đây</span>
                  </div>
                  <Button icon="pi pi-plus" label="Chọn" size="small" severity="secondary" outlined @click="ctrl.pickFile('b')" />
                </template>
              </div>
            </div>
          </div>

          <!-- Cảnh báo / lỗi -->
          <div
            v-if="ctrl.kindMismatch.value || ctrl.error.value"
            class="mt-3 flex items-center gap-2 rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-xs font-medium text-amber-700"
          >
            <i class="pi pi-exclamation-triangle" />
            <span>{{ ctrl.error.value || "2 file phải cùng loại mới so sánh được. Vui lòng chọn lại." }}</span>
          </div>

          <div class="mt-4 flex justify-end gap-2">
            <Button
              icon="pi pi-refresh"
              label="Đặt lại"
              size="small"
              severity="secondary"
              outlined
              :disabled="!ctrl.fileA.value && !ctrl.fileB.value"
              @click="ctrl.reset()"
            />
            <Button
              icon="pi pi-arrow-right-arrow-left"
              label="So sánh"
              :loading="ctrl.loading.value"
              :disabled="!ctrl.canCompare.value"
              @click="ctrl.compare()"
            />
          </div>
        </section>

        <!-- ═══════════ Section 2: Kết quả ═══════════ -->
        <section class="flex min-h-0 flex-1 flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <FileCompareResult
            :result="ctrl.result.value"
            :loading="ctrl.loading.value"
            :exporting="ctrl.exporting.value"
            expandable
            @expand="fullscreen = true"
            @export="ctrl.exportExcel()"
          />
        </section>
      </div>
    </div>

    <!-- ═══════════ Dialog full màn hình ═══════════ -->
    <Dialog
      v-model:visible="fullscreen"
      modal
      maximizable
      header="Kết quả so sánh"
      :style="{ width: '92vw', height: '88vh' }"
      :content-style="{ display: 'flex', flexDirection: 'column', flex: '1 1 auto', minHeight: 0 }"
      :pt="{ root: { class: 'flex flex-col' } }"
    >
      <FileCompareResult
        :result="ctrl.result.value"
        :loading="ctrl.loading.value"
        :exporting="ctrl.exporting.value"
        @export="ctrl.exportExcel()"
      />
    </Dialog>
  </section>
</template>
