<script setup lang="ts">
import { watch } from "vue";
import Button from "primevue/button";
import Column from "primevue/column";
import DataTable from "primevue/datatable";
import Dialog from "primevue/dialog";
import type { GitApi } from "../composables/useGit";
import { useDataTablePagination } from "@/shared/composables/useDataTablePagination";

const props = defineProps<{ git: GitApi }>();
const { paginationCompact } = useDataTablePagination();
const visible = defineModel<boolean>("visible", { default: false });

const emit = defineEmits<{ "create-tag": [] }>();

watch(visible, (v) => {
  if (v) props.git.loadTags();
});
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Quản lý tags" :style="{ width: '1000px' }">
    <DataTable
      :value="git.tags.value"
      paginator
      :rows="paginationCompact.rows"
      :rows-per-page-options="paginationCompact.rowsPerPageOptions"
      :paginator-template="paginationCompact.paginatorTemplate"
      :current-page-report-template="paginationCompact.currentPageReportTemplate"
      scrollable
      scroll-height="550px"
      empty-message="Chưa có tag nào."
      class="app-data-table"
    >
      <Column header="Tag" field="name" sortable>
        <template #body="{ data }">
          <div class="flex items-center gap-2">
            <i class="pi pi-tag shrink-0 text-xs text-brand" />
            <span class="font-medium text-ink">{{ data.name }}</span>
          </div>
        </template>
      </Column>
      <Column header="SHA" field="target" style="width: 100px">
        <template #body="{ data }">
          <span class="font-mono text-xs text-muted">{{ data.target }}</span>
        </template>
      </Column>
      <Column header="Message" field="subject">
        <template #body="{ data }">
          <span class="text-sm text-secondary">{{ data.subject || "—" }}</span>
        </template>
      </Column>
      <Column header="Ngày" field="date" sortable style="width: 140px">
        <template #body="{ data }">
          <span class="text-xs text-muted">{{ data.date || "—" }}</span>
        </template>
      </Column>
      <Column header="" style="width: 90px">
        <template #body="{ data }">
          <div class="flex items-center gap-1">
            <Button
              size="small"
              text
              severity="secondary"
              title="Copy tên tag"
              @click="git.copyText(data.name, 'tên tag')"
            >
              <i class="pi pi-copy" />
            </Button>
            <Button
              size="small"
              text
              severity="danger"
              title="Xóa tag (local)"
              @click="git.deleteTag(data.name, false)"
            >
              <i class="pi pi-trash" />
            </Button>
          </div>
        </template>
      </Column>
    </DataTable>
    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Đóng</Button>
      <Button size="small" @click="emit('create-tag')">
        <i class="pi pi-plus mr-1.5" /> Tạo tag mới
      </Button>
    </template>
  </Dialog>
</template>
