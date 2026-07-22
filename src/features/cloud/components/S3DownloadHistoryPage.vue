<script setup lang="ts">
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Select from "primevue/select";
import Checkbox from "primevue/checkbox";
import DatePicker from "primevue/datepicker";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import { useS3DownloadHistory } from "../composables/useS3DownloadHistory";

const { storages, results, isSearching, params, search, clearSearch } =
  useS3DownloadHistory();

function formatDateForParam(date: Date | null): string {
  if (!date) return "";
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}${m}${d}`;
}

function parseDateFromParam(val: string): Date | null {
  if (!val || val.length !== 8) return null;
  return new Date(
    Number(val.slice(0, 4)),
    Number(val.slice(4, 6)) - 1,
    Number(val.slice(6, 8)),
  );
}

function onFromDateChange(
  val: Date | Date[] | (Date | null)[] | null | undefined,
) {
  const date = val instanceof Date ? val : null;
  params.value.fromDate = formatDateForParam(date);
}

function onToDateChange(
  val: Date | Date[] | (Date | null)[] | null | undefined,
) {
  const date = val instanceof Date ? val : null;
  params.value.toDate = formatDateForParam(date);
}

function formatDate(ymd: string): string {
  if (ymd.length !== 8) return ymd;
  return `${ymd.slice(0, 4)}/${ymd.slice(4, 6)}/${ymd.slice(6, 8)}`;
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Search -->
    <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Search" toggleable>
      <div class="grid gap-3">
        <div class="grid gap-3 lg:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Nguồn tải về</span>
            <Select
              v-model="params.awsCd"
              :options="[{ code: '', name: 'Tất cả' }, ...storages]"
              option-label="name"
              option-value="code"
              placeholder="Tất cả"
              class="mt-1 w-full"
            />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Bug No</span>
            <InputText
              v-model="params.bugNo"
              placeholder="Nhập mã phiếu bug..."
              class="mt-1 w-full"
            />
          </label>
        </div>

        <div class="grid gap-3 lg:grid-cols-2">
          <div>
            <span class="text-xs font-bold text-muted">Ngày tải về</span>
            <div class="mt-1 flex items-center gap-2">
              <DatePicker
                :model-value="parseDateFromParam(params.fromDate)"
                date-format="yy/mm/dd"
                placeholder="Từ ngày"
                show-icon
                class="flex-1"
                @update:model-value="onFromDateChange"
              />
              <span class="text-muted">~</span>
              <DatePicker
                :model-value="parseDateFromParam(params.toDate)"
                date-format="yy/mm/dd"
                placeholder="Đến ngày"
                show-icon
                class="flex-1"
                @update:model-value="onToDateChange"
              />
            </div>
          </div>
          <div class="flex items-end gap-4 pb-1">
            <div class="flex items-center gap-2">
              <Checkbox
                v-model="params.isMovedAtS3"
                :binary="true"
                input-id="chk-moved-s3"
              />
              <label for="chk-moved-s3" class="cursor-pointer text-sm text-secondary">
                Đã di chuyển trên S3
              </label>
            </div>
            <div class="flex items-center gap-2">
              <Checkbox
                v-model="params.isMovedAtLocal"
                :binary="true"
                input-id="chk-moved-local"
              />
              <label for="chk-moved-local" class="cursor-pointer text-sm text-secondary">
                Đã di chuyển nội bộ
              </label>
            </div>
          </div>
        </div>

        <div class="flex items-center justify-end gap-2">
          <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined @click="clearSearch" />
          <Button icon="pi pi-search" label="Search" :loading="isSearching" @click="search" />
        </div>
      </div>
    </Fieldset>

    <!-- Results -->
    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Download history</h3>
        <span class="text-xs text-muted">{{ results.length.toLocaleString("en-US") }} records</span>
      </div>
      <DataTable
        class="app-data-table min-h-0"
        :value="results"
        :empty-message="isSearching ? 'Loading...' : 'No records match the search conditions.'"
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '700px' }"
        paginator
        :rows="20"
        :rows-per-page-options="[20, 50, 100]"
        paginator-template="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        current-page-report-template="Showing {first} to {last} of {totalRecords}"
      >
        <Column header="Ngày tải">
          <template #body="{ data }">
            {{ formatDate(data.downloadYmd) }}
          </template>
        </Column>
        <Column header="Nguồn" field="awsName" />
        <Column header="Thông tin đã tải" field="bugNo" />
        <Column header="Di chuyển nội bộ">
          <template #body="{ data }">
            <span v-if="data.isMovedAtLocal" class="text-green-600">Đã di chuyển</span>
          </template>
        </Column>
      </DataTable>
    </section>
  </section>
</template>
