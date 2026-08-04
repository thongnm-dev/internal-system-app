<script setup lang="ts">
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { computed, onMounted, onUnmounted, ref } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import InputGroup from "primevue/inputgroup";
import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { useToast } from "@/shared/composables/useToast";
import { fileNameFromPath, useExcelHelper } from "../composables/useExcelHelper";

const toast = useToast();
const loading = useGlobalLoading();
const ctrl = useExcelHelper();

async function run() {
  await loading.run(() => ctrl.run());
  if (!ctrl.message.value) return;
  if (ctrl.messageMode.value === "error") toast.error(ctrl.message.value);
  else toast.info(ctrl.message.value);
}

const resultByPath = computed(() => new Map(ctrl.results.value.map((r) => [r.source_path, r])));

// Drag-to-resize giữa cột dropzone và cột settings — theo đúng cơ chế đã dùng ở bản trước (mouse
// event thô + nhớ chiều rộng vào localStorage), vì layout ở đây là flex thường, không phải
// <ResizablePanel> nào có sẵn.
const layoutRowRef = ref<HTMLElement | null>(null);
const SETTINGS_WIDTH_KEY = "excelHelper.width.settings";
const SETTINGS_MIN_WIDTH = 320;
const SETTINGS_MAX_WIDTH = 640;

function loadSettingsWidth() {
  const raw = Number(localStorage.getItem(SETTINGS_WIDTH_KEY) ?? "");
  return Number.isFinite(raw) && raw > 0 ? Math.max(SETTINGS_MIN_WIDTH, Math.min(SETTINGS_MAX_WIDTH, raw)) : 400;
}

const settingsWidth = ref(loadSettingsWidth());
const isResizingSettings = ref(false);
let activeSettingsResizeMove: ((e: MouseEvent) => void) | null = null;

function startResizeSettings(e: MouseEvent) {
  e.preventDefault();
  isResizingSettings.value = true;
  const move = (ev: MouseEvent) => {
    const right = layoutRowRef.value?.getBoundingClientRect().right ?? 0;
    settingsWidth.value = Math.max(SETTINGS_MIN_WIDTH, Math.min(SETTINGS_MAX_WIDTH, right - ev.clientX));
  };
  activeSettingsResizeMove = move;
  document.addEventListener("mousemove", move);
  document.addEventListener("mouseup", endResizeSettings);
}

function endResizeSettings() {
  isResizingSettings.value = false;
  if (activeSettingsResizeMove) document.removeEventListener("mousemove", activeSettingsResizeMove);
  document.removeEventListener("mouseup", endResizeSettings);
  activeSettingsResizeMove = null;
  localStorage.setItem(SETTINGS_WIDTH_KEY, String(Math.round(settingsWidth.value)));
}

// Kéo thả file .xlsx từ ngoài vào vùng bên trái — dùng đúng cơ chế Tauri webview drag-drop đã có ở
// S3BrowserPage.vue (getCurrentWebview().onDragDropEvent), khác với HTML5 drag events vì Tauri
// nhận path thật trên đĩa thay vì File object của browser.
const isDragOver = ref(false);
let unlistenDrop: (() => void) | null = null;

onMounted(async () => {
  try {
    unlistenDrop = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        isDragOver.value = false;
        ctrl.addFiles(event.payload.paths);
      } else if (event.payload.type === "over") {
        isDragOver.value = true;
      } else {
        isDragOver.value = false;
      }
    });
  } catch {
    // Tauri drag-drop không khả dụng (vd chạy trong browser dev), nút "Browse files" vẫn dùng được.
  }
});

onUnmounted(() => {
  unlistenDrop?.();
  unlistenDrop = null;
  if (activeSettingsResizeMove) document.removeEventListener("mousemove", activeSettingsResizeMove);
  document.removeEventListener("mouseup", endResizeSettings);
});
</script>

<template>
  <section ref="layoutRowRef" class="relative flex h-full min-h-0 flex-1 overflow-hidden" :class="isResizingSettings ? 'select-none' : ''">
    <!-- Cột trái: vùng kéo-thả nhiều file .xlsx (thay cho preview Excel trước đây) -->
    <div
      class="relative flex min-w-0 flex-1 flex-col overflow-hidden rounded-lg border-2 border-dashed p-6 transition-colors"
      :class="isDragOver ? 'border-brand bg-brand/5' : 'border-divider bg-panel'"
    >
      <div v-if="!ctrl.inputPaths.value.length" class="flex flex-1 flex-col items-center justify-center gap-3 text-center">
        <i class="pi pi-cloud-upload text-4xl text-muted opacity-60" />
        <p class="text-sm text-muted">Drag & drop Excel (.xlsx) files here</p>
        <span class="text-xs text-muted">or</span>
        <Button icon="pi pi-folder-open" label="Browse files" severity="secondary" outlined size="small" @click="ctrl.pickInputFiles()" />
      </div>

      <template v-else>
        <div class="flex shrink-0 items-center justify-between gap-2">
          <span class="text-xs font-bold uppercase tracking-wide text-muted">{{ ctrl.inputPaths.value.length }} file(s)</span>
          <div class="flex items-center gap-1">
            <Button icon="pi pi-plus" label="Add more" size="small" text severity="secondary" @click="ctrl.pickInputFiles()" />
            <Button icon="pi pi-trash" size="small" text severity="danger" title="Clear all" @click="ctrl.clearFiles()" />
          </div>
        </div>

        <div class="mt-3 min-h-0 flex-1 overflow-y-auto">
          <ul class="grid gap-2">
            <li
              v-for="path in ctrl.inputPaths.value"
              :key="path"
              class="flex items-center justify-between gap-2 rounded-md border border-divider bg-canvas px-3 py-2 text-xs"
            >
              <div class="flex min-w-0 items-center gap-2">
                <i class="pi pi-file-excel shrink-0 text-brand" />
                <span class="truncate" :title="path">{{ fileNameFromPath(path) }}</span>
              </div>
              <div class="flex shrink-0 items-center gap-2">
                <i
                  v-if="resultByPath.get(path)"
                  class="pi pi-check-circle text-green-500"
                  :title="`${resultByPath.get(path)!.images_resized} image(s) resized`"
                />
                <Button icon="pi pi-times" size="small" text severity="danger" title="Remove" @click="ctrl.removeFile(path)" />
              </div>
            </li>
          </ul>
        </div>
      </template>

      <div class="mt-3 flex shrink-0 items-center justify-between gap-3 rounded-lg border border-divider bg-canvas px-4 py-3 shadow-sm">
        <span class="truncate text-xs font-semibold" :class="ctrl.messageMode.value === 'error' ? 'text-red-500' : 'text-muted'">
          {{ ctrl.message.value }}
        </span>
        <Button
          icon="pi pi-images"
          label="Preparing"
          :disabled="ctrl.isProcessing.value || !ctrl.inputPaths.value.length"
          @click="run()"
        />
      </div>

      <div
        v-if="isDragOver"
        class="pointer-events-none absolute inset-0 flex items-center justify-center rounded-lg bg-brand/10 text-sm font-semibold text-brand"
      >
        Drop to add
      </div>
    </div>

    <!-- Resize handle: dropzone | cài đặt -->
    <div
      class="flex w-2 shrink-0 cursor-col-resize items-center justify-center hover:bg-brand/10"
      :class="isResizingSettings ? 'bg-brand/20' : ''"
      @mousedown="startResizeSettings"
    >
      <div class="h-8 w-0.5 rounded-full bg-divider" :class="isResizingSettings ? 'bg-brand' : ''" />
    </div>

    <!-- Cột phải: cài đặt -->
    <aside
      :style="{ width: settingsWidth + 'px' }"
      class="flex shrink-0 flex-col gap-4 overflow-y-auto rounded-lg border border-divider bg-panel p-4 shadow-sm text-[11px]"
    >
      <div class="flex items-center gap-2">
        <i class="pi pi-sliders-h text-[17px] text-brand" />
        <h3 class="section-title">Settings</h3>
      </div>

      <section class="rounded-lg border border-divider bg-canvas p-4">
        <div class="flex items-center gap-2">
          <i class="pi pi-folder text-brand" />
          <h4 class="section-title">Output</h4>
        </div>

        <div class="mt-4 grid gap-1.5">
          <label class="grid gap-1.5">
            <InputGroup class="h-8">
              <InputText
                readonly
                placeholder="Choose output folder..."
                :model-value="ctrl.outputFolder.value"
                @update:model-value="ctrl.setOutputFolder($event as string)"
              />
              <Button icon="pi pi-folder-open" severity="secondary" outlined title="Choose output folder" @click="ctrl.pickOutputFolder()" />
              <Button v-if="ctrl.outputFolder.value" icon="pi pi-times" severity="danger" text title="Clear output folder" @click="ctrl.setOutputFolder('')" />
            </InputGroup>
            <span class="text-[10px] text-muted">Each file is saved there as "&lt;name&gt;_resized.xlsx".</span>
          </label>
        </div>
      </section>

      <section class="rounded-lg border border-divider bg-canvas p-4">
        <div class="flex items-center gap-2">
          <i class="pi pi-sliders-h text-[15px] text-brand" />
          <h4 class="section-title">Settings</h4>
        </div>

        <div class="mt-4 grid gap-1.5">
          <div class="grid grid-cols-2 gap-3">
            <label class="grid gap-1.5">
              <span class="font-bold tracking-wide text-muted">Width</span>
              <InputNumber
                class="h-8 w-full"
                input-class="w-full"
                :min="0.01"
                :max-fraction-digits="2"
                placeholder="auto"
                suffix=" cm"
                :model-value="ctrl.widthCm.value"
                @update:model-value="ctrl.widthCm.value = $event as number | null"
              />
            </label>

            <label class="grid gap-1.5">
              <span class="font-bold tracking-wide text-muted">Height</span>
              <InputNumber
                class="h-8 w-full"
                input-class="w-full"
                :min="0.01"
                :max-fraction-digits="2"
                placeholder="auto"
                suffix=" cm"
                :model-value="ctrl.heightCm.value"
                @update:model-value="ctrl.heightCm.value = $event as number | null"
              />
            </label>
          </div>

          <label class="mt-1 flex items-center gap-2">
            <Checkbox v-model="ctrl.avoidCoveringContent.value" binary />
            <span>Avoid covering cell content (may insert rows)</span>
          </label>
        </div>

        <div class="mt-4 grid gap-3 pt-4">
          <div class="flex items-center gap-2">
            <span class="shrink-0 font-bold uppercase tracking-wide text-muted">Print settings</span>
            <div class="h-px flex-1 border-t border-divider"></div>
          </div>
          <label class="flex items-center gap-2">
            <Checkbox v-model="ctrl.pageBreakPreview.value" binary />
            <span class="text-[11px]">Page Break Preview</span>
          </label>

          <div class="flex items-center gap-2">
            <label class="flex shrink-0 items-center gap-2">
              <Checkbox v-model="ctrl.zoomEnabled.value" binary />
              <span class="text-[11px]">Zoom level</span>
            </label>
            <InputNumber
              class="h-8 w-28"
              input-class="w-full"
              :disabled="!ctrl.zoomEnabled.value"
              :min="10"
              :max="400"
              suffix="%"
              placeholder="100"
              :model-value="ctrl.zoomPercent.value"
              @update:model-value="ctrl.zoomPercent.value = $event as number | null"
            />
          </div>
        </div>

        <div class="mt-4 grid gap-3 pt-3">
          <div class="flex items-center gap-2">
            <span class="shrink-0 font-bold uppercase tracking-wide text-muted">Font</span>
            <div class="h-px flex-1 border-t border-divider"></div>
          </div>
          <label class="grid gap-1.5">
            <span class="font-bold uppercase tracking-wide text-muted">Font name</span>
            <InputText
              class="h-8"
              placeholder="e.g. Calibri"
              :model-value="ctrl.fontName.value"
              @update:model-value="ctrl.fontName.value = $event as string"
            />
          </label>

          <label class="grid gap-1.5">
            <span class="font-bold uppercase tracking-wide text-muted">Font size</span>
            <InputNumber
              class="h-8"
              :min="1"
              :max="409"
              placeholder="e.g. 11"
              :model-value="ctrl.fontSize.value"
              @update:model-value="ctrl.fontSize.value = $event as number | null"
            />
          </label>
        </div>
        <div class="mt-4 grid gap-3 pt-3">
          <div class="flex items-center gap-2">
            <span class="shrink-0 font-bold uppercase tracking-wide text-muted">Mixed</span>
            <div class="h-px flex-1 border-t border-divider"></div>
          </div>
          <div class="flex items-center gap-2">
            <label class="flex shrink-0 items-center gap-2">
              <Checkbox v-model="ctrl.startColumnEnabled.value" binary />
              <span class="text-[11px]">Column start</span>
            </label>
            <InputText
              class="h-8 w-28"
              input-class="w-full"
              :disabled="!ctrl.startColumnEnabled.value"
              placeholder="e.g. B2"
              :model-value="ctrl.startColumn.value"
              @update:model-value="ctrl.startColumn.value = ($event as string).toUpperCase()"
            />
          </div>

          <label class="flex items-center gap-2">
            <Checkbox v-model="ctrl.resetActiveCell.value" binary />
            <span class="text-[11px]">Reset to first cell (A1) on every sheet</span>
          </label>
        </div>
      </section>

      <section v-if="ctrl.results.value.length" class="rounded-lg border border-divider bg-canvas p-4">
        <div class="flex items-center gap-2">
          <i class="pi pi-check-circle text-[15px] text-brand" />
          <h4 class="section-title">Result</h4>
        </div>
        <div class="mt-3 grid gap-2 text-[11px]">
          <div v-for="r in ctrl.results.value" :key="r.output_path" class="rounded-md border border-divider bg-panel px-3 py-2">
            <div class="font-semibold text-ink">{{ r.output_file_name }}</div>
            <div class="text-muted">{{ r.images_resized }} image(s) across {{ r.drawings_processed }} sheet(s)</div>
          </div>
        </div>
      </section>

      <Button
        class="w-full"
        icon="pi pi-refresh"
        severity="danger"
        label="Reset"
        outlined
        title="Reset to defaults"
        @click="ctrl.reset()"
      />
    </aside>
  </section>
</template>
