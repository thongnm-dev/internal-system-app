<script setup lang="ts">
import { ref, computed } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import InputText from "primevue/inputtext";

interface AwsStorage {
  aws_cd: string;
  aws_name: string;
  aws_name_alias?: string;
  file_only?: boolean;
}

const props = defineProps<{
  awsStorage: AwsStorage;
}>();

const expanded = ref(true);
const items = ref<string[]>([]);
const isDownloadable = ref(false);
const isMoveable = ref(false);
const isLoading = ref(false);

const showDownloadModal = ref(false);
const showMoveModal = ref(false);
const destinationPath = ref("");
const errorCheck = ref("");
const selectedBugs = ref<Set<string>>(new Set());

const tableData = computed(() =>
  items.value.map((bug) => ({ bug_no: bug })),
);

const selectedBugsList = computed(() => Array.from(selectedBugs.value));

const modalTitle = computed(() => {
  if (showMoveModal.value) {
    return props.awsStorage.file_only ? "Xoá tập tin ở S3" : "Di chuyển file S3";
  }
  return "Chọn đường dẫn nơi lưu";
});

function toggle() {
  expanded.value = !expanded.value;
}

function handleRefresh() {
  // TODO: call backend
}

function handleDownload() {
  showDownloadModal.value = true;
}

function handleMove() {
  selectedBugs.value = new Set(items.value);
  showMoveModal.value = true;
}

function chooseDestinationFolder() {
  // TODO: call Tauri file dialog
}

function handleCancelModal() {
  showDownloadModal.value = false;
  showMoveModal.value = false;
}

function handleConfirm() {
  // TODO: call backend for download or move
  showDownloadModal.value = false;
  showMoveModal.value = false;
}
</script>

<template>
  <div class="grid grid-cols-1 rounded bg-surface-0 shadow dark:bg-surface-900">
    <!-- Header -->
    <div class="border-b border-surface-200 px-4 dark:border-surface-700">
      <div class="flex items-center justify-between gap-3">
        <button class="flex flex-1 items-center gap-4 bg-transparent py-2" @click="toggle">
          <i :class="['pi text-xl text-orange-500', expanded ? 'pi-folder-open' : 'pi-folder']" />
          <span class="text-lg font-bold text-surface-800 dark:text-surface-100">
            {{ awsStorage.aws_name_alias ?? awsStorage.aws_name }}
            <span class="text-red-600">({{ items.length }})</span>
          </span>
        </button>
        <div class="flex items-center gap-2 py-2">
          <Button
            label="Tải lại"
            icon="pi pi-refresh"
            severity="secondary"
            outlined
            size="small"
            @click="handleRefresh"
          />
          <Button
            v-if="items.length > 0 && isMoveable"
            :label="awsStorage.file_only ? 'Xoá trên S3' : 'Di chuyển trên S3'"
            icon="pi pi-eraser"
            severity="danger"
            outlined
            size="small"
            @click="handleMove"
          />
          <Button
            v-if="items.length > 0 && isDownloadable"
            label="Tải về"
            icon="pi pi-download"
            severity="warn"
            outlined
            size="small"
            @click="handleDownload"
          />
        </div>
      </div>
    </div>

    <!-- Content -->
    <div v-if="expanded" class="overflow-y-auto py-2">
      <DataTable
        :value="tableData"
        scrollable
        scroll-height="280px"
        size="small"
        striped-rows
        class="px-2"
      >
        <Column field="bug_no" header="Mã phiếu bug" />
      </DataTable>
    </div>
  </div>

  <!-- Download modal -->
  <Dialog
    v-model:visible="showDownloadModal"
    header="Chọn đường dẫn nơi lưu"
    :modal="true"
    :style="{ width: '36rem' }"
    :closable="true"
  >
    <div class="flex flex-col gap-4">
      <div class="flex items-center gap-2">
        <InputText
          v-model="destinationPath"
          class="flex-1 font-mono text-sm"
          placeholder="No directory selected"
          readonly
        />
        <Button label="..." severity="secondary" @click="chooseDestinationFolder" />
      </div>
      <small v-if="errorCheck" class="text-red-500">{{ errorCheck }}</small>
    </div>
    <template #footer>
      <Button label="Đóng" icon="pi pi-times" severity="secondary" @click="handleCancelModal" />
      <Button
        label="Bắt đầu..."
        icon="pi pi-check"
        :disabled="!destinationPath || !!errorCheck"
        @click="handleConfirm"
      />
    </template>
  </Dialog>

  <!-- Move / Delete modal -->
  <Dialog
    v-model:visible="showMoveModal"
    :header="modalTitle"
    :modal="true"
    :style="{ width: '36rem' }"
    :closable="true"
  >
    <div class="flex flex-col gap-4">
      <!-- Selected files preview -->
      <div class="rounded-lg bg-surface-50 p-4 dark:bg-surface-800">
        <h4 class="mb-2 text-sm font-semibold text-surface-700 dark:text-surface-200">
          Danh sách đã chọn:
        </h4>
        <div class="grid grid-cols-1 gap-2 md:grid-cols-2 lg:grid-cols-3">
          <div
            v-for="(bug, idx) in selectedBugsList.slice(0, 6)"
            :key="idx"
            class="truncate rounded bg-surface-100 px-3 py-2 text-sm text-surface-600 dark:bg-surface-700 dark:text-surface-300"
          >
            {{ bug }}
          </div>
          <div
            v-if="selectedBugsList.length > 6"
            class="rounded bg-surface-100 px-3 py-2 text-sm text-surface-500 dark:bg-surface-700"
          >
            ... và {{ selectedBugsList.length - 6 }} files.
          </div>
        </div>
      </div>

      <!-- S3 path -->
      <div>
        <h4 class="mb-1 text-sm font-semibold text-surface-700 dark:text-surface-200">
          Đường dẫn lưu ở S3
        </h4>
        <div class="rounded-lg border border-red-300 px-3 py-3 font-mono text-sm break-all">
          {{ awsStorage.aws_name }}
        </div>
      </div>
    </div>
    <template #footer>
      <Button label="Đóng" icon="pi pi-times" severity="secondary" @click="handleCancelModal" />
      <Button label="Bắt đầu..." icon="pi pi-check" @click="handleConfirm" />
    </template>
  </Dialog>
</template>
