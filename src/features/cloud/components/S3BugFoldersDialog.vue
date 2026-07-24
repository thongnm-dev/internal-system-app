<script setup lang="ts">
import { ref } from "vue";
import Dialog from "primevue/dialog";
import Tabs from "primevue/tabs";
import TabList from "primevue/tablist";
import Tab from "primevue/tab";
import TabPanels from "primevue/tabpanels";
import TabPanel from "primevue/tabpanel";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Button from "primevue/button";
import ProgressSpinner from "primevue/progressspinner";
import { useS3BugFolders } from "../composables/useS3BugFolders";
import type { BugFolderTab } from "@/_/types/s3";

const visible = ref(true);
const emit = defineEmits<{ close: [] }>();

const DISPLAY_LIMIT = 100;

const {
  tabsWithItems,
  totalBugCount,
  isLoading,
  isRefreshing,
  refresh,
} = useS3BugFolders();

const activeTab = ref("0");

function displayItems(tab: BugFolderTab) {
  return tab.items.slice(0, DISPLAY_LIMIT);
}

function overflowCount(tab: BugFolderTab) {
  return Math.max(0, tab.items.length - DISPLAY_LIMIT);
}

function handleHide() {
  emit("close");
}
</script>

<template>
  <Dialog
    v-model:visible="visible"
    header="Trạng thái S3 Bug Folders"
    :modal="true"
    :style="{ width: '64rem' }"
    :closable="true"
    :content-style="{ minHeight: '400px' }"
    @hide="handleHide"
  >
    <!-- Loading -->
    <div v-if="isLoading" class="flex h-64 items-center justify-center">
      <ProgressSpinner style="width: 40px; height: 40px" />
    </div>

    <template v-else>
      <!-- Header -->
      <div class="mb-3 flex items-center justify-between rounded-lg bg-panel px-4 py-3 shadow-card">
        <div class="flex items-center gap-3">
          <i class="pi pi-folder-open text-xl text-orange-500" />
          <span class="text-sm font-semibold text-ink">
            Tổng cộng
            <span class="text-red-600">{{ totalBugCount }}</span>
            thư mục bug trên
            <span class="text-blue-600">{{ tabsWithItems.length }}</span>
            nơi lưu trữ
          </span>
        </div>
        <Button
          label="Tải lại"
          icon="pi pi-refresh"
          severity="secondary"
          outlined
          size="small"
          :loading="isRefreshing"
          @click="refresh"
        />
      </div>

      <!-- Tabs -->
      <div v-if="tabsWithItems.length > 0">
        <Tabs v-model:value="activeTab">
          <TabList>
            <Tab
              v-for="(tab, idx) in tabsWithItems"
              :key="tab.name"
              :value="String(idx)"
            >
              {{ tab.nameAlias || tab.name }}
              <span class="ml-1 text-red-600">({{ tab.items.length }})</span>
            </Tab>
          </TabList>
          <TabPanels>
            <TabPanel
              v-for="(tab, idx) in tabsWithItems"
              :key="tab.name"
              :value="String(idx)"
            >
              <div class="p-3">
                <DataTable
                  :value="displayItems(tab)"
                  scrollable
                  scroll-height="320px"
                  size="small"
                  striped-rows
                  class="text-sm"
                >
                  <Column field="bugNo" header="Mã phiếu bug">
                    <template #body="{ data }">
                      <span>{{ data.bugNo }}</span>
                      <span
                        v-if="data.inSubscribe"
                        class="ml-2 text-xs text-red-500"
                      >
                        chưa di chuyển
                      </span>
                    </template>
                  </Column>
                  <template v-if="overflowCount(tab) > 0" #footer>
                    <div class="text-center text-sm text-muted">
                      Và <span class="font-semibold text-blue-600">{{ overflowCount(tab) }}</span> tài liệu.
                      Hãy truy cập S3 để xem chi tiết.
                    </div>
                  </template>
                </DataTable>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>

      <!-- Empty state -->
      <div
        v-else
        class="flex h-64 flex-col items-center justify-center text-muted"
      >
        <i class="pi pi-inbox mb-4 text-5xl" />
        <span class="text-sm">Không có thư mục bug nào trên S3.</span>
      </div>
    </template>

    <template #footer>
      <Button label="Đóng" icon="pi pi-times" severity="secondary" @click="visible = false" />
    </template>
  </Dialog>
</template>
