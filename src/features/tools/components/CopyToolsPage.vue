<script setup lang="ts">
import InputSwitch from "primevue/inputswitch";
import { useCopyTools } from "../composables/useCopyTools";

const ctrl = useCopyTools();

const HELP: Record<string, string> = {
  input: "FOLDER_ROOT — thư mục gốc cần quét (BẮT BUỘC). Dùng / hoặc \\ đều được.",
  output: "Thư mục đích để copy file vào (BẮT BUỘC). Tương đối tính từ thư mục chạy.",
  keyword:
    "Dùng CHUNG cho 2 nút:\n" +
    "• Copy → từ khóa lọc theo TÊN file (phân tách bằng dấu phẩy hoặc xuống dòng). Để TRỐNG = lấy tất cả.\n" +
    "• Copy theo folder → mỗi DÒNG là 1 tên folder cần gom.",
  files:
    "Danh sách tên file CỐ ĐỊNH cần copy (ghi đủ cả đuôi).\n" +
    "NẾU có giá trị → ƯU TIÊN: chỉ copy đúng các file này, BỎ QUA keyword/ext/limit_copy.",
  ext: "Đuôi file cần copy, phân tách bằng dấu phẩy. Mặc định (để trống) = xlsx.\nVD: xlsx, xlsm",
  limit_copy: "Giới hạn số file copy khi lọc bằng keyword (chống quét nhầm quá nhiều). 0 = không giới hạn.",
  skip_dir:
    "Thêm từ khóa nhận diện thư mục cần BỎ QUA (cộng dồn vào mặc định: 履歴, history, old, 旧, backup, bak, 過去, archive).\n" +
    "Ngoài ra thư mục theo NGÀY (yyyyMMdd) cũng tự động bỏ qua.",
  report_dir: "Thư mục lưu log file đã tồn tại (khi overwrite=false và create_history=false). Tương đối → tính từ gốc repo.",
  flat: "true = gom phẳng tất cả vào 1 thư mục.\nfalse = giữ nguyên cây thư mục con.",
  group_by_parens:
    "true = đặt mỗi file vào thư mục con lấy từ NỘI DUNG cặp ngoặc NGOÀI CÙNG của tên file. ƯU TIÊN hơn flat.\n" +
    "false = không gom theo ngoặc.",
  overwrite:
    "overwrite & create_history ĐỘC LẬP nhau. Điều khiển ghi vào VỊ TRÍ CHÍNH khi file đã tồn tại:\n" +
    "true = ghi đè.\nfalse = KHÔNG đụng vị trí chính.",
  create_history:
    "true = LUÔN tạo snapshot vào folder lịch sử theo NGÀY copy (yyyyMMdd; nếu đã có thì yyyyMMdd_02, _03…).\n" +
    "false = không tạo snapshot, chỉ ghi vị trí chính.",
  delete_non_vn:
    "true = trước khi copy file '*_VN', XÓA file cùng tên KHÔNG có '_VN' ở mọi vị trí trong output (trừ folder lịch sử).\n" +
    "false = không xoá (mặc định).",
  no_default_skip:
    "true = KHÔNG dùng nhận diện lịch sử mặc định (TẮT cả từ khóa LẪN pattern ngày yyyyMMdd); chỉ bỏ qua theo skip_dir.\n" +
    "false = dùng nhận diện lịch sử mặc định.",
  dry_run: "true = chỉ in danh sách, KHÔNG copy thật (xem trước).\nfalse = copy thật.",
};
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Config form -->
    <div class="min-h-0 flex-1 overflow-y-auto">
      <div class="flex flex-col gap-4">
        <!-- Section 1: Paths & Keyword -->
        <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <div class="flex items-center gap-2">
            <i class="pi pi-folder-open text-xl text-brand" />
            <h3 class="font-bold">Paths & Keyword</h3>
          </div>

          <div class="mt-4 grid gap-3">
            <label class="grid gap-1.5">
              <span class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wide text-muted">
                Input (FOLDER_ROOT)
                <i v-tooltip.top="HELP.input" class="pi pi-question-circle cursor-help text-brand opacity-70" style="font-size: 0.8rem" />
              </span>
              <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                <input
                  class="h-10 min-w-0 rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="D:\OUTPUT"
                  type="text"
                  :value="ctrl.config.value.input"
                  @input="ctrl.set('input', ($event.target as HTMLInputElement).value)"
                />
                <button
                  class="flex h-10 w-10 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
                  type="button"
                  title="Browse input folder"
                  @click="ctrl.pickFolder('input')"
                >
                  <i class="pi pi-folder-open" />
                </button>
              </div>
            </label>

            <label class="grid gap-1.5">
              <span class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wide text-muted">
                Output
                <i v-tooltip.top="HELP.output" class="pi pi-question-circle cursor-help text-brand opacity-70" style="font-size: 0.8rem" />
              </span>
              <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                <input
                  class="h-10 min-w-0 rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="D:\OUTPUT_VN"
                  type="text"
                  :value="ctrl.config.value.output"
                  @input="ctrl.set('output', ($event.target as HTMLInputElement).value)"
                />
                <button
                  class="flex h-10 w-10 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
                  type="button"
                  title="Browse output folder"
                  @click="ctrl.pickFolder('output')"
                >
                  <i class="pi pi-folder-open" />
                </button>
              </div>
            </label>

            <label class="grid gap-1.5">
              <span class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wide text-muted">
                Keyword
                <i v-tooltip.top="HELP.keyword" class="pi pi-question-circle cursor-help text-brand opacity-70" style="font-size: 0.8rem" />
              </span>
              <textarea
                class="min-w-0 resize-y rounded-md border border-divider bg-panel px-3 py-2 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                rows="3"
                placeholder="Copy: keyword filter (comma separated)&#10;Copy by folder: one folder name per line"
                :value="ctrl.config.value.keyword"
                @input="ctrl.set('keyword', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>
          </div>
        </section>

        <!-- Section 2: Settings -->
        <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <div class="flex items-center gap-2">
            <i class="pi pi-cog text-xl text-brand" />
            <h3 class="font-bold">Settings</h3>
          </div>

          <div class="mt-4 grid gap-3">
            <label class="grid gap-1.5">
              <span class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wide text-muted">
                Files (fixed list)
                <i v-tooltip.top="HELP.files" class="pi pi-question-circle cursor-help text-brand opacity-70" style="font-size: 0.8rem" />
              </span>
              <textarea
                class="min-w-0 resize-y rounded-md border border-divider bg-panel px-3 py-2 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                rows="2"
                placeholder="login_仕様書.xlsx"
                :value="ctrl.config.value.files"
                @input="ctrl.set('files', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>

            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
              <label class="grid gap-1.5">
                <span class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wide text-muted">
                  Ext
                  <i v-tooltip.top="HELP.ext" class="pi pi-question-circle cursor-help text-brand opacity-70" style="font-size: 0.8rem" />
                </span>
                <input
                  class="h-10 min-w-0 rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="xlsx, xlsm"
                  type="text"
                  :value="ctrl.config.value.ext"
                  @input="ctrl.set('ext', ($event.target as HTMLInputElement).value)"
                />
              </label>
              <label class="grid gap-1.5">
                <span class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wide text-muted">
                  Limit copy
                  <i v-tooltip.top="HELP.limit_copy" class="pi pi-question-circle cursor-help text-brand opacity-70" style="font-size: 0.8rem" />
                </span>
                <input
                  class="h-10 min-w-0 rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  type="number"
                  min="0"
                  :value="ctrl.config.value.limit_copy"
                  @input="ctrl.set('limit_copy', Number(($event.target as HTMLInputElement).value) || 0)"
                />
              </label>
            </div>

            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
              <label class="grid gap-1.5">
                <span class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wide text-muted">
                  Skip dir keywords
                  <i v-tooltip.top="HELP.skip_dir" class="pi pi-question-circle cursor-help text-brand opacity-70" style="font-size: 0.8rem" />
                </span>
                <input
                  class="h-10 min-w-0 rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="履歴, history, old"
                  type="text"
                  :value="ctrl.config.value.skip_dir"
                  @input="ctrl.set('skip_dir', ($event.target as HTMLInputElement).value)"
                />
              </label>
              <label class="grid gap-1.5">
                <span class="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wide text-muted">
                  Report dir
                  <i v-tooltip.top="HELP.report_dir" class="pi pi-question-circle cursor-help text-brand opacity-70" style="font-size: 0.8rem" />
                </span>
                <input
                  class="h-10 min-w-0 rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="reports"
                  type="text"
                  :value="ctrl.config.value.report_dir"
                  @input="ctrl.set('report_dir', ($event.target as HTMLInputElement).value)"
                />
              </label>
            </div>

            <!-- Toggle switches -->
            <div class="grid grid-cols-1 gap-2 sm:grid-cols-2 xl:grid-cols-3">
              <div
                v-for="toggle in [
                  { key: 'flat' as const, label: 'Flat' },
                  { key: 'group_by_parens' as const, label: 'Group by parens' },
                  { key: 'overwrite' as const, label: 'Overwrite' },
                  { key: 'create_history' as const, label: 'Create history' },
                  { key: 'delete_non_vn' as const, label: 'Delete non-VN' },
                  { key: 'no_default_skip' as const, label: 'No default skip' },
                  { key: 'dry_run' as const, label: 'Dry run' },
                ]"
                :key="toggle.key"
                class="flex items-center justify-between gap-2 rounded-md border border-divider bg-panel px-3 py-2"
              >
                <span class="flex items-center gap-1.5 text-sm font-medium">
                  {{ toggle.label }}
                  <i v-tooltip.top="HELP[toggle.key]" class="pi pi-question-circle cursor-help text-brand opacity-70" style="font-size: 0.75rem" />
                </span>
                <InputSwitch
                  :model-value="ctrl.config.value[toggle.key]"
                  @update:model-value="ctrl.set(toggle.key, !!$event)"
                />
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>

    <!-- Action bar -->
    <div class="flex items-center justify-between gap-3 rounded-lg border border-divider bg-panel px-4 py-3 shadow-sm">
      <button
        class="inline-flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-60"
        type="button"
        :disabled="ctrl.running.value || ctrl.runningFolders.value"
        @click="ctrl.loadFromIni()"
      >
        <i class="pi pi-refresh" />
        Reset
      </button>
      <div class="flex items-center gap-2">
        <button
          class="inline-flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-60"
          type="button"
          :disabled="ctrl.running.value"
          @click="ctrl.runByFolders()"
        >
          <i v-if="ctrl.runningFolders.value" class="pi pi-spin pi-spinner" />
          <i v-else class="pi pi-folder-open" />
          {{ ctrl.config.value.dry_run ? "Copy by folder (dry-run)" : "Copy by folder" }}
        </button>
        <button
          class="inline-flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
          type="button"
          :disabled="ctrl.runningFolders.value"
          @click="ctrl.run()"
        >
          <i v-if="ctrl.running.value" class="pi pi-spin pi-spinner" />
          <i v-else class="pi pi-copy" />
          {{ ctrl.config.value.dry_run ? "Copy (dry-run)" : "Copy" }}
        </button>
      </div>
    </div>

    <!-- Result log -->
    <section
      v-if="ctrl.showResult.value"
      class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm"
    >
      <div class="flex items-center justify-between gap-3 border-b border-divider px-4 py-3">
        <div class="flex items-center gap-2">
          <i class="pi pi-list text-brand" />
          <h3 class="font-bold">Result</h3>
        </div>
        <button
          class="text-xs font-semibold text-muted hover:text-ink"
          type="button"
          @click="ctrl.showResult.value = false"
        >
          Hide
        </button>
      </div>
      <pre class="min-h-0 max-h-80 flex-1 overflow-auto whitespace-pre-wrap break-words bg-slate-950 p-4 text-xs leading-6 text-slate-100">{{ ctrl.log.value.join("\n") }}</pre>
    </section>
  </section>
</template>
