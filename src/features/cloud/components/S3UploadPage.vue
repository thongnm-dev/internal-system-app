<script setup lang="ts">
import { ref } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Select from "primevue/select";
import S3UploadCard from "./S3UploadCard.vue";

interface AwsStorage {
  aws_cd: string;
  aws_name: string;
  aws_name_alias?: string;
}

interface FileItem {
  file_id?: number;
  parent_name: string;
  sub_folder?: string;
  name: string;
  file_path: string;
  full_path: string;
  file_size?: number;
}

interface UploadItem {
  aws_cd?: string;
  bug_no?: string;
}

// STUB data — remove after backend wiring
const listUploadItems = ref<AwsStorage[]>([
  { aws_cd: "01", aws_name: "s3://project-bucket/upload/release", aws_name_alias: "Release Upload" },
  { aws_cd: "011", aws_name: "s3://project-bucket/upload/hotfix", aws_name_alias: "Hotfix Upload" },
]);
const uploadedId = ref("");

const openModal = ref(false);
const isUpdating = ref(false);
const modalTitle = ref("");
const modalHeaderTitle = ref("");
const destination = ref<AwsStorage>({} as AwsStorage);
const createFolderSameName = ref(false);
const uploadFileItems = ref<FileItem[]>([]);
const deleteItems = ref<UploadItem[]>([]);
const deleteOptions = ref<AwsStorage[]>([]);

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
  selected_items: FileItem[];
}) {
  modalHeaderTitle.value = params.aws_storage.aws_name;
  createFolderSameName.value = params.is_folder_same_name;
  destination.value = params.aws_storage;
  modalTitle.value = "Tải lên S3 AWS";
  uploadFileItems.value = params.selected_items;
  openModal.value = true;
  isUpdating.value = true;
}

function handleConfirm() {
  // TODO: call backend for upload or delete
  openModal.value = false;
}

function handleCloseModal() {
  openModal.value = false;
}

function updateDeleteItemAwsCd(bugNo: string, newValue: string) {
  deleteItems.value = deleteItems.value.map((item) =>
    item.bug_no === bugNo ? { ...item, aws_cd: newValue } : item,
  );
}
</script>

<template>
  <div class="space-y-4">
    <!-- Upload cards -->
    <div v-if="listUploadItems.length > 0" class="grid grid-cols-1 gap-3">
      <S3UploadCard
        v-for="(item, index) in listUploadItems"
        :key="index"
        :aws-storage="item"
        :uploaded-id="uploadedId"
        @upload="uploadAction"
        @clear="uploadedId = ''"
      />
    </div>

    <!-- Empty state -->
    <div v-else class="flex h-full flex-col items-center justify-center rounded-lg bg-surface-0 py-16 dark:bg-surface-900">
      <i class="pi pi-cloud-upload mb-4 text-5xl text-surface-300" />
      <span class="text-surface-500">Chưa có cấu hình nơi tải lên.</span>
    </div>
  </div>

  <!-- Upload / Delete confirmation modal -->
  <Dialog
    v-model:visible="openModal"
    :header="modalTitle"
    :modal="true"
    :style="{ width: '48rem' }"
    :closable="true"
  >
    <!-- Upload header -->
    <div v-if="isUpdating" class="mb-3 flex items-center gap-2 rounded border border-surface-200 bg-surface-0 p-4 dark:border-surface-700 dark:bg-surface-900">
      <span class="font-bold">Bạn đang thực hiện tải các tập tin lên đường dẫn sau:</span>
      <span class="font-bold text-red-600">{{ modalHeaderTitle }}</span>
    </div>

    <!-- Delete header -->
    <div v-if="!isUpdating && deleteOptions.length === 1" class="mb-3 flex items-center gap-2 rounded border border-surface-200 bg-surface-0 p-4 dark:border-surface-700 dark:bg-surface-900">
      <span class="font-bold">Bạn đang thực hiện xoá các tập tin lên đường dẫn sau:</span>
      <span class="font-bold text-red-600">{{ deleteOptions[0]?.aws_name }}</span>
    </div>

    <!-- Folder same name checkbox -->
    <div v-if="destination.aws_cd === '011'" class="mb-3 flex items-center gap-2 p-2">
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
    <div v-if="isUpdating" class="rounded-lg shadow">
      <DataTable
        :value="uploadFileItems.map((f) => ({ name: f.name, size: f.file_size ?? 0, file_path: f.file_path }))"
        scrollable
        scroll-height="300px"
        size="small"
        striped-rows
      >
        <Column field="name" header="Tên tập tin">
          <template #body="{ data }">
            <div class="flex items-center gap-2">
              <i class="pi pi-file-excel text-lg text-green-600" />
              <span class="font-medium text-surface-900 dark:text-surface-100">{{ data.name }}</span>
            </div>
          </template>
        </Column>
        <Column field="size" header="Kích thước">
          <template #body="{ data }">
            <span class="text-surface-600 dark:text-surface-400">{{ formatFileSize(data.size) }}</span>
          </template>
        </Column>
      </DataTable>
    </div>

    <!-- Delete items list -->
    <div v-if="!isUpdating" class="rounded-lg shadow">
      <DataTable
        :value="deleteItems.map((f) => ({ bug_no: f.bug_no, aws_cd: f.aws_cd }))"
        scrollable
        scroll-height="300px"
        size="small"
        striped-rows
      >
        <Column field="aws_cd" header="Đích nơi xoá">
          <template #body="{ data }">
            <Select
              :model-value="data.aws_cd"
              :options="deleteOptions"
              option-label="aws_name"
              option-value="aws_cd"
              :disabled="destination.aws_cd === '05'"
              class="w-full"
              @update:model-value="(v: string) => updateDeleteItemAwsCd(data.bug_no, v)"
            />
          </template>
        </Column>
        <Column field="bug_no" header="Đối tượng xoá" />
      </DataTable>
    </div>

    <template #footer>
      <Button label="Đóng" icon="pi pi-times" severity="secondary" @click="handleCloseModal" />
      <Button label="Bắt đầu..." icon="pi pi-check" @click="handleConfirm" />
    </template>
  </Dialog>
</template>
