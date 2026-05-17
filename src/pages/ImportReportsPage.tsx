import { Calendar, RotateCcw, Search } from "lucide-react";
import { Button } from "primereact/button";
import { Calendar as PrimeCalendar } from "primereact/calendar";
import { Column } from "primereact/column";
import { DataTable } from "primereact/datatable";
import { InputText } from "primereact/inputtext";
import { formatHourValue } from "../core/timeMath";
import type { ImportReportListItem, ImportReportSearchCriteria, MessageMode } from "../types/statistics";

type ImportReportsPageProps = {
  criteria: ImportReportSearchCriteria;
  isSearching: boolean;
  items: ImportReportListItem[];
  message: string;
  messageMode: MessageMode;
  onReset: () => void;
  onSearch: () => void;
  onSetCriteria: (criteria: ImportReportSearchCriteria) => void;
  onOpenReport: (reportId: number) => void;
};

export function ImportReportsPage({
  criteria,
  isSearching,
  items,
  onReset,
  onSearch,
  onSetCriteria,
  onOpenReport,
}: ImportReportsPageProps) {
  const setField = (field: keyof ImportReportSearchCriteria, value: string) => {
    onSetCriteria({ ...criteria, [field]: value });
  };

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <section className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
        <div className="space-y-3">
          <div className="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] gap-3">
            <MonthPicker
              label="Ngày đối tượng từ"
              value={criteria.target_month_from}
              onChange={(value) => setField("target_month_from", value)}
            />
            <MonthPicker
              label="Ngày đối tượng đến"
              value={criteria.target_month_to}
              onChange={(value) => setField("target_month_to", value)}
            />
          </div>

          <label className="block min-w-0">
            <span className="text-xs font-bold text-slate-500">Tên</span>
            <InputText
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Tên báo cáo"
              type="text"
              value={criteria.report_name}
              onChange={(event) => setField("report_name", event.target.value)}
            />
          </label>

          <label className="block min-w-0">
            <span className="text-xs font-bold text-slate-500">Keyword</span>
            <InputText
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Tên, ghi chú, file, người import"
              type="text"
              value={criteria.keyword}
              onChange={(event) => setField("keyword", event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  onSearch();
                }
              }}
            />
          </label>

          <div className="flex items-center justify-end gap-2">
            <Button
              className="flex h-10 items-center justify-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:bg-emerald-800 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              disabled={isSearching}
              onClick={onSearch}
            >
              <Search className="h-4 w-4" />
              Search
            </Button>

            <Button
              className="flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              title="Reset search conditions"
              disabled={isSearching}
              onClick={onReset}
            >
              <RotateCcw className="h-4 w-4" />
              Reset
            </Button>
          </div>
        </div>
      </section>

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
        <div className="border-b border-stone-200 px-4 py-3">
          <h3 className="font-bold">Danh sách import</h3>
          <p className="mt-1 text-xs text-slate-500">Dữ liệu đã được lưu từ màn hình Import CSV.</p>
        </div>

        <DataTable
          className="app-data-table min-h-0"
          emptyMessage="No imported reports found."
          rowClassName={() => "cursor-pointer"}
          scrollable
          scrollHeight="flex"
          tableStyle={{ minWidth: "1080px" }}
          value={items}
          onRowClick={(event) => onOpenReport(event.data.id)}
        >
          <Column header="SEQ" body={(item: ImportReportListItem) => `#${item.id}`} bodyClassName="num font-bold text-slate-700" headerClassName="num" />
          <Column header="Tên" body={reportNameBody} />
          <Column header="Ghi chú" body={(item: ImportReportListItem) => item.note || "-"} bodyClassName="max-w-[260px] truncate" />
          <Column field="source_file_name" header="Tên file đã import" bodyClassName="max-w-[280px] truncate" />
          <Column field="imported_at" header="Ngày giờ" bodyClassName="whitespace-nowrap" />
          <Column header="Người import" body={(item: ImportReportListItem) => item.imported_by || "-"} bodyClassName="whitespace-nowrap" />
          <Column header="Rows" body={(item: ImportReportListItem) => item.row_count.toLocaleString("en-US")} bodyClassName="num" headerClassName="num" />
          <Column header="Hours" body={(item: ImportReportListItem) => formatHourValue(item.total_minutes)} bodyClassName="num" headerClassName="num" />
        </DataTable>
      </section>
    </section>
  );
}

function MonthPicker({
  label,
  onChange,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  value: string;
}) {
  return (
    <label className="block min-w-0">
      <span className="text-xs font-bold text-slate-500">{label}</span>
      <PrimeCalendar
        className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white text-sm outline-none focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100"
        dateFormat="yy-mm"
        icon={<Calendar className="h-4 w-4 text-slate-400" />}
        placeholder="yyyy-mm"
        showIcon
        value={parseMonth(value)}
        view="month"
        onChange={(event) => onChange(formatMonth(event.value instanceof Date ? event.value : null))}
      />
    </label>
  );
}

function parseMonth(value: string) {
  const [year, month] = value.split("-").map(Number);
  if (!year || !month) {
    return null;
  }

  return new Date(year, month - 1, 1);
}

function formatMonth(value: Date | null) {
  if (!value) {
    return "";
  }

  return `${value.getFullYear()}-${String(value.getMonth() + 1).padStart(2, "0")}`;
}

function reportNameBody(item: ImportReportListItem) {
  return (
    <div className="min-w-0">
      <strong className="block truncate text-slate-900">{item.report_name || "-"}</strong>
      <span className="mt-1 block text-xs text-slate-500">{formatTargetMonthRange(item)}</span>
    </div>
  );
}

function formatTargetMonthRange(item: ImportReportListItem) {
  if (!item.target_month_from && !item.target_month_to) {
    return "No target month";
  }

  if (item.target_month_from === item.target_month_to || !item.target_month_to) {
    return item.target_month_from;
  }

  if (!item.target_month_from) {
    return item.target_month_to;
  }

  return `${item.target_month_from} - ${item.target_month_to}`;
}
