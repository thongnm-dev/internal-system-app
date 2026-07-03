<script setup lang="ts">
import { useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import { useImportReports } from "../composables/useImportReports";
import { formatHourValue } from "@/shared/utils/timeMath";
import type { ImportReportListItem, ImportReportSearchCriteria } from "@/shared/types/import-report";

const router = useRouter();
const ctrl = useImportReports();

function setField(field: keyof ImportReportSearchCriteria, value: string) {
  ctrl.criteria.value = { ...ctrl.criteria.value, [field]: value };
}

function formatTargetMonthRange(item: ImportReportListItem) {
  if (!item.target_month_from && !item.target_month_to) return "No target month";
  if (item.target_month_from === item.target_month_to || !item.target_month_to) return item.target_month_from;
  if (!item.target_month_from) return item.target_month_to;
  return `${item.target_month_from} - ${item.target_month_to}`;
}

function parseMonth(value: string): string {
  if (!value) return "";
  return value.slice(0, 7);
}

function openReport(id: number) {
  router.push(`/import-reports/${id}`);
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="space-y-3">
        <div class="grid grid-cols-2 gap-3">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Ngay doi tuong tu</span>
            <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm outline-none hover:border-brand focus:border-brand focus:ring-2 focus:ring-emerald-100" type="month" :value="parseMonth(ctrl.criteria.value.target_month_from)" @change="setField('target_month_from', ($event.target as HTMLInputElement).value)" />
          </label>
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Ngay doi tuong den</span>
            <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm outline-none hover:border-brand focus:border-brand focus:ring-2 focus:ring-emerald-100" type="month" :value="parseMonth(ctrl.criteria.value.target_month_to)" @change="setField('target_month_to', ($event.target as HTMLInputElement).value)" />
          </label>
        </div>

        <label class="block min-w-0">
          <span class="text-xs font-bold text-muted">Ten</span>
          <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Ten bao cao" type="text" :value="ctrl.criteria.value.report_name" @input="setField('report_name', ($event.target as HTMLInputElement).value)" />
        </label>

        <label class="block min-w-0">
          <span class="text-xs font-bold text-muted">Keyword</span>
          <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Ten, ghi chu, file, nguoi import" type="text" :value="ctrl.criteria.value.keyword" @input="setField('keyword', ($event.target as HTMLInputElement).value)" @keydown.enter="ctrl.search()" />
        </label>

        <div class="flex items-center justify-end gap-2">
          <button class="flex h-10 items-center justify-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:bg-emerald-800 disabled:cursor-not-allowed disabled:opacity-60" type="button" :disabled="ctrl.isSearching.value" @click="ctrl.search()"><i class="pi pi-search" />Search</button>
          <button class="flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-60" type="button" :disabled="ctrl.isSearching.value" @click="ctrl.reset()"><i class="pi pi-refresh" />Reset</button>
        </div>
      </div>
    </section>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="border-b border-divider px-4 py-3">
        <h3 class="font-bold">Danh sach import</h3>
        <p class="mt-1 text-xs text-muted">Du lieu da duoc luu tu man hinh Import CSV.</p>
      </div>
      <DataTable class="app-data-table min-h-0" empty-message="No imported reports found." :row-class="() => 'cursor-pointer'" scrollable scroll-height="flex" :table-style="{ minWidth: '1080px' }" :value="ctrl.items.value" @row-click="(e: any) => openReport(e.data.id)">
        <Column header="SEQ" body-class="num font-bold text-secondary" header-class="num"><template #body="{ data }">{{ `#${data.id}` }}</template></Column>
        <Column header="Ten"><template #body="{ data }"><div class="min-w-0"><strong class="block truncate text-ink">{{ data.report_name || '-' }}</strong><span class="mt-1 block text-xs text-muted">{{ formatTargetMonthRange(data) }}</span></div></template></Column>
        <Column header="Ghi chu" body-class="max-w-[260px] truncate"><template #body="{ data }">{{ data.note || '-' }}</template></Column>
        <Column field="source_file_name" header="Ten file da import" body-class="max-w-[280px] truncate" />
        <Column field="imported_at" header="Ngay gio" body-class="whitespace-nowrap" />
        <Column header="Nguoi import" body-class="whitespace-nowrap"><template #body="{ data }">{{ data.imported_by || '-' }}</template></Column>
        <Column header="Rows" body-class="num" header-class="num"><template #body="{ data }">{{ data.row_count.toLocaleString("en-US") }}</template></Column>
        <Column header="Hours" body-class="num" header-class="num"><template #body="{ data }">{{ formatHourValue(data.total_minutes) }}</template></Column>
      </DataTable>
    </section>
  </section>
</template>
