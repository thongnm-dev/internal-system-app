<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import type { AwsStorage } from "@/_/types/s3";

const props = defineProps<{
  awsStorage: AwsStorage;
  ensureOnline: () => Promise<boolean>;
  getDownloadList: (code: string) => Promise<string[]>;
  selectFolder: () => Promise<string | null>;
  downloadFiles: (code: string, bugList: string[], localPath: string) => Promise<void>;
  moveObjects: (code: string, items: string[]) => Promise<void>;
  deleteObjects: (code: string, items: string[]) => Promise<void>;
}>();

const emit = defineEmits<{
  refreshed: [];
  downloaded: [path: string];
}>();

const STORAGE_KEY = "download_state";

const expanded = ref(true);
const items = ref<string[]>([]);
const isLoadingList = ref(false);

const showDownloadModal = ref(false);
const showMoveModal = ref(false);
const showMoveWarning = ref(false);
const showRedownloadWarning = ref(false);
const hasDownloaded = ref(false);
const destinationPath = ref("");
const errorCheck = ref("");
const selectedBugs = ref<Set<string>>(new Set());
const cancelBtnRef = ref<{ $el: HTMLElement } | null>(null);

const tableData = computed(() =>
  items.value.map((bug) => ({ bug_no: bug })),
);

const selectedBugsList = computed(() => Array.from(selectedBugs.value));

const hasItems = computed(() => items.value.length > 0);

const modalTitle = computed(() => {
  if (showMoveModal.value) {
    return props.awsStorage.fileOnly ? "Xoá tập tin ở S3" : "Di chuyển file S3";
  }
  return "Chọn đường dẫn nơi lưu";
});

function toggle() {
  expanded.value = !expanded.value;
}

function loadSavedPath() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      const state = JSON.parse(saved);
      if (state[props.awsStorage.code]?.localPathSync) {
        destinationPath.value = state[props.awsStorage.code].localPathSync;
      }
    }
  } catch {
    // ignore
  }
}

function savePath() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    const state = saved ? JSON.parse(saved) : {};
    state[props.awsStorage.code] = { localPathSync: destinationPath.value };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // ignore
  }
}

async function loadItems() {
  if (!(await props.ensureOnline())) return;
  isLoadingList.value = true;
  try {
    items.value = await props.getDownloadList(props.awsStorage.code);
  } finally {
    isLoadingList.value = false;
  }
}

async function handleRefresh() {
  await loadItems();
  emit("refreshed");
}

async function handleDownload() {
  if (!(await props.ensureOnline())) return;
  if (hasDownloaded.value) {
    showRedownloadWarning.value = true;
    return;
  }
  openDownloadModal();
}

function openDownloadModal() {
  loadSavedPath();
  errorCheck.value = "";
  showDownloadModal.value = true;
}

function confirmRedownload() {
  showRedownloadWarning.value = false;
  openDownloadModal();
}

function dismissRedownloadWarning() {
  showRedownloadWarning.value = false;
}

async function handleMove() {
  if (!(await props.ensureOnline())) return;
  selectedBugs.value = new Set(items.value);
  if (!hasDownloaded.value) {
    showMoveWarning.value = true;
    return;
  }
  showMoveModal.value = true;
}

function confirmMoveWarning() {
  showMoveWarning.value = false;
  showMoveModal.value = true;
}

function dismissMoveWarning() {
  showMoveWarning.value = false;
}

async function chooseDestinationFolder() {
  const dir = await props.selectFolder();
  if (dir) {
    destinationPath.value = dir;
    errorCheck.value = "";
    savePath();
  }
}

function handleCancelModal() {
  showDownloadModal.value = false;
  showMoveModal.value = false;
}

async function handleConfirmDownload() {
  if (!destinationPath.value) {
    errorCheck.value = "Vui lòng chọn đường dẫn.";
    return;
  }
  showDownloadModal.value = false;
  await props.downloadFiles(props.awsStorage.code, items.value, destinationPath.value);
  hasDownloaded.value = true;
  emit("downloaded", destinationPath.value);
  await loadItems();
  emit("refreshed");
}

async function handleConfirmMove() {
  showMoveModal.value = false;
  if (props.awsStorage.fileOnly) {
    await props.deleteObjects(props.awsStorage.code, Array.from(selectedBugs.value));
  } else {
    await props.moveObjects(props.awsStorage.code, Array.from(selectedBugs.value));
  }
  await loadItems();
  emit("refreshed");
}

watch(showMoveWarning, (visible) => {
  if (visible) {
    nextTick(() => {
      cancelBtnRef.value?.$el?.focus();
    });
  }
});

onMounted(() => {
  loadItems();
});
</script>

<template>
  <div class="grid grid-cols-1 rounded bg-surface-0 shadow dark:bg-surface-900">
    <!-- Header -->
    <div class="border-b border-surface-200 px-4 dark:border-surface-700">
      <div class="flex items-center justify-between gap-3">
        <Button class="flex flex-1 items-center gap-4 bg-transparent py-2" unstyled @click="toggle">
          <i :class="['pi text-xl text-orange-500', expanded ? 'pi-folder-open' : 'pi-folder']" />
          <span class="text-lg font-bold text-surface-800 dark:text-surface-100">
            {{ awsStorage.nameAlias || awsStorage.name }}
            <span class="text-red-600">({{ items.length }})</span>
          </span>
        </Button>
        <div class="flex items-center gap-2 py-2">
          <Button
            label="Tải lại"
            icon="pi pi-refresh"
            severity="secondary"
            outlined
            size="small"
            :loading="isLoadingList"
            @click="handleRefresh"
          />
          <Button
            v-if="hasItems"
            :label="awsStorage.fileOnly ? 'Xoá trên S3' : 'Di chuyển trên S3'"
            icon="pi pi-eraser"
            severity="danger"
            outlined
            size="small"
            @click="handleMove"
          />
          <Button
            v-if="hasItems"
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
        <Button icon="pi pi-folder-open" severity="secondary" @click="chooseDestinationFolder" />
      </div>
      <small v-if="errorCheck" class="text-red-500">{{ errorCheck }}</small>
    </div>
    <template #footer>
      <Button label="Đóng" icon="pi pi-times" severity="secondary" @click="handleCancelModal" />
      <Button
        label="Bắt đầu..."
        icon="pi pi-check"
        :disabled="!destinationPath || !!errorCheck"
        @click="handleConfirmDownload"
      />
    </template>
  </Dialog>

  <!-- Redownload warning dialog -->
  <Dialog
    v-model:visible="showRedownloadWarning"
    header="Cảnh báo"
    :modal="true"
    :style="{ width: '28rem' }"
    :closable="true"
  >
    <div class="flex items-center gap-3">
      <i class="pi pi-exclamation-triangle text-3xl text-yellow-500" />
      <span class="text-sm text-surface-600 dark:text-surface-400">
        Bạn đã tải về danh sách phiếu bug rồi. Bạn có muốn tải lại không?
      </span>
    </div>
    <template #footer>
      <Button
        label="Cancel"
        icon="pi pi-times"
        severity="secondary"
        autofocus
        @click="dismissRedownloadWarning"
      />
      <Button label="Đồng ý" icon="pi pi-check" severity="warn" @click="confirmRedownload" />
    </template>
  </Dialog>

  <!-- Move warning dialog -->
  <Dialog
    v-model:visible="showMoveWarning"
    header="Cảnh báo"
    :modal="true"
    :style="{ width: '28rem' }"
    :closable="true"
  >
    <div class="flex items-center gap-3">
      <i class="pi pi-exclamation-triangle text-3xl text-yellow-500" />
      <span class="text-sm text-surface-600 dark:text-surface-400">
        Bạn chưa tải về máy. Bạn có chắc chắn muốn thực hiện thao tác này không?
      </span>
    </div>
    <template #footer>
      <Button
        ref="cancelBtnRef"
        label="Cancel"
        icon="pi pi-times"
        severity="secondary"
        @click="dismissMoveWarning"
      />
      <Button label="Đồng ý" icon="pi pi-check" severity="danger" @click="confirmMoveWarning" />
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
          {{ awsStorage.name }}
        </div>
      </div>
    </div>
    <template #footer>
      <Button label="Đóng" icon="pi pi-times" severity="secondary" @click="handleCancelModal" />
      <Button label="Bắt đầu..." icon="pi pi-check" @click="handleConfirmMove" />
    </template>
  </Dialog>
</template>
