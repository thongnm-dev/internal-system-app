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

          <div class="relative mt-2 grid gap-3 md:grid-cols-2">
            <!-- File A -->
            <div class="grid gap-1.5">
              <span class="text-xs font-bold uppercase tracking-wide text-muted">File gốc (A)</span>
              <div
                ref="boxARef"
                class="rounded-lg border-2 border-dashed transition-colors"
                :class="
                  dragOverSlot === 'a'
                    ? 'border-brand bg-brand/5'
                    : 'border-sky-300 bg-sky-50/70 dark:border-sky-700 dark:bg-sky-900/10'
                "
              >
                <div
                  class="flex cursor-pointer flex-col items-center justify-center gap-1.5 px-4 py-6 text-center"
                  @click="ctrl.pickFile('a')"
                >
                  <span class="flex h-10 w-10 items-center justify-center rounded-full bg-sky-100 text-sky-500 dark:bg-sky-800/40">
                    <i :class="ctrl.fileA.value ? kindMeta[ctrl.fileA.value.kind].icon : 'pi pi-upload'" class="text-lg" />
                  </span>
                  <span v-if="ctrl.fileA.value" class="block w-full truncate text-sm font-semibold text-ink">
                    {{ ctrl.fileA.value.name }}
                  </span>
                  <span v-else class="text-sm font-semibold text-ink">Kéo &amp; thả file vào đây</span>
                  <span v-if="ctrl.fileA.value" class="text-xs text-muted">
                    <span class="font-medium text-brand underline" @click.stop="explorerOpenFile(ctrl.fileA.value!.path)">Mở file</span>
                    <span class="mx-1">·</span>
                    <span class="font-medium text-muted underline" @click.stop="explorerOpen(ctrl.fileA.value!.path)">Show in folder</span>
                    <span class="mx-1">·</span>
                    <span class="font-medium text-red-500 underline" @click.stop="ctrl.clearFile('a')">Xóa</span>
                  </span>
                  <span v-else class="text-xs text-muted">
                    hoặc <span class="font-medium text-brand underline">bấm để chọn</span> từ máy
                  </span>
                  <span v-if="!ctrl.fileA.value" class="text-[11px] text-muted/70">.docx · .xlsx · .txt · .md</span>
                  <span v-else class="rounded bg-panel px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-muted">
                    {{ kindMeta[ctrl.fileA.value.kind].label }}
                  </span>
                </div>
              </div>
            </div>

            <!-- File B -->
            <div class="grid gap-1.5">
              <span class="text-xs font-bold uppercase tracking-wide text-muted">File so sánh (B)</span>
              <div
                ref="boxBRef"
                class="rounded-lg border-2 border-dashed transition-colors"
                :class="dragOverSlot === 'b' ? 'border-brand bg-brand/5' : 'border-divider bg-canvas'"
              >
                <div
                  class="flex cursor-pointer flex-col items-center justify-center gap-1.5 px-4 py-6 text-center"
                  @click="ctrl.pickFile('b')"
                >
                  <span class="flex h-10 w-10 items-center justify-center rounded-full bg-panel text-muted">
                    <i :class="ctrl.fileB.value ? kindMeta[ctrl.fileB.value.kind].icon : 'pi pi-upload'" class="text-lg" />
                  </span>
                  <span v-if="ctrl.fileB.value" class="block w-full truncate text-sm font-semibold text-ink">
                    {{ ctrl.fileB.value.name }}
                  </span>
                  <span v-else class="text-sm font-semibold text-ink">Kéo &amp; thả file vào đây</span>
                  <span v-if="ctrl.fileB.value" class="text-xs text-muted">
                    <span class="font-medium text-brand underline" @click.stop="explorerOpenFile(ctrl.fileB.value!.path)">Mở file</span>
                    <span class="mx-1">·</span>
                    <span class="font-medium text-muted underline" @click.stop="explorerOpen(ctrl.fileB.value!.path)">Show in folder</span>
                    <span class="mx-1">·</span>
                    <span class="font-medium text-red-500 underline" @click.stop="ctrl.clearFile('b')">Xóa</span>
                  </span>
                  <span v-else class="text-xs text-muted">
                    hoặc <span class="font-medium text-brand underline">bấm để chọn</span> từ máy
                  </span>
                  <span v-if="!ctrl.fileB.value" class="text-[11px] text-muted/70">.docx · .xlsx · .txt · .md</span>
                  <span v-else class="rounded bg-panel px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-muted">
                    {{ kindMeta[ctrl.fileB.value.kind].label }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Badge "VS" giữa 2 ô — absolute nên không chiếm cell của grid, chỉ hiện khi 2 ô đứng cạnh nhau (md+). -->
            <div class="pointer-events-none absolute inset-0 hidden items-center justify-center md:flex">
              <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand text-[11px] font-bold text-white shadow-sm ring-4 ring-panel">
                VS
              </span>
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
