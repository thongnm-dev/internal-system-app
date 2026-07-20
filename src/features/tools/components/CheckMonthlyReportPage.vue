<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import Calendar from "primevue/calendar";
import Checkbox from "primevue/checkbox";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Button from "primevue/button";
import { useAuthStore } from "@/app/stores/auth";
import { useCheckMonthlyReport } from "../composables/useCheckMonthlyReport";
import { emptyTotals, formatHourValue, totalMinutes } from "@/shared/utils/timeMath";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { readScheduleExcel, type ScheduleResult } from "@/tauri/commands/schedule";
import type {
  AnalysisResult,
  ProjectSummary,
  SelectedPhaseDetail,
} from "@/_/types/analysis";
import type { ImportCsvPreviewResult } from "@/_/types/check-monthly-report";

const auth = useAuthStore();
const loading = useGlobalLoading();
const ctrl = useCheckMonthlyReport();

// Preview is shown inside a dialog with two views: summary + raw detail.
const isPreviewDialogOpen = ref(false);
const previewDialogView = ref<"summary" | "detail">("summary");

// Result of the "Compare" button, rendered in the main Preview section.
type CompareStatus = "match" | "mismatch" | "csv-only" | "schedule-only";
type CompareRow = {
  date: string;
  csvHours: number;
  scheduleHours: number;
  diffHours: number;
  status: CompareStatus;
};
const compareResult = ref<CompareRow[] | null>(null);

function normalizeDate(value: string): string {
  const m = value.match(/(\d{4})\D+(\d{1,2})\D+(\d{1,2})/);
  if (!m) return value.trim();
  const [, y, mo, da] = m;
  return `${y}-${mo.padStart(2, "0")}-${da.padStart(2, "0")}`;
}

const schedulePath = ref("");
const targetMonth = ref<Date>(new Date());
const targetUser = ref(auth.user?.username ?? "");
const autoFetchData = ref(false);
const isConflictDialogOpen = ref(false);
const isScheduleDialogOpen = ref(false);
const scheduleData = ref<ScheduleResult | null>(null);

type ScheduleTableRow = {
  id: string;
  date: string;
  phase: string;
  job_id: string;
  hours: number;
  sheet_name: string;
  worker_name: string;
};

const scheduleRows = computed<ScheduleTableRow[]>(() => {
  if (!scheduleData.value) return [];
  const grouped = new Map<string, ScheduleTableRow>();
  const tm = scheduleData.value.target_month;
  const [mm, yyyy] = tm.split("/");
  for (const worker of scheduleData.value.workers) {
    for (const day of worker.days) {
      for (const entry of day.entries) {
        const date = `${yyyy}/${mm}/${day.day}`;
        const key = `${worker.worker_name}|${date}|${entry.job_id}|${entry.sheet_name}`;
        const existing = grouped.get(key);
        if (existing) {
          existing.hours += entry.hours;
        } else {
          grouped.set(key, {
            id: key,
            date,
            phase: entry.phase,
            job_id: entry.job_id,
            hours: entry.hours,
            sheet_name: entry.sheet_name,
            worker_name: worker.worker_name,
          });
        }
      }
    }
  }
  return Array.from(grouped.values());
});

const scheduleTotalHours = computed(() =>
  scheduleRows.value.reduce((sum, r) => sum + r.hours, 0),
);

const compareTotals = computed(() => {
  const rows = compareResult.value ?? [];
  const csv = rows.reduce((s, r) => s + r.csvHours, 0);
  const schedule = rows.reduce((s, r) => s + r.scheduleHours, 0);
  const mismatches = rows.filter((r) => r.status !== "match").length;
  return { csv, schedule, diff: csv - schedule, mismatches };
});

async function viewScheduleData() {
  if (!schedulePath.value) return;
  await loading.run(async () => {
    try {
      const year = targetMonth.value.getFullYear();
      const month = targetMonth.value.getMonth() + 1;
      scheduleData.value = await readScheduleExcel(schedulePath.value, year, month, targetUser.value || undefined);
      isScheduleDialogOpen.value = true;
    } catch (e) {
      ctrl.message.value = friendlyError(e);
      ctrl.messageMode.value = "error";
    }
  });
}

async function previewWithLoading() {
  await loading.run(() => ctrl.previewCsv());
  if (ctrl.previewResult.value) {
    previewDialogView.value = "summary";
    isPreviewDialogOpen.value = true;
  }
}

function onCompareClick() {
  if (ctrl.csvPath.value && autoFetchData.value) {
    isConflictDialogOpen.value = true;
    return;
  }
  runCompare(autoFetchData.value ? "fetch" : "file");
}

async function runCompare(source: "file" | "fetch") {
  isConflictDialogOpen.value = false;
  await loading.run(async () => {
    if (source === "fetch" && !ctrl.previewResult.value) {
      ctrl.message.value = "Chức năng tự động lấy dữ liệu từ hệ thống chưa được hỗ trợ. Đang so sánh với schedule.";
      ctrl.messageMode.value = "info";
    }

    // CSV recorded hours per date (minutes → summed later).
    const csvByDate = new Map<string, number>();
    for (const row of ctrl.previewResult.value?.preview_rows ?? []) {
      const key = normalizeDate(row.date);
      csvByDate.set(key, (csvByDate.get(key) ?? 0) + row.total_minutes);
    }

    // Schedule planned hours per date.
    const scheduleByDate = new Map<string, number>();
    for (const r of scheduleRows.value) {
      const key = normalizeDate(r.date);
      scheduleByDate.set(key, (scheduleByDate.get(key) ?? 0) + r.hours);
    }

    const allDates = Array.from(new Set([...csvByDate.keys(), ...scheduleByDate.keys()])).sort();
    compareResult.value = allDates.map((date) => {
      const csvHours = (csvByDate.get(date) ?? 0) / 60;
      const scheduleHours = scheduleByDate.get(date) ?? 0;
      const diffHours = csvHours - scheduleHours;
      let status: CompareStatus;
      if (!csvByDate.has(date)) status = "schedule-only";
      else if (!scheduleByDate.has(date)) status = "csv-only";
      else status = Math.abs(diffHours) < 0.01 ? "match" : "mismatch";
      return { date, csvHours, scheduleHours, diffHours, status };
    });
  });
}

async function pickScheduleFile() {
  if (!canUseTauriRuntime()) {
    ctrl.message.value = tauriRuntimeMessage;
    ctrl.messageMode.value = "error";
    return;
  }
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Excel", extensions: ["xlsx", "xls"] }],
    });
    if (typeof selected === "string") {
      schedulePath.value = selected;
    }
  } catch (e) {
    ctrl.message.value = friendlyError(e);
    ctrl.messageMode.value = "error";
  }
}

type ImportProjectTableRow =
  | { id: string; kind: "project"; project: ProjectSummary; phase: null; rowCount: number; totals: ProjectSummary["totals"] }
  | { id: string; kind: "phase"; project: ProjectSummary; phase: ProjectSummary["phases"][number]; rowCount: number; totals: ProjectSummary["totals"] };

function buildImportSummary(result: ImportCsvPreviewResult): AnalysisResult {
  const projects = new Map<string, ProjectSummary>();
  for (const row of result.preview_rows) {
    const project = projects.get(row.project_code) ?? {
      project_code: row.project_code,
      project_name: row.project_name,
      totals: emptyTotals(),
      row_count: 0,
      phases: [],
    } satisfies ProjectSummary;

    let phase = project.phases.find((p) => p.process_code === row.process_code);
    if (!phase) {
      phase = { process_code: row.process_code, phase_name: row.phase_name, totals: emptyTotals(), row_count: 0, details: [] };
      project.phases.push(phase);
    }
    project.row_count += 1;
    project.totals.regular_minutes += row.total_minutes;
    phase.row_count += 1;
    phase.totals.regular_minutes += row.total_minutes;
    phase.details.push({ date: row.date, work_content: row.work_content, total_minutes: row.total_minutes });
    projects.set(row.project_code, project);
  }
  const sorted = Array.from(projects.values()).sort((a, b) => totalMinutes(b.totals) - totalMinutes(a.totals));
  for (const p of sorted) p.phases.sort((a, b) => totalMinutes(b.totals) - totalMinutes(a.totals));
  return { source_path: result.source_path, row_count: result.row_count, grand_total: { regular_minutes: result.total_minutes, normal_overtime_minutes: 0, legal_holiday_overtime_minutes: 0, legal_public_holiday_overtime_minutes: 0, late_night_overtime_minutes: 0 }, projects: sorted };
}

function buildImportProjectRows(project: ProjectSummary): ImportProjectTableRow[] {
  return [
    { id: `${project.project_code}-all`, kind: "project", project, phase: null, rowCount: project.row_count, totals: project.totals },
    ...project.phases.map((phase) => ({ id: `${project.project_code}-${phase.process_code}`, kind: "phase" as const, project, phase, rowCount: phase.row_count, totals: phase.totals })),
  ];
}

const previewRows = computed(() => {
  if (!ctrl.previewResult.value) return [];
  const summary = buildImportSummary(ctrl.previewResult.value);
  return summary.projects.flatMap(buildImportProjectRows);
});

const hiddenCsvDetailHeaders = new Set(["社員番号", "給与社員番号", "プロジェクトコード（日本）", "固定プロジェクト名", "プロセス"]);

function formatCsvMinuteValue(value: string | undefined): string {
  const minutes = Number((value ?? "").trim().replace(/,/g, ""));
  if (!Number.isFinite(minutes) || minutes === 0) return "-";
  return formatHourValue(minutes);
}

function onOpenDetail(detail: SelectedPhaseDetail) {
  // placeholder for phase detail
  void detail;
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <div class="grid grid-cols-1 gap-4">
      <div class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
        <div class="mt-4 grid grid-cols-[1fr_auto] gap-2">
          <InputText class="min-w-0 flex-1" placeholder="Select CSV file..."
          :model-value="ctrl.csvPath.value"
          @update:model-value="ctrl.updateCsvPath($event as string)"
          readonly
          />
          <Button icon="pi pi-folder-open" severity="secondary" outlined title="Browse CSV" @click="ctrl.pickCsvFile()" />
        </div>
        <div class="mt-4 grid grid-cols-[1fr_auto] gap-2">
          <InputText class="min-w-0 flex-1" placeholder="Select schedule Excel file..."
          :model-value="schedulePath"
          readonly
          />
          <Button icon="pi pi-folder-open" severity="secondary" outlined title="Browse Excel" @click="pickScheduleFile()" />
        </div>
        <div class="mt-2 flex items-center gap-2">
          <span class="w-24 shrink-0 text-sm font-medium text-muted">Target month</span>
          <Calendar v-model="targetMonth" view="month" date-format="yy/mm" show-icon icon-display="input" class="w-44" />
        </div>
        <div class="mt-2 flex items-center gap-2">
          <span class="w-24 shrink-0 text-sm font-medium text-muted">Target user</span>
          <InputText v-model="targetUser" placeholder="Worker name" class="w-44" />
        </div>
        <div class="mt-2 flex items-center gap-2">
          <span class="w-24 shrink-0 text-sm font-medium text-muted"></span>
          <Checkbox v-model="autoFetchData" input-id="autoFetch" :binary="true" />
          <label for="autoFetch" class="cursor-pointer text-sm text-ink">Tự động lấy dữ liệu từ hệ thống</label>
        </div>
        <div class="mt-2 flex items-center justify-end gap-2">
          <Button icon="pi pi-eye" severity="info" outlined label="View schedule data" :disabled="!schedulePath" @click="viewScheduleData()" />
          <Button icon="pi pi-file-import" label="Preview" :disabled="!ctrl.csvPath.value || ctrl.isImporting.value" @click="previewWithLoading()" />
          <Button icon="pi pi-check-circle" label="Compare" :disabled="!ctrl.csvPath.value || !schedulePath" @click="onCompareClick()" />
        </div>
      </div>
    </div>

    <div class="grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-hidden">
      <!-- Compare result -->
      <section class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
        <div class="flex items-start justify-between gap-3 border-b border-divider px-4 py-3">
          <div class="min-w-0">
            <h3 class="font-bold">Preview</h3>
            <p class="mt-1 truncate text-xs text-muted">
              {{ compareResult
                ? `CSV ${compareTotals.csv.toFixed(1)}h vs Schedule ${compareTotals.schedule.toFixed(1)}h — ${compareTotals.mismatches} ngày lệch`
                : 'Chưa có so sánh. Chọn dữ liệu rồi nhấn Compare.' }}
            </p>
          </div>
        </div>
        <DataTable v-if="compareResult" class="app-data-table min-h-0" empty-message="No overlapping dates to compare." :row-class="(row: any) => row.status === 'mismatch' ? 'bg-rose-50' : (row.status === 'match' ? '' : 'bg-amber-50')" scrollable scroll-height="flex" :table-style="{ minWidth: '720px' }" :value="compareResult">
          <Column header="Date" field="date" style="width: 130px" />
          <Column header="CSV (hour)" body-class="num" header-class="num" style="width: 120px">
            <template #body="{ data }">{{ data.csvHours.toFixed(1) }}</template>
          </Column>
          <Column header="Schedule (hour)" body-class="num" header-class="num" style="width: 140px">
            <template #body="{ data }">{{ data.scheduleHours.toFixed(1) }}</template>
          </Column>
          <Column header="Diff" body-class="num font-bold" header-class="num" style="width: 100px">
            <template #body="{ data }">
              <span :class="Math.abs(data.diffHours) < 0.01 ? 'text-muted' : 'text-rose-600'">{{ data.diffHours > 0 ? '+' : '' }}{{ data.diffHours.toFixed(1) }}</span>
            </template>
          </Column>
          <Column header="Status" style="width: 130px">
            <template #body="{ data }">
              <span v-if="data.status === 'match'" class="inline-block rounded bg-emerald-100 px-2 py-0.5 text-xs font-bold text-emerald-800">Match</span>
              <span v-else-if="data.status === 'mismatch'" class="inline-block rounded bg-rose-100 px-2 py-0.5 text-xs font-bold text-rose-800">Mismatch</span>
              <span v-else-if="data.status === 'csv-only'" class="inline-block rounded bg-amber-100 px-2 py-0.5 text-xs font-bold text-amber-800">CSV only</span>
              <span v-else class="inline-block rounded bg-amber-100 px-2 py-0.5 text-xs font-bold text-amber-800">Schedule only</span>
            </template>
          </Column>
        </DataTable>
        <div v-else class="flex flex-1 items-center justify-center p-8 text-center text-sm text-muted">
          Nhấn <strong class="mx-1">Compare</strong> để so sánh giờ trong CSV với schedule theo từng ngày.
        </div>
      </section>
    </div>

    <!-- Schedule Data Dialog -->
    <Dialog :visible="isScheduleDialogOpen" modal class="w-full max-w-6xl overflow-hidden rounded-lg bg-panel shadow-2xl" :style="{ maxHeight: '86vh' }" @update:visible="isScheduleDialogOpen = $event">
      <template #header>
        <div class="min-w-0">
          <h3 class="truncate text-lg font-bold text-ink">Schedule data</h3>
          <p class="mt-1 truncate text-sm text-muted">
            {{ scheduleData ? `${scheduleData.target_month} — ${scheduleRows.length} rows — Total: ${scheduleTotalHours.toFixed(1)}h` : '' }}
          </p>
        </div>
      </template>
      <DataTable class="app-data-table min-h-0" empty-message="No schedule data found for this month." scrollable scroll-height="flex" :table-style="{ minWidth: '920px' }" :value="scheduleRows">
        <Column header="Date" field="date" style="width: 100px" />
        <Column header="Worker" field="worker_name" style="width: 130px" />
        <Column header="Phase" field="phase" style="width: 180px" />
        <Column header="Job ID" field="job_id" />
        <Column header="Hours" field="hours" body-class="num font-bold text-brand" header-class="num" style="width: 80px">
          <template #body="{ data }">{{ data.hours.toFixed(1) }}</template>
        </Column>
        <Column header="Sheet" field="sheet_name" style="width: 140px" />
      </DataTable>
    </Dialog>

    <!-- Conflict Dialog -->
    <Dialog :visible="isConflictDialogOpen" modal header="Xác nhận nguồn dữ liệu" class="w-full max-w-lg" @update:visible="isConflictDialogOpen = $event">
      <p class="text-sm text-ink">
        Bạn đã nhập file CSV và đồng thời bật <strong>Tự động lấy dữ liệu từ hệ thống</strong>. Vui lòng chọn nguồn dữ liệu để so khớp:
      </p>
      <template #footer>
        <div class="flex justify-end gap-2">
          <Button label="Cancel" severity="secondary" outlined @click="isConflictDialogOpen = false" />
          <Button label="Sử dụng file" icon="pi pi-file" severity="info" @click="runCompare('file')" />
          <Button label="Lấy dữ liệu mới" icon="pi pi-download" @click="runCompare('fetch')" />
        </div>
      </template>
    </Dialog>

    <!-- Preview Dialog (summary + detail toggle) -->
    <Dialog v-if="ctrl.previewResult.value" :visible="isPreviewDialogOpen" modal class="w-full max-w-6xl overflow-hidden rounded-lg bg-panel shadow-2xl" :style="{ maxHeight: '86vh' }" @update:visible="isPreviewDialogOpen = $event">
      <template #header>
        <div class="flex w-full items-center justify-between gap-3 pr-8">
          <div class="min-w-0">
            <h3 class="truncate text-lg font-bold text-ink">{{ previewDialogView === 'summary' ? 'Preview' : 'CSV detail' }}</h3>
            <p class="mt-1 truncate text-sm text-muted">{{ ctrl.previewResult.value.source_file_name }}{{ ctrl.result.value ? ` - saved batch #${ctrl.result.value.batch_id}` : '' }}</p>
          </div>
          <Button
            :icon="previewDialogView === 'summary' ? 'pi pi-list' : 'pi pi-chart-bar'"
            :label="previewDialogView === 'summary' ? 'Show detail' : 'Show summary'"
            severity="secondary" outlined size="small"
            @click="previewDialogView = previewDialogView === 'summary' ? 'detail' : 'summary'" />
        </div>
      </template>

      <!-- Summary view -->
      <DataTable v-if="previewDialogView === 'summary'" class="app-data-table min-h-0" empty-message="No data." :row-class="(row: any) => row.kind === 'project' ? 'bg-emerald-50 font-bold' : 'cursor-pointer'" scrollable scroll-height="flex" :table-style="{ minWidth: '920px' }" :value="previewRows" @row-click="(e: any) => { if (e.data.kind === 'phase') onOpenDetail({ project_code: e.data.project.project_code, project_name: e.data.project.project_name, phase: e.data.phase }); }">
        <Column header="Project">
          <template #body="{ data }"><strong v-if="data.kind === 'project'">{{ `${data.project.project_code} ${data.project.project_name}`.trim() }}</strong></template>
        </Column>
        <Column header="Phase">
          <template #body="{ data }">
            <template v-if="data.kind === 'project'">All phases</template>
            <template v-else><span class="mr-2 inline-block min-w-8 rounded bg-blue-100 px-1.5 py-0.5 text-center text-xs font-extrabold text-blue-800">{{ data.phase.process_code }}</span>{{ data.phase.phase_name }}</template>
          </template>
        </Column>
        <Column header="Rows" body-class="num" header-class="num"><template #body="{ data }">{{ data.rowCount.toLocaleString("en-US") }}</template></Column>
        <Column header="Total (hour)" body-class="num font-bold text-brand" header-class="num"><template #body="{ data }">{{ formatHourValue(totalMinutes(data.totals)) }}</template></Column>
      </DataTable>

      <!-- Detail view -->
      <DataTable v-else class="app-data-table min-h-0" empty-message="No CSV rows." scrollable scroll-height="flex" :table-style="{ minWidth: `${Math.max(920, (ctrl.previewResult.value?.raw_headers.filter(h => !hiddenCsvDetailHeaders.has(h)).length ?? 0) * 140)}px` }" :value="ctrl.previewResult.value.raw_rows.map((cells, i) => ({ cells, id: i }))">
        <Column v-for="col in ctrl.previewResult.value.raw_headers.map((h, i) => ({ header: h, index: i })).filter(c => !hiddenCsvDetailHeaders.has(c.header))" :key="`${col.header}-${col.index}`" :header="col.header" :body-class="ctrl.previewResult.value.minute_column_indexes.includes(col.index) ? 'num' : undefined" :header-class="ctrl.previewResult.value.minute_column_indexes.includes(col.index) ? 'num' : undefined">
          <template #body="{ data }">{{ ctrl.previewResult.value!.minute_column_indexes.includes(col.index) ? formatCsvMinuteValue(data.cells[col.index]) : (data.cells[col.index] || '') }}</template>
        </Column>
      </DataTable>
    </Dialog>
  </section>
</template>
