<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import S3DownloadCard from "./S3DownloadCard.vue";
import { useCloudGuard } from "../composables/useCloudGuard";

interface AwsStorage {
  aws_cd: string;
  aws_name: string;
  aws_name_alias?: string;
  file_only?: boolean;
}

interface DownloadItem {
  id: string;
  download_ymd?: string;
  download_hms?: string;
  sync_path?: string;
  download_count?: number;
  aws_name?: string;
}

const guard = useCloudGuard();

const listDownloadItems = ref<AwsStorage[]>([]);
const downloadItems = ref<DownloadItem[]>([]);
const downloadable = ref<Record<string, { download_available: boolean }>>({});
const isReloading = ref(false);

const hasDownloadable = computed(() => {
  if (Object.keys(downloadable.value).length === 0) return false;
  return listDownloadItems.value.some(
    (s) => downloadable.value[s.aws_cd]?.download_available,
  );
});

const downloadableStorages = computed(() =>
  listDownloadItems.value.filter(
    (s) => downloadable.value[s.aws_cd]?.download_available,
  ),
);

const hasHistory = computed(() => downloadItems.value.length > 0);

async function handleRefresh() {
  if (!(await guard.ensureOnline())) return;
  isReloading.value = true;
  // TODO: call backend to refresh
  isReloading.value = false;
}

onMounted(async () => {
  if (!(await guard.ensureOnline())) return;
  // TODO: load download storages from backend
});
</script>

<template>
  <div class="flex h-full flex-col gap-4">
    <!-- Downloadable items -->
    <template v-if="hasDownloadable">
      <div class="flex flex-col gap-3 rounded-lg shadow">
        <S3DownloadCard
          v-for="storage in downloadableStorages"
          :key="storage.aws_cd"
          :aws-storage="storage"
          :ensure-online="guard.ensureOnline"
        />
      </div>
    </template>

    <!-- Empty state -->
    <template v-else>
      <div class="flex h-full flex-col items-center justify-center rounded-lg bg-surface-0 dark:bg-surface-900 text-lg text-surface-500">
        <i class="pi pi-cloud-download mb-4 text-5xl text-surface-300" />
        <span class="animate-bounce py-4 text-sm text-red-500">
          Không có tập tin nào để tải về...
        </span>
        <Button
          label="Làm mới trạng thái"
          icon="pi pi-refresh"
          severity="secondary"
          outlined
          :loading="isReloading"
          @click="handleRefresh"
        />
      </div>
    </template>

    <!-- Download history -->
    <div v-if="hasHistory" class="rounded-lg border border-surface-200 dark:border-surface-700">
      <div class="border-b border-surface-200 dark:border-surface-700 px-4 py-3">
        <h3 class="text-sm font-semibold text-surface-700 dark:text-surface-200">
          Thông tin lịch sử đã tải về
        </h3>
      </div>
      <DataTable
        :value="downloadItems"
        scrollable
        scroll-height="400px"
        size="small"
        striped-rows
      >
        <Column field="id" header="ID">
          <template #body="{ data }">
            <span class="cursor-pointer text-blue-500 hover:text-blue-700">
              #{{ data.id }}
            </span>
          </template>
        </Column>
        <Column field="download_time" header="Thời gian">
          <template #body="{ data }">
            {{ (data.download_ymd ?? '') + (data.download_hms ?? '') }}
          </template>
        </Column>
        <Column field="aws_name" header="Trạng thái S3" />
        <Column field="download_count" header="Số lượng tập tin đã tải" />
      </DataTable>
    </div>

    <!-- Offline Dialog -->
    <Dialog
      v-model:visible="guard.showOfflineDialog.value"
      header="Lỗi kết nối"
      :modal="true"
      :closable="true"
      :style="{ width: '28rem' }"
    >
      <div class="flex items-center gap-3">
        <i class="pi pi-wifi text-3xl text-red-500" />
        <span class="text-sm text-surface-600 dark:text-surface-400">{{ guard.offlineMessage }}</span>
      </div>
      <template #footer>
        <Button label="Đóng" @click="guard.dismissOfflineDialog()" />
      </template>
    </Dialog>
  </div>
</template>
