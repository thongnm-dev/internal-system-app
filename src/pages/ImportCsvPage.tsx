import { Database, FileInput, FolderOpen, List, Save, X } from "lucide-react";
import { useMemo, useState } from "react";
import { MessageBanner } from "../components/MessageBanner";
import { emptyTotals, formatHourValue, totalMinutes } from "../core/timeMath";
import type {
  AnalysisResult,
  ImportBatchSummary,
  ImportCsvPreviewResult,
  ImportCsvResult,
  MessageMode,
  ProjectSummary,
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
  note: string;
  reportName: string;
  onCsvPathChange: (value: string) => void;
  onImport: () => void;
  onNoteChange: (value: string) => void;
  onOpenDetail: (detail: SelectedPhaseDetail) => void;
  onPickCsvFile: () => void;
  onReportNameChange: (value: string) => void;
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
  note,
  reportName,
  onCsvPathChange,
  onImport,
  onNoteChange,
  onOpenDetail,
  onPickCsvFile,
  onReportNameChange,
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

          <div className="mt-3 grid grid-cols-[minmax(0,280px)_minmax(0,1fr)] gap-2">
            <input
              className="h-10 min-w-0 rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Report name"
              type="text"
              value={reportName}
              onChange={(event) => onReportNameChange(event.target.value)}
            />
            <input
              className="h-10 min-w-0 rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Note"
              type="text"
              value={note}
              onChange={(event) => onNoteChange(event.target.value)}
            />
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
  const importSummary = useMemo(() => (result ? buildImportSummary(result) : null), [result]);
  const [isDetailDialogOpen, setIsDetailDialogOpen] = useState(false);

  return (
    <>
      <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
        <div className="grid grid-cols-[minmax(0,1fr)_auto] items-start gap-3 border-b border-stone-200 px-4 py-3">
          <div className="min-w-0">
            <h3 className="font-bold">Preview</h3>
            <p className="mt-1 truncate text-xs text-slate-500">
              {result
                ? `${result.source_file_name}${saveResult ? ` - saved batch #${saveResult.batch_id}` : ""}`
                : "No CSV data imported yet."}
            </p>
          </div>
          <button
            className="inline-flex h-9 items-center justify-center gap-2 rounded-md border border-slate-200 bg-white px-3 text-xs font-bold text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            disabled={!result}
            title="Show detail list"
            onClick={() => setIsDetailDialogOpen(true)}
          >
            <List className="h-4 w-4" />
            Show detail
          </button>
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
                importSummary?.projects.map((project) => <ImportProjectRows key={project.project_code} project={project} onOpenDetail={onOpenDetail} />)
              )}
            </tbody>
          </table>
        </div>
      </section>

      <ImportDetailListDialog
        isOpen={isDetailDialogOpen}
        onClose={() => setIsDetailDialogOpen(false)}
        result={result}
        saveResult={saveResult}
      />
    </>
  );
}

function ImportProjectRows({
  onOpenDetail,
  project,
}: {
  onOpenDetail: (detail: SelectedPhaseDetail) => void;
  project: ProjectSummary;
}) {
  const projectLabel = `${project.project_code} ${project.project_name}`.trim();

  return (
    <>
      <tr className="bg-emerald-50 font-bold">
        <td className="table-cell">
          <strong>{projectLabel}</strong>
        </td>
        <td className="table-cell">All phases</td>
        <td className="table-cell num">{project.row_count.toLocaleString("en-US")}</td>
        <td className="table-cell num font-bold text-brand">{formatHourValue(totalMinutes(project.totals))}</td>
        <td className="table-cell num"></td>
      </tr>
      {project.phases.map((phase) => (
        <tr
          key={`${project.project_code}-${phase.process_code}`}
          className="cursor-pointer hover:bg-slate-50"
          title="Open phase details"
          onClick={() =>
            onOpenDetail({
              project_code: project.project_code,
              project_name: project.project_name,
              phase,
            })
          }
        >
          <td className="table-cell"></td>
          <td className="table-cell">
            <span className="mr-2 inline-block min-w-8 rounded bg-blue-100 px-1.5 py-0.5 text-center text-xs font-extrabold text-blue-800">
              {phase.process_code}
            </span>
            {phase.phase_name}
          </td>
          <td className="table-cell num">{phase.row_count.toLocaleString("en-US")}</td>
          <td className="table-cell num font-bold text-brand">{formatHourValue(totalMinutes(phase.totals))}</td>
          <td className="table-cell num">
            <button
              className="inline-flex h-8 items-center justify-center rounded-md border border-slate-200 px-3 text-xs font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              title="Open detail"
              onClick={(event) => {
                event.stopPropagation();
                onOpenDetail({
                  project_code: project.project_code,
                  project_name: project.project_name,
                  phase,
                });
              }}
            >
              Detail
            </button>
          </td>
        </tr>
      ))}
    </>
  );
}

function buildImportSummary(result: ImportCsvPreviewResult): AnalysisResult {
  const projects = new Map<string, ProjectSummary>();

  for (const row of result.preview_rows) {
    const project =
      projects.get(row.project_code) ??
      ({
        project_code: row.project_code,
        project_name: row.project_name,
        totals: emptyTotals(),
        row_count: 0,
        phases: [],
      } satisfies ProjectSummary);

    let phase = project.phases.find((item) => item.process_code === row.process_code);
    if (!phase) {
      phase = {
        process_code: row.process_code,
        phase_name: row.phase_name,
        totals: emptyTotals(),
        row_count: 0,
        details: [],
      };
      project.phases.push(phase);
    }

    project.row_count += 1;
    project.totals.regular_minutes += row.total_minutes;
    phase.row_count += 1;
    phase.totals.regular_minutes += row.total_minutes;
    phase.details.push({
      date: row.date,
      work_content: row.work_content,
      total_minutes: row.total_minutes,
    });

    projects.set(row.project_code, project);
  }

  const sortedProjects = Array.from(projects.values()).sort((a, b) => totalMinutes(b.totals) - totalMinutes(a.totals));
  for (const project of sortedProjects) {
    project.phases.sort((a, b) => totalMinutes(b.totals) - totalMinutes(a.totals));
  }

  return {
    source_path: result.source_path,
    row_count: result.row_count,
    grand_total: {
      regular_minutes: result.total_minutes,
      normal_overtime_minutes: 0,
      legal_holiday_overtime_minutes: 0,
      legal_public_holiday_overtime_minutes: 0,
      late_night_overtime_minutes: 0,
    },
    projects: sortedProjects,
  };
}

function ImportDetailListDialog({
  isOpen,
  onClose,
  result,
  saveResult,
}: {
  isOpen: boolean;
  onClose: () => void;
  result: ImportCsvPreviewResult | null;
  saveResult: ImportCsvResult | null;
}) {
  if (!isOpen || !result) {
    return null;
  }

  const minuteColumns = new Set(result.minute_column_indexes);
  const visibleColumns = result.raw_headers
    .map((header, index) => ({ header, index }))
    .filter((column) => !hiddenCsvDetailHeaders.has(column.header));
  const tableMinWidth = Math.max(920, visibleColumns.length * 140);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 p-6">
      <section className="flex max-h-[86vh] w-full max-w-6xl flex-col overflow-hidden rounded-lg bg-white shadow-2xl">
        <header className="flex items-start justify-between gap-4 border-b border-stone-200 px-5 py-4">
          <div className="min-w-0">
            <h3 className="truncate text-lg font-bold text-slate-900">CSV detail</h3>
            <p className="mt-1 truncate text-sm text-slate-500">
              {result.source_file_name}
              {saveResult ? ` - saved batch #${saveResult.batch_id}` : ""}
            </p>
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
          <table className="w-full border-collapse" style={{ minWidth: `${tableMinWidth}px` }}>
            <thead>
              <tr>
                {visibleColumns.map((column) => (
                  <th key={`${column.header}-${column.index}`} className={minuteColumns.has(column.index) ? "table-head num" : "table-head"}>
                    {column.header}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {result.raw_rows.length === 0 ? (
                <tr>
                  <td className="table-cell h-40 text-center text-slate-500" colSpan={Math.max(1, visibleColumns.length)}>
                    No CSV rows.
                  </td>
                </tr>
              ) : (
                result.raw_rows.map((row, rowIndex) => (
                  <tr key={rowIndex}>
                    {visibleColumns.map((column) => {
                      const isMinuteColumn = minuteColumns.has(column.index);
                      return (
                        <td key={`${rowIndex}-${column.index}`} className={isMinuteColumn ? "table-cell num" : "table-cell"}>
                          {isMinuteColumn ? formatCsvMinuteValue(row[column.index]) : row[column.index] || ""}
                        </td>
                      );
                    })}
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

const hiddenCsvDetailHeaders = new Set(["社員番号", "給与社員番号", "プロジェクトコード（日本）", "固定プロジェクト名", "プロセス"]);

function formatCsvMinuteValue(value: string | undefined): string {
  const minutes = Number((value ?? "").trim().replace(/,/g, ""));
  if (!Number.isFinite(minutes) || minutes === 0) {
    return "-";
  }

  return formatHourValue(minutes);
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
