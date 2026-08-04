<script setup lang="ts">
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { onMounted, onUnmounted, ref } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import InputGroup from "primevue/inputgroup";
import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
import Password from "primevue/password";
import { useFileSplit } from "../composables/useFileSplit";

const ctrl = useFileSplit();

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(1)} ${units[unit]}`;
}

// Kéo-thả file/folder từ ngoài vào "Nguồn cần nén" — dùng Tauri webview drag-drop (cho path thật
// trên đĩa) giống ExcelHelperPage.vue/S3BrowserPage.vue, khác HTML5 drag events (chỉ cho File blob).
const isDragOver = ref(false);
let unlistenDrop: (() => void) | null = null;

onMounted(async () => {
  try {
    unlistenDrop = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        isDragOver.value = false;
        void ctrl.addDroppedPaths(event.payload.paths);
      } else if (event.payload.type === "over") {
        isDragOver.value = true;
      } else {
        isDragOver.value = false;
      }
    });
  } catch {
    // Tauri drag-drop không khả dụng (vd chạy trong browser dev), 2 nút "Thêm file/folder" vẫn dùng được.
  }
});

// Drag-to-resize giữa cột nguồn và cột thông số — cùng cơ chế đã dùng ở ExcelHelperPage.vue (mouse
// event thô + nhớ chiều rộng vào localStorage), vì layout ở đây là flex thường.
const layoutRowRef = ref<HTMLElement | null>(null);
const SETTINGS_WIDTH_KEY = "fileSplit.width.settings";
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

onUnmounted(() => {
  unlistenDrop?.();
  unlistenDrop = null;
  if (activeSettingsResizeMove) document.removeEventListener("mousemove", activeSettingsResizeMove);
  document.removeEventListener("mouseup", endResizeSettings);
});
</script>

<template>
  <section ref="layoutRowRef" class="relative flex h-full min-h-0 flex-1 overflow-hidden" :class="isResizingSettings ? 'select-none' : ''">
    <!-- Cột trái: Nguồn cần nén (file/folder + kéo-thả) -->
      <div class="flex min-w-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel p-4 shadow-sm">
        <div class="flex shrink-0 items-center justify-between gap-2">
          <div class="flex items-center gap-2">
            <i class="pi pi-inbox text-xl text-brand" />
            <h3 class="section-title">Nguồn cần nén</h3>
            <span class="text-xs font-semibold text-muted">({{ ctrl.sources.value.length }} mục)</span>
          </div>
          <div class="flex items-center gap-2">
            <Button icon="pi pi-file" label="Thêm file" size="small" severity="secondary" outlined @click="ctrl.pickFiles()" />
            <Button icon="pi pi-folder" label="Thêm folder" size="small" severity="secondary" outlined @click="ctrl.pickFolders()" />
            <Button
              icon="pi pi-trash"
              size="small"
              text
              severity="danger"
              title="Xoá tất cả"
              :disabled="!ctrl.sources.value.length"
              @click="ctrl.clearSources()"
            />
          </div>
        </div>

        <div
          class="relative mt-3 min-h-0 flex-1 rounded-md transition-colors"
          :class="isDragOver ? 'ring-2 ring-brand ring-offset-2 ring-offset-panel' : ''"
        >
          <div
            v-if="!ctrl.sources.value.length"
            class="flex h-full flex-col items-center justify-center gap-1 rounded-md border border-dashed text-center text-sm transition-colors"
            :class="isDragOver ? 'border-brand bg-brand/5 text-brand' : 'border-divider text-muted'"
          >
            <i class="pi pi-cloud-upload text-2xl opacity-60" />
            <span>Kéo thả file/folder vào đây, hoặc dùng “Thêm file” / “Thêm folder”.</span>
          </div>

          <ul v-else class="flex h-full flex-col divide-y divide-divider overflow-y-auto rounded-md border border-divider">
            <li
              v-for="item in ctrl.sources.value"
              :key="item.path"
              class="flex items-center gap-3 px-3 py-2"
            >
              <i :class="item.kind === 'folder' ? 'pi pi-folder text-amber-500' : 'pi pi-file text-brand'" />
              <div class="min-w-0 flex-1">
                <div class="truncate text-sm font-medium">{{ item.name }}</div>
                <div class="truncate text-xs text-muted">{{ item.path }}</div>
              </div>
              <span class="rounded bg-canvas px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-muted">
                {{ item.kind === "folder" ? "Folder" : "File" }}
              </span>
              <Button icon="pi pi-times" size="small" text severity="secondary" title="Bỏ mục này" @click="ctrl.removeSource(item.path)" />
            </li>
          </ul>

          <div
            v-if="isDragOver && ctrl.sources.value.length"
            class="pointer-events-none absolute inset-0 flex items-center justify-center rounded-md bg-brand/10 text-sm font-semibold text-brand"
          >
            Thả để thêm
          </div>
        </div>

        <!-- Kết quả: hiện ở cuối cột 1 sau khi nhấn Xem trước hoặc Nén & Tách -->
        <section
          v-if="ctrl.showResult.value"
          class="mt-3 flex max-h-[45%] shrink-0 flex-col overflow-hidden rounded-lg border border-divider bg-canvas shadow-sm"
        >
          <div class="flex shrink-0 items-center justify-between gap-3 border-b border-divider px-4 py-3">
            <div class="flex items-center gap-2">
              <i :class="ctrl.resultMode.value === 'preview' ? 'pi pi-eye' : 'pi pi-list'" class="text-brand" />
              <h3 class="section-title">Kết quả</h3>
              <span v-if="ctrl.resultMode.value === 'preview'" class="text-xs font-semibold text-muted">
                (xem trước · {{ ctrl.previewArchives.value.length }} file zip)
              </span>
            </div>
            <Button label="Ẩn" text size="small" @click="ctrl.showResult.value = false" />
          </div>

          <div class="min-h-0 flex-1 overflow-y-auto p-4">
            <!-- Preview mode -->
            <template v-if="ctrl.resultMode.value === 'preview'">
              <div class="rounded-md border border-divider">
                <!-- Header: tên zip + dung lượng nguồn -->
                <div class="flex items-center gap-2 border-b border-divider bg-panel px-3 py-2 text-sm font-semibold">
                  <i class="pi pi-box text-brand" />
                  <span class="truncate">{{ ctrl.previewArchives.value[0] }}</span>
                  <span v-if="ctrl.previewSizeBytes.value != null" class="ml-auto whitespace-nowrap text-xs font-semibold text-muted">
                    {{ formatBytes(ctrl.previewSizeBytes.value) }}
                  </span>
                  <i v-if="ctrl.config.password" v-tooltip.top="'Mã hoá AES-256'" class="pi pi-lock text-amber-500" style="font-size: 0.75rem" />
                </div>
                <!-- Danh sách nguồn bên trong zip -->
                <ul class="flex flex-col divide-y divide-divider">
                  <li
                    v-for="item in ctrl.sources.value"
                    :key="item.path"
                    class="flex items-center gap-2 px-3 py-1.5 pl-7 text-sm text-muted"
                  >
                    <i :class="item.kind === 'folder' ? 'pi pi-folder text-amber-500' : 'pi pi-file text-brand'" style="font-size: 0.75rem" />
                    <span class="truncate">{{ item.name }}</span>
                  </li>
                </ul>
              </div>

              <p class="mt-3 text-[11px] leading-5 text-muted">
                Dung lượng hiển thị là tổng nguồn trước khi nén — file zip thực tế sẽ nhỏ hơn.
                Nếu bật tách, mỗi file có thể được cắt thành
                <code>.001</code>, <code>.002</code>… — số phần được tính khi chạy thật.
              </p>
            </template>

            <!-- Run mode -->
            <template v-else>
              <div
                v-if="!ctrl.parts.value.length"
                class="rounded-md border border-dashed border-divider py-8 text-center text-sm text-muted"
              >
                Chưa có phần nào được tạo (backend chưa nối).
              </div>
              <ul v-else class="flex flex-col divide-y divide-divider rounded-md border border-divider">
                <li v-for="part in ctrl.parts.value" :key="part.name" class="flex items-center justify-between gap-3 px-3 py-2">
                  <span class="flex items-center gap-2 truncate text-sm">
                    <i class="pi pi-box text-muted" />
                    {{ part.name }}
                  </span>
                  <span class="text-xs font-semibold text-muted">{{ formatBytes(part.sizeBytes) }}</span>
                </li>
              </ul>

              <div class="mt-3 rounded-md bg-panel p-3 text-[11px] leading-5 text-muted">
                <strong class="text-ink">Cách ghép lại khi nhận:</strong>
                <div>• 7-Zip: chuột phải file <code>.001</code> → Extract (tự nhận các phần).</div>
                <div>• Windows CMD: <code>copy /b tên.zip.* tên.zip</code></div>
              </div>
            </template>
          </div>
        </section>

        <!-- Action bar -->
        <div class="mt-3 flex shrink-0 items-center justify-between gap-3 rounded-lg border border-divider bg-canvas px-4 py-3 shadow-sm">
          <span
            class="truncate text-xs font-semibold"
            :class="ctrl.messageMode.value === 'error' ? 'text-red-500' : 'text-muted'"
          >
            {{ ctrl.message.value }}
          </span>
          <div class="flex items-center gap-2">
            <Button
              icon="pi pi-eye"
              label="Xem trước"
              severity="secondary"
              outlined
              :disabled="!ctrl.sources.value.length || ctrl.running.value"
              @click="ctrl.preview()"
            />
            <Button
              :icon="ctrl.running.value ? 'pi pi-spinner pi-spin' : 'pi pi-file-export'"
              label="Nén & Tách"
              :disabled="!ctrl.canRun.value"
              @click="ctrl.run()"
            />
          </div>
        </div>
      </div>

      <!-- Resize handle: nguồn | thông số -->
      <div
        class="flex w-2 shrink-0 cursor-col-resize items-center justify-center hover:bg-brand/10"
        :class="isResizingSettings ? 'bg-brand/20' : ''"
        @mousedown="startResizeSettings"
      >
        <div class="h-8 w-0.5 rounded-full bg-divider" :class="isResizingSettings ? 'bg-brand' : ''" />
      </div>

      <!-- Cột phải: thông số + kết quả -->
      <aside
        :style="{ width: settingsWidth + 'px' }"
        class="flex shrink-0 flex-col gap-4 overflow-y-auto rounded-lg border border-divider bg-panel p-4 shadow-sm"
      >
        <section>
          <div class="flex items-center gap-2">
            <i class="pi pi-cog text-xl text-brand" />
            <h3 class="section-title">Thông số nén &amp; tách</h3>
          </div>

          <div class="mt-4 grid gap-3">
            <!-- Output dir -->
            <label class="grid gap-1.5">
              <span class="text-xs font-bold uppercase tracking-wide text-muted">Thư mục xuất</span>
              <InputGroup class="h-8">
                <InputText
                  readonly
                  placeholder="Nơi lưu file zip và các phần tách..."
                  :model-value="ctrl.config.outputDir"
                />
                <Button icon="pi pi-folder-open" severity="secondary" outlined title="Chọn thư mục xuất" @click="ctrl.pickOutputDir()" />
                <Button
                  v-if="ctrl.config.outputDir"
                  icon="pi pi-times"
                  severity="danger"
                  text
                  title="Xoá đường dẫn"
                  @click="ctrl.config.outputDir = ''"
                />
              </InputGroup>
            </label>

            <!-- Archive name -->
            <label class="grid gap-1.5">
              <span class="text-xs font-bold uppercase tracking-wide text-muted">Tên file zip</span>
              <InputText
                class="h-10 min-w-0"
                placeholder="attachment"
                :model-value="ctrl.config.archiveName"
                @update:model-value="ctrl.config.archiveName = ($event as string) ?? ''"
              />
            </label>

            <!-- Password -->
            <label class="grid gap-1.5">
              <span class="text-xs font-bold uppercase tracking-wide text-muted">Mật khẩu (tuỳ chọn)</span>
              <Password
                class="min-w-0"
                input-class="h-10 w-full"
                toggle-mask
                :feedback="false"
                placeholder="Để trống nếu không cần mã hoá"
                :model-value="ctrl.config.password"
                @update:model-value="ctrl.config.password = ($event as string) ?? ''"
              />
              <span class="text-[11px] text-muted">
                Có mật khẩu sẽ mã hoá AES-256 — người nhận cần 7-Zip/WinRAR để mở.
              </span>
            </label>

            <!-- No-split checkbox -->
            <div class="grid gap-1.5">
              <span class="text-xs font-bold uppercase tracking-wide text-muted">Chế độ tách</span>
              <div class="flex h-10 items-center gap-2 rounded-md border border-divider bg-canvas px-3">
                <Checkbox
                  input-id="noSplit"
                  binary
                  :model-value="ctrl.config.noSplit"
                  @update:model-value="ctrl.config.noSplit = $event as boolean"
                />
                <label for="noSplit" class="cursor-pointer text-sm font-medium">Không tách file</label>
              </div>
              <span class="text-[11px] text-muted">
                Bật: giữ nguyên 1 file zip (chấp nhận dung lượng lớn). Tắt: tách theo giới hạn MB.
              </span>
            </div>

            <!-- Limit MB -->
            <label class="grid gap-1.5">
              <span class="text-xs font-bold uppercase tracking-wide text-muted">Giới hạn mỗi phần (MB)</span>
              <InputNumber
                class="min-w-0"
                :min="1"
                :useGrouping="false"
                suffix=" MB"
                :disabled="ctrl.config.noSplit"
                :placeholder="ctrl.config.noSplit ? 'Không tách' : ''"
                :model-value="ctrl.config.limitMb"
                @update:model-value="ctrl.config.limitMb = $event ?? null"
              />
            </label>
          </div>
        </section>

        <Button
          class="w-full"
          icon="pi pi-refresh"
          severity="danger"
          label="Reset"
          outlined
          title="Reset to defaults"
          :disabled="ctrl.running.value"
          @click="ctrl.reset()"
        />
    </aside>
  </section>
</template>
