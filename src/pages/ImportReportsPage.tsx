import { Calendar, RotateCcw, Search } from "lucide-react";
import { MessageBanner } from "../components/MessageBanner";
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
  message,
  messageMode,
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
            <input
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Tên báo cáo"
              type="text"
              value={criteria.report_name}
              onChange={(event) => setField("report_name", event.target.value)}
            />
          </label>

          <label className="block min-w-0">
            <span className="text-xs font-bold text-slate-500">Keyword</span>
            <input
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
            <button
              className="flex h-10 items-center justify-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:bg-emerald-800 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              disabled={isSearching}
              onClick={onSearch}
            >
              <Search className="h-4 w-4" />
              Search
            </button>

            <button
              className="flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              title="Reset search conditions"
              disabled={isSearching}
              onClick={onReset}
            >
              <RotateCcw className="h-4 w-4" />
              Reset
            </button>
          </div>
        </div>
      </section>

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
        <div className="border-b border-stone-200 px-4 py-3">
          <h3 className="font-bold">Danh sách import</h3>
          <p className="mt-1 text-xs text-slate-500">Dữ liệu đã được lưu từ màn hình Import CSV.</p>
        </div>

        <div className="min-h-0 overflow-auto">
          <table className="w-full min-w-[1080px] border-collapse">
            <thead>
              <tr>
                <th className="table-head num">SEQ</th>
                <th className="table-head">Tên</th>
                <th className="table-head">Ghi chú</th>
                <th className="table-head">Tên file đã import</th>
                <th className="table-head">Ngày giờ</th>
                <th className="table-head">Người import</th>
                <th className="table-head num">Rows</th>
                <th className="table-head num">Hours</th>
              </tr>
            </thead>
            <tbody>
              {items.length === 0 ? (
                <tr>
                  <td className="table-cell h-48 text-center text-slate-500" colSpan={8}>
                    No imported reports found.
                  </td>
                </tr>
              ) : (
                items.map((item) => <ImportReportRow key={item.id} item={item} onOpenReport={onOpenReport} />)
              )}
            </tbody>
          </table>
        </div>
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
      <span className="relative mt-1 block">
        <input
          className="h-10 w-full rounded-md border border-slate-300 bg-white px-3 pr-10 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
          type="month"
          value={value}
          onChange={(event) => onChange(event.target.value)}
        />
        <Calendar className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
      </span>
    </label>
  );
}

function ImportReportRow({
  item,
  onOpenReport,
}: {
  item: ImportReportListItem;
  onOpenReport: (reportId: number) => void;
}) {
  return (
    <tr
      className="cursor-pointer hover:bg-slate-50"
      title={`Open report #${item.id}`}
      onClick={() => onOpenReport(item.id)}
    >
      <td className="table-cell num font-bold text-slate-700">#{item.id}</td>
      <td className="table-cell">
        <div className="min-w-0">
          <strong className="block truncate text-slate-900">{item.report_name || "-"}</strong>
          <span className="mt-1 block text-xs text-slate-500">{formatTargetMonthRange(item)}</span>
        </div>
      </td>
      <td className="table-cell max-w-[260px] truncate">{item.note || "-"}</td>
      <td className="table-cell max-w-[280px] truncate">{item.source_file_name}</td>
      <td className="table-cell whitespace-nowrap">{item.imported_at}</td>
      <td className="table-cell whitespace-nowrap">{item.imported_by || "-"}</td>
      <td className="table-cell num">{item.row_count.toLocaleString("en-US")}</td>
      <td className="table-cell num">{formatHourValue(item.total_minutes)}</td>
    </tr>
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
