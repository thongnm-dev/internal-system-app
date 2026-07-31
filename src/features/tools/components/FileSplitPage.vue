<script setup lang="ts">
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
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <div class="min-h-0 flex-1 overflow-y-auto">
      <div class="flex flex-col gap-4">
        <!-- Section 1: Source files/folders -->
        <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-2">
              <i class="pi pi-inbox text-xl text-brand" />
              <h3 class="font-bold">Nguồn cần nén</h3>
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

          <div class="mt-3">
            <div
              v-if="!ctrl.sources.value.length"
              class="flex flex-col items-center justify-center gap-1 rounded-md border border-dashed border-divider py-8 text-center text-sm text-muted"
            >
              <i class="pi pi-cloud-upload text-2xl opacity-60" />
              <span>Chưa chọn nguồn. Dùng “Thêm file” hoặc “Thêm folder”.</span>
            </div>

            <ul v-else class="flex max-h-[312px] flex-col divide-y divide-divider overflow-y-auto rounded-md border border-divider">
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
          </div>
        </section>

        <!-- Section 2: Settings -->
        <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <div class="flex items-center gap-2">
            <i class="pi pi-cog text-xl text-brand" />
            <h3 class="font-bold">Thông số nén &amp; tách</h3>
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

            <div class="grid grid-cols-1 items-start gap-3 sm:grid-cols-2">
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
            </div>

            <div class="grid grid-cols-1 items-start gap-3 sm:grid-cols-2">
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
          </div>
        </section>

        <!-- Result: hiện ngay dưới panel Thông số sau khi nhấn Xem trước hoặc Nén & Tách -->
        <section
          v-if="ctrl.showResult.value"
          class="overflow-hidden rounded-lg border border-divider bg-panel shadow-sm"
        >
          <div class="flex items-center justify-between gap-3 border-b border-divider px-4 py-3">
            <div class="flex items-center gap-2">
              <i :class="ctrl.resultMode.value === 'preview' ? 'pi pi-eye' : 'pi pi-list'" class="text-brand" />
              <h3 class="font-bold">Kết quả</h3>
              <span v-if="ctrl.resultMode.value === 'preview'" class="text-xs font-semibold text-muted">
                (xem trước · {{ ctrl.previewArchives.value.length }} file zip)
              </span>
            </div>
            <Button label="Ẩn" text size="small" @click="ctrl.showResult.value = false" />
          </div>

          <div class="p-4">
            <!-- Preview mode -->
            <template v-if="ctrl.resultMode.value === 'preview'">
              <div class="rounded-md border border-divider">
                <!-- Header: tên zip + dung lượng nguồn -->
                <div class="flex items-center gap-2 border-b border-divider bg-canvas px-3 py-2 text-sm font-semibold">
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

              <div class="mt-3 rounded-md bg-canvas p-3 text-[11px] leading-5 text-muted">
                <strong class="text-ink">Cách ghép lại khi nhận:</strong>
                <div>• 7-Zip: chuột phải file <code>.001</code> → Extract (tự nhận các phần).</div>
                <div>• Windows CMD: <code>copy /b tên.zip.* tên.zip</code></div>
              </div>
            </template>
          </div>
        </section>
      </div>
    </div>

    <!-- Action bar -->
    <div class="flex items-center justify-between gap-3 rounded-lg border border-divider bg-panel px-4 py-3 shadow-sm">
      <span
        class="truncate text-xs font-semibold"
        :class="ctrl.messageMode.value === 'error' ? 'text-red-500' : 'text-muted'"
      >
        {{ ctrl.message.value }}
      </span>
      <div class="flex items-center gap-2">
        <Button icon="pi pi-refresh" label="Đặt lại" severity="secondary" outlined :disabled="ctrl.running.value" @click="ctrl.reset()" />
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
  </section>
</template>
