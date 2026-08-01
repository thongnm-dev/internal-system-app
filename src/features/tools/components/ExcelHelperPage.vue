<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import InputGroup from "primevue/inputgroup";
import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
import MultiSelect from "primevue/multiselect";
import ProgressSpinner from "primevue/progressspinner";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { useToast } from "@/shared/composables/useToast";
import { useExcelHelper } from "../composables/useExcelHelper";
import { useExcelPreview } from "../composables/useExcelPreview";

const toast = useToast();
const loading = useGlobalLoading();
const ctrl = useExcelHelper();
const preview = useExcelPreview();

watch(
  () => ctrl.inputPath.value,
  (path) => {
    if (path.trim()) void preview.loadWorkbook(path);
  },
  { immediate: true },
);

async function run() {
  await loading.run(() => ctrl.run());
  if (!ctrl.message.value) return;
  if (ctrl.messageMode.value === "error") toast.error(ctrl.message.value);
  else toast.info(ctrl.message.value);
}

function resetSettings() {
  ctrl.reset();
  preview.clear();
}

function clearInputFile() {
  ctrl.updateInputPath("");
  preview.clear();
}

// === Cột cài đặt: collapse/expand + drag-to-resize (nhớ trạng thái vào localStorage) ===
const SETTINGS_WIDTH_KEY = "excelHelper.width.settings";
const SETTINGS_COLLAPSED_KEY = "excelHelper.collapsed.settings";

function loadSettingsWidth() {
  const raw = Number(localStorage.getItem(SETTINGS_WIDTH_KEY) ?? "");
  return Number.isFinite(raw) && raw > 0 ? Math.max(320, Math.min(640, raw)) : 400;
}

const settingsWidth = ref(loadSettingsWidth());
const isSettingsCollapsed = ref(localStorage.getItem(SETTINGS_COLLAPSED_KEY) === "1");
const isResizingSettings = ref(false);
const layoutRowRef = ref<HTMLElement | null>(null);
let activeSettingsResizeMove: ((e: MouseEvent) => void) | null = null;

function startResizeSettings(e: MouseEvent) {
  e.preventDefault();
  isResizingSettings.value = true;
  const move = (ev: MouseEvent) => {
    const right = layoutRowRef.value?.getBoundingClientRect().right ?? 0;
    settingsWidth.value = Math.max(320, Math.min(640, right - ev.clientX));
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

function toggleSettingsCollapsed() {
  isSettingsCollapsed.value = !isSettingsCollapsed.value;
  localStorage.setItem(SETTINGS_COLLAPSED_KEY, isSettingsCollapsed.value ? "1" : "0");
}

onBeforeUnmount(() => {
  if (activeSettingsResizeMove) document.removeEventListener("mousemove", activeSettingsResizeMove);
  document.removeEventListener("mouseup", endResizeSettings);
});
</script>

<template>
  <section ref="layoutRowRef" class="relative flex h-full min-h-0 flex-1 overflow-hidden" :class="isResizingSettings ? 'select-none' : ''">
    <!-- Cột trái: preview Excel (Univer + Luckyexcel) -->
    <div class="relative min-w-0 flex-1 overflow-hidden rounded-lg border border-divider bg-panel">
      <div v-if="!ctrl.inputPath.value" class="flex h-full flex-col items-center justify-center p-6">
        <i class="pi pi-table text-3xl text-muted" />
        <p class="mt-2 text-sm text-muted">Excel preview will appear here.</p>
      </div>
      <div v-else-if="preview.errorMessage.value" class="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
        <i class="pi pi-exclamation-triangle text-3xl text-red-500" />
        <p class="text-sm text-muted">{{ preview.errorMessage.value }}</p>
      </div>
      <div v-show="ctrl.inputPath.value && !preview.errorMessage.value" :ref="(el) => (preview.containerRef.value = el as HTMLElement | null)" class="h-full w-full" />
      <div v-if="preview.isLoading.value" class="absolute inset-0 flex items-center justify-center bg-panel/70">
        <ProgressSpinner style="width: 3rem; height: 3rem" stroke-width="4" />
      </div>
    </div>

    <template v-if="!isSettingsCollapsed">
      <!-- Resize handle: preview | cài đặt -->
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
        <div class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-2">
            <i class="pi pi-sliders-h text-[17px] text-brand" />
            <h3 class="section-title">Settings</h3>
          </div>
          <div class="flex items-center gap-1">
            <Button
              icon="pi pi-refresh"
              severity="danger"
              label="Reset"
              rounded
              title="Reset to defaults"
              @click="resetSettings()"
            />
            <Button
              icon="pi pi-angle-right"
              severity="secondary"
              text
              rounded
              title="Collapse settings"
              @click="toggleSettingsCollapsed()"
            />
          </div>
        </div>

        <section class="rounded-lg border border-divider bg-canvas p-4">
          <div class="flex items-center gap-2">
            <i class="pi pi-file-excel text-brand" />
            <h4 class="section-title">Excel workbook</h4>
          </div>

          <div class="mt-4 grid gap-3">
            <label class="grid gap-1.5">
              <span class="font-bold uppercase tracking-wide text-muted">Input .xlsx</span>
              <InputGroup class="h-8">
                <InputText
                  readonly
                  placeholder="Select Excel workbook..."
                  :model-value="ctrl.inputPath.value"
                  @update:model-value="ctrl.updateInputPath($event as string)"
                />
                <Button icon="pi pi-folder-open" severity="secondary" outlined title="Browse Excel workbook" @click="ctrl.pickInputFile()" />
                <Button v-if="ctrl.inputPath.value" icon="pi pi-times" severity="danger" text title="Clear selected file" @click="clearInputFile()" />
              </InputGroup>
            </label>

            <label class="grid gap-1.5">
              <span class="font-bold uppercase tracking-wide text-muted">Output .xlsx</span>
              <InputGroup class="h-8">
                <InputText
                  readonly
                  placeholder="Resized workbook output path..."
                  :model-value="ctrl.outputPath.value"
                  @update:model-value="ctrl.setOutputPath($event as string)"
                />
                <Button icon="pi pi-save" severity="secondary" outlined title="Choose output path" @click="ctrl.pickOutputFile()" />
                <Button v-if="ctrl.outputPath.value" icon="pi pi-times" severity="danger" text title="Clear output path" @click="ctrl.setOutputPath('')" />
              </InputGroup>
            </label>
          </div>
        </section>

        <section v-if="ctrl.inputPath.value && ctrl.availableSheets.value.length" class="rounded-lg border border-divider bg-canvas p-4">
          <div class="flex items-center gap-2">
            <i class="pi pi-arrows-alt text-[15px] text-brand" />
            <h4 class="section-title">Sheets</h4>
          </div>
          <label class="mt-3 grid gap-1.5">
            <span class="text-[10px] font-bold uppercase tracking-wide text-muted">Sheets to process</span>
            <MultiSelect
              class="w-full"
              :model-value="ctrl.selectedSheets.value"
              :options="ctrl.availableSheets.value"
              :loading="ctrl.isLoadingSheets.value"
              display="chip"
              :max-selected-labels="2"
              selected-items-label="{0} sheets selected"
              placeholder="Select sheets..."
              filter
              :show-toggle-all="true"
              @update:model-value="ctrl.selectedSheets.value = $event as string[]"
            />
          </label>
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

        <section v-if="ctrl.result.value" class="rounded-lg border border-divider bg-canvas p-4">
          <div class="flex items-center gap-2">
            <i class="pi pi-check-circle text-[15px] text-brand" />
            <h4 class="section-title">Result</h4>
          </div>
          <div class="mt-3 grid grid-cols-1 gap-2 text-[11px]">
            <div>
              <span class="font-semibold text-muted">Output file:</span>
              {{ ctrl.result.value.output_file_name }}
            </div>
            <div>
              <span class="font-semibold text-muted">Images resized:</span>
              {{ ctrl.result.value.images_resized }}
            </div>
            <div>
              <span class="font-semibold text-muted">Sheets processed:</span>
              {{ ctrl.result.value.drawings_processed }}
            </div>
          </div>
        </section>

        <section class="rounded-lg p-4">
          <Button
            class="h-8 w-full"
            icon="pi pi-images"
            :label="ctrl.isProcessing.value ? 'Resizing...' : 'Resize Images'"
            :disabled="ctrl.isProcessing.value"
            @click="run()"
          />
        </section>
      </aside>
    </template>

    <!-- Trạng thái collapsed: chỉ hiện 1 button nổi -->
    <button
      v-else
      type="button"
      class="absolute right-4 top-4 z-10 flex h-8 w-8 items-center justify-center rounded-full border border-divider bg-panel text-brand shadow-md transition-colors hover:bg-canvas"
      title="Show settings"
      @click="toggleSettingsCollapsed()"
    >
      <i class="pi pi-sliders-h" />
    </button>
  </section>
</template>
