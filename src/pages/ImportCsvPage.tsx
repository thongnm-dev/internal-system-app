import { Database, Eye, FileInput, FolderOpen, Save } from "lucide-react";
import { useMemo } from "react";
import { MessageBanner } from "../components/MessageBanner";
import { formatHourValue } from "../core/timeMath";
import type {
  ImportBatchSummary,
  ImportCsvPreviewResult,
  ImportCsvResult,
  ImportPreviewRow,
  MessageMode,
  SelectedPhaseDetail,
} from "../types/statistics";

type ImportCsvPageProps = {
  batches: ImportBatchSummary[];
  csvPath: string;
  importPreviewResult: ImportCsvPreviewResult | null;
  importResult: ImportCsvResult | null;
  isImporting: boolean;
  isSavingImport: boolean;
  message: string;
  messageMode: MessageMode;
  onCsvPathChange: (value: string) => void;
  onImport: () => void;
  onOpenDetail: (detail: SelectedPhaseDetail) => void;
  onPickCsvFile: () => void;
  onSave: () => void;
};

export function ImportCsvPage({
  batches,
  csvPath,
  importPreviewResult,
  importResult,
  isImporting,
  isSavingImport,
  message,
  messageMode,
  onCsvPathChange,
  onImport,
  onOpenDetail,
  onPickCsvFile,
  onSave,
}: ImportCsvPageProps) {
  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <div className="grid grid-cols-[minmax(0,1fr)_320px] gap-4">
        <div className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
          <div className="flex items-center gap-2">
            <Database className="h-5 w-5 text-brand" />
            <h3 className="font-bold">Monthly report CSV import</h3>
          </div>
          <p className="mt-2 text-sm text-slate-600">
            Upload an exported CSV from the system and save it as check data for monthly report matching.
          </p>

          <div className="mt-4 grid grid-cols-[minmax(0,1fr)_42px_112px_96px] gap-2">
            <input
              className="h-10 min-w-0 rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Select CSV file..."
              type="text"
              value={csvPath}
              onChange={(event) => onCsvPathChange(event.target.value)}
            />
            <button
              className="flex h-10 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700 hover:bg-slate-50"
              type="button"
              title="Browse CSV"
              onClick={onPickCsvFile}
            >
              <FolderOpen className="h-4 w-4" />
            </button>
            <button
              className="flex h-10 items-center justify-center gap-2 rounded-md bg-slate-800 px-3 text-sm font-bold text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              disabled={isImporting}
              onClick={onImport}
            >
              <FileInput className="h-4 w-4" />
              Import
            </button>
            <button
              className="flex h-10 items-center justify-center gap-2 rounded-md bg-brand px-3 text-sm font-bold text-white hover:bg-emerald-800 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              disabled={isSavingImport || !importPreviewResult}
              onClick={onSave}
            >
              <Save className="h-4 w-4" />
              Save
            </button>
          </div>
        </div>

        <div className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
          <span className="text-sm font-bold text-slate-500">Current import</span>
          <strong className="mt-2 block text-2xl leading-tight text-slate-900">
            {importResult ? importResult.row_count.toLocaleString("en-US") : "-"}
          </strong>
          <p className="mt-1 text-sm text-slate-500">saved rows</p>
        </div>
      </div>

      <MessageBanner message={message} mode={messageMode} />

      <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_320px] gap-4 overflow-hidden">
        <ImportPreview result={importPreviewResult} saveResult={importResult} onOpenDetail={onOpenDetail} />
        <ImportHistory batches={batches} />
      </div>
    </section>
  );
}

function ImportPreview({
  onOpenDetail,
  result,
  saveResult,
}: {
  onOpenDetail: (detail: SelectedPhaseDetail) => void;
  result: ImportCsvPreviewResult | null;
  saveResult: ImportCsvResult | null;
}) {
  const groups = useMemo(() => (result ? buildPreviewGroups(result.preview_rows) : []), [result]);

  return (
    <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
      <div className="border-b border-stone-200 px-4 py-3">
        <h3 className="font-bold">Preview</h3>
        <p className="mt-1 truncate text-xs text-slate-500">
          {result
            ? `${result.source_file_name}${saveResult ? ` - saved batch #${saveResult.batch_id}` : ""}`
            : "No CSV data imported yet."}
        </p>
      </div>
      <div className="min-h-0 overflow-auto">
        <table className="w-full min-w-[920px] border-collapse">
          <thead>
            <tr>
              <th className="table-head">Project</th>
              <th className="table-head">Phase</th>
              <th className="table-head num">Rows</th>
              <th className="table-head num">Total (hour)</th>
              <th className="table-head num">Detail</th>
            </tr>
          </thead>
          <tbody>
            {!result ? (
              <tr>
                <td className="table-cell h-48 text-center text-slate-500" colSpan={5}>
                  No data. Select a CSV file, then click Import.
                </td>
              </tr>
            ) : (
              groups.map((group) => (
                <tr
                  key={`${group.projectCode}-${group.processCode}`}
                  className="cursor-pointer hover:bg-slate-50"
                  title="Open phase details"
                  onClick={() => onOpenDetail(group.detail)}
                >
                  <td className="table-cell">
                    <div className="font-bold">{group.projectCode}</div>
                    <div className="text-xs text-slate-500">{group.projectName || "-"}</div>
                  </td>
                  <td className="table-cell">
                    <span className="mr-2 inline-block min-w-8 rounded bg-blue-100 px-1.5 py-0.5 text-center text-xs font-extrabold text-blue-800">
                      {group.processCode}
                    </span>
                    {group.phaseName}
                  </td>
                  <td className="table-cell num">{group.rowCount.toLocaleString("en-US")}</td>
                  <td className="table-cell num font-bold text-brand">{formatHourValue(group.totalMinutes)}</td>
                  <td className="table-cell num">
                    <button
                      className="inline-flex h-8 w-8 items-center justify-center rounded-md border border-slate-200 text-slate-600 hover:bg-slate-50"
                      type="button"
                      title="Open detail"
                      onClick={(event) => {
                        event.stopPropagation();
                        onOpenDetail(group.detail);
                      }}
                    >
                      <Eye className="h-4 w-4" />
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}

type PreviewGroup = {
  detail: SelectedPhaseDetail;
  phaseName: string;
  processCode: string;
  projectCode: string;
  projectName: string;
  rowCount: number;
  totalMinutes: number;
};

function buildPreviewGroups(rows: ImportPreviewRow[]): PreviewGroup[] {
  const groups = new Map<string, PreviewGroup>();

  for (const row of rows) {
    const key = `${row.project_code}\u0000${row.process_code}`;
    const current =
      groups.get(key) ??
      ({
        detail: {
          project_code: row.project_code,
          project_name: row.project_name,
          phase: {
            process_code: row.process_code,
            phase_name: row.phase_name,
            row_count: 0,
            totals: {
              regular_minutes: 0,
              normal_overtime_minutes: 0,
              legal_holiday_overtime_minutes: 0,
              legal_public_holiday_overtime_minutes: 0,
              late_night_overtime_minutes: 0,
            },
            details: [],
          },
        },
        phaseName: row.phase_name,
        processCode: row.process_code,
        projectCode: row.project_code,
        projectName: row.project_name,
        rowCount: 0,
        totalMinutes: 0,
      } satisfies PreviewGroup);

    current.rowCount += 1;
    current.totalMinutes += row.total_minutes;
    current.detail.phase.row_count = current.rowCount;
    current.detail.phase.totals.regular_minutes = current.totalMinutes;
    current.detail.phase.details.push({
      date: row.date,
      work_content: row.work_content,
      total_minutes: row.total_minutes,
    });
    groups.set(key, current);
  }

  return Array.from(groups.values()).sort((a, b) => b.totalMinutes - a.totalMinutes);
}

function ImportHistory({ batches }: { batches: ImportBatchSummary[] }) {
  return (
    <aside className="min-h-0 overflow-auto rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
      <h3 className="font-bold">Recent imports</h3>
      <div className="mt-4 space-y-3">
        {batches.length === 0 ? (
          <p className="text-sm text-slate-500">No import history.</p>
        ) : (
          batches.map((batch) => (
            <div key={batch.id} className="border-b border-stone-100 pb-3 last:border-b-0">
              <div className="flex items-center justify-between gap-3">
                <span className="min-w-0 truncate text-sm font-bold">{batch.source_file_name}</span>
                <span className="text-xs font-bold text-brand">#{batch.id}</span>
              </div>
              <p className="mt-1 text-xs text-slate-500">{batch.imported_at}</p>
              <p className="mt-1 text-xs text-slate-500">
                {batch.row_count.toLocaleString("en-US")} rows / {formatHourValue(batch.total_minutes)} hours
              </p>
            </div>
          ))
        )}
      </div>
    </aside>
  );
}
