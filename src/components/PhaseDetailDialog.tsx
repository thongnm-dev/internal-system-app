import { X } from "lucide-react";
import { formatHourValue } from "../core/timeMath";
import type { SelectedPhaseDetail } from "../types/statistics";

type PhaseDetailDialogProps = {
  detail: SelectedPhaseDetail | null;
  onClose: () => void;
};

export function PhaseDetailDialog({ detail, onClose }: PhaseDetailDialogProps) {
  if (!detail) {
    return null;
  }

  const title = `${detail.project_code} - ${detail.phase.process_code} ${detail.phase.phase_name}`;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 p-6">
      <section className="flex max-h-[82vh] w-full max-w-3xl flex-col overflow-hidden rounded-lg bg-white shadow-2xl">
        <header className="flex items-start justify-between gap-4 border-b border-stone-200 px-5 py-4">
          <div className="min-w-0">
            <h3 className="truncate text-lg font-bold text-slate-900">{title}</h3>
            <p className="mt-1 truncate text-sm text-slate-500">{detail.project_name || "No project name"}</p>
          </div>
          <button
            className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md border border-slate-200 text-slate-600 hover:bg-slate-50"
            type="button"
            title="Close"
            onClick={onClose}
          >
            <X className="h-4 w-4" />
          </button>
        </header>

        <div className="min-h-0 overflow-auto">
          <table className="w-full min-w-[680px] border-collapse">
            <thead>
              <tr>
                <th className="table-head">日付</th>
                <th className="table-head">作業内容</th>
                <th className="table-head num">Total (hour)</th>
              </tr>
            </thead>
            <tbody>
              {detail.phase.details.length === 0 ? (
                <tr>
                  <td className="table-cell h-28 text-center text-slate-500" colSpan={3}>
                    No detail rows.
                  </td>
                </tr>
              ) : (
                detail.phase.details.map((row, index) => (
                  <tr key={`${row.date}-${index}`}>
                    <td className="table-cell whitespace-nowrap">{row.date}</td>
                    <td className="table-cell">{row.work_content || "-"}</td>
                    <td className="table-cell num font-bold text-brand">{formatHourValue(row.total_minutes)}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
