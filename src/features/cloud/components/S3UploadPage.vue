<script setup lang="ts">
import { ref, nextTick, watch } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Select from "primevue/select";
import ProgressSpinner from "primevue/progressspinner";
import S3UploadCard from "./S3UploadCard.vue";
import S3ConfigError from "./S3ConfigError.vue";
import S3BugFoldersDialog from "./S3BugFoldersDialog.vue";
import { useS3Upload } from "../composables/useS3Upload";
import { useS3ConfigGuard } from "../composables/useS3ConfigGuard";
import type { AwsStorage, ScannedFile, UploadFileRequest } from "@/_/types/s3";

const s3Guard = useS3ConfigGuard();
s3Guard.checkConfig();

const {
  uploadStorages,
  isLoading,
  isUploading,
  isDeleting,
  deleteItems,
  deleteOptions,
  showDeleteDialog,
  showOfflineDialog,
  offlineMessage,
  dismissOfflineDialog,
  scanFolder,
  uploadFiles,
  confirmDelete,
  dismissDeleteDialog,
  updateDeleteItemCode,
} = useS3Upload();

const uploadedId = ref("");
const showS3ConfirmDialog = ref(false);
const showBugFoldersDialog = ref(false);
const pendingS3Confirm = ref(false);

const openModal = ref(false);
const modalTitle = ref("");
const modalHeaderTitle = ref("");
const destination = ref<AwsStorage | null>(null);
const createFolderSameName = ref(false);
const uploadFileItems = ref<ScannedFile[]>([]);

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function uploadAction(params: {
  aws_storage: AwsStorage;
  is_folder_same_name: boolean;
  selected_items: ScannedFile[];
}) {
  modalHeaderTitle.value = params.aws_storage.name || params.aws_storage.nameAlias;
  createFolderSameName.value = params.is_folder_same_name;
  destination.value = params.aws_storage;
  modalTitle.value = "Tải lên S3 AWS";
  uploadFileItems.value = params.selected_items;
  openModal.value = true;
}

async function handleScanFolder(callback: (files: ScannedFile[]) => void) {
  const files = await scanFolder();
  callback(files);
}

async function handleConfirm() {
  if (!destination.value || uploadFileItems.value.length === 0) return;

  const files: UploadFileRequest[] = uploadFileItems.value.map((f) => ({
    parentName: f.parentName,
    name: f.name,
    localPath: f.filePath,
  }));

  openModal.value = false;

  const result = await uploadFiles(files, destination.value, createFolderSameName.value);
  if (result?.success) {
    uploadedId.value = destination.value.code;
  }

  if (deleteItems.value.length > 0) {
    pendingS3Confirm.value = !!result?.success;
    await nextTick();
    showDeleteDialog.value = true;
  } else if (result?.success) {
    showS3ConfirmDialog.value = true;
  }
}

watch(showDeleteDialog, (val) => {
  if (!val && pendingS3Confirm.value) {
    pendingS3Confirm.value = false;
    showS3ConfirmDialog.value = true;
  }
});

function handleCloseModal() {
  openModal.value = false;
}

</script>

<template>
  <S3ConfigError
    v-if="s3Guard.configError.value"
    :error="s3Guard.configError.value"
    :is-checking="s3Guard.configChecking.value"
    @retry="s3Guard.checkConfig()"
  />

  <div v-else class="space-y-4">
    <!-- Loading -->
    <div v-if="isLoading" class="flex items-center justify-center py-16">
      <ProgressSpinner style="width: 40px; height: 40px" />
    </div>

    <!-- Upload cards -->
    <div v-else-if="uploadStorages.length > 0" class="grid grid-cols-1 gap-3">
      <S3UploadCard
        v-for="item in uploadStorages"
        :key="item.code"
        :aws-storage="item"
        :uploaded-id="uploadedId"
        @upload="uploadAction"
        @clear="uploadedId = ''"
        @scan-folder="handleScanFolder"
      />
    </div>

    <!-- Empty state -->
    <div v-else class="flex h-full flex-col items-center justify-center rounded-lg bg-surface-0 py-16 dark:bg-surface-900">
      <i class="pi pi-cloud-upload mb-4 text-5xl text-surface-300" />
      <span class="text-surface-500">Chưa có cấu hình nơi tải lên.</span>
    </div>

  </div>

  <!-- Offline Dialog -->
  <Dialog
    v-model:visible="showOfflineDialog"
    header="Lỗi kết nối"
    :modal="true"
    :closable="true"
    :style="{ width: '28rem' }"
  >
    <div class="flex items-center gap-3">
      <i class="pi pi-wifi text-3xl text-red-500" />
      <span class="text-sm">{{ offlineMessage }}</span>
    </div>
    <template #footer>
      <Button label="Đóng" @click="dismissOfflineDialog()" />
    </template>
  </Dialog>

  <!-- Upload confirmation modal -->
  <Dialog
    v-model:visible="openModal"
    :header="modalTitle"
    :modal="true"
    :style="{ width: '48rem' }"
    :closable="true"
  >
    <!-- Upload header -->
    <div class="mb-3 flex items-center gap-2 rounded border border-surface-200 bg-surface-0 p-4 dark:border-surface-700 dark:bg-surface-900">
      <span class="font-bold">Bạn đang thực hiện tải các tập tin lên đường dẫn sau:</span>
      <span class="font-bold text-red-600">{{ modalHeaderTitle }}</span>
    </div>

    <!-- Folder same name checkbox -->
    <div v-if="destination?.code === '011'" class="mb-3 flex items-center gap-2 p-2">
      <Checkbox
        v-model="createFolderSameName"
        :binary="true"
        disabled
        input-id="chkCreateFolderSameName"
      />
      <label for="chkCreateFolderSameName" class="text-sm font-bold text-red-600">
        Tạo thư mục tương ứng với tên tập tin
      </label>
    </div>

    <!-- Upload file list -->
    <div class="rounded-lg shadow">
      <DataTable
        :value="uploadFileItems.map((f) => ({ name: f.name, size: f.fileSize ?? 0, parentName: f.parentName }))"
        scrollable
        scroll-height="300px"
        size="small"
        striped-rows
      >
        <Column field="parentName" header="Thư mục">
          <template #body="{ data }">
            <div class="flex items-center gap-2">
              <i class="pi pi-folder text-lg text-orange-500" />
              <span class="font-medium text-surface-900 dark:text-surface-100">{{ data.parentName }}</span>
            </div>
          </template>
        </Column>
        <Column field="name" header="Tên tập tin">
          <template #body="{ data }">
            <div class="flex items-center gap-2">
              <i class="pi pi-file text-lg text-blue-500" />
              <span class="font-medium text-surface-900 dark:text-surface-100">{{ data.name }}</span>
            </div>
          </template>
        </Column>
        <Column field="size" header="Kích thước" style="width: 120px">
          <template #body="{ data }">
            <span class="text-surface-600 dark:text-surface-400">{{ formatFileSize(data.size) }}</span>
          </template>
        </Column>
      </DataTable>
    </div>

    <template #footer>
      <Button label="Đóng" icon="pi pi-times" severity="secondary" @click="handleCloseModal" />
      <Button
        label="Bắt đầu tải lên"
        icon="pi pi-upload"
        :loading="isUploading"
        @click="handleConfirm"
      />
    </template>
  </Dialog>

  <!-- S3 Status Confirm Dialog -->
  <Dialog
    v-model:visible="showS3ConfirmDialog"
    header="Xác nhận"
    :modal="true"
    :closable="true"
    :style="{ width: '28rem' }"
  >
    <div class="flex items-center gap-3">
      <i class="pi pi-question-circle text-3xl text-blue-500" />
      <span class="text-sm text-surface-600 dark:text-surface-400">
        Bạn có muốn mở màn hình để xem trạng thái trên S3 không?
      </span>
    </div>
    <template #footer>
      <Button label="Cancel" icon="pi pi-times" severity="secondary" @click="showS3ConfirmDialog = false" />
      <Button label="OK" icon="pi pi-check" @click="showS3ConfirmDialog = false; showBugFoldersDialog = true" />
    </template>
  </Dialog>

  <!-- S3 Bug Folders Dialog -->
  <S3BugFoldersDialog v-if="showBugFoldersDialog" @close="showBugFoldersDialog = false" />

  <!-- Delete after upload dialog -->
  <Dialog
    v-model:visible="showDeleteDialog"
    header="Thực hiện xoá tập tin S3"
    :modal="true"
    :style="{ width: '48rem' }"
    :closable="true"
    @hide="dismissDeleteDialog"
  >
    <div v-if="deleteOptions.length === 1" class="mb-3 flex items-center gap-2 rounded border border-surface-200 bg-surface-0 p-4 dark:border-surface-700 dark:bg-surface-900">
      <span class="font-bold">Bạn đang thực hiện xoá các tập tin ở đường dẫn sau:</span>
      <span class="font-bold text-red-600">{{ deleteOptions[0].name }}</span>
    </div>

    <div class="rounded-lg shadow">
      <DataTable
        :value="deleteItems"
        scrollable
        scroll-height="300px"
        size="small"
        striped-rows
      >
        <Column header="Đích nơi xoá" style="width: 50%">
          <template #body="{ data }">
            <Select
              :model-value="data.awsCd"
              :options="deleteOptions"
              option-label="name"
              option-value="code"
              :disabled="deleteOptions.length <= 1"
              class="w-full"
              @update:model-value="(val: string) => updateDeleteItemCode(data.bugNo, val)"
            />
          </template>
        </Column>
        <Column header="Đối tượng xoá" style="width: 50%">
          <template #body="{ data }">
            <div class="flex items-center gap-2">
              <i class="pi pi-folder text-lg text-orange-500" />
              <span class="font-medium text-surface-900 dark:text-surface-100">{{ data.bugNo }}</span>
            </div>
          </template>
        </Column>
      </DataTable>
    </div>

    <template #footer>
      <Button label="Bỏ qua" icon="pi pi-times" severity="secondary" @click="dismissDeleteDialog" />
      <Button
        label="Xác nhận xoá"
        icon="pi pi-trash"
        severity="danger"
        :loading="isDeleting"
        @click="confirmDelete"
      />
    </template>
  </Dialog>
</template>
