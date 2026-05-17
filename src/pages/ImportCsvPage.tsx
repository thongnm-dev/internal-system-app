import { Database, FileInput, FolderOpen, List } from "lucide-react";
import { Button } from "primereact/button";
import { Column } from "primereact/column";
import { DataTable } from "primereact/datatable";
import { Dialog } from "primereact/dialog";
import { InputText } from "primereact/inputtext";
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

          <div className="mt-4 grid grid-cols-[1fr_auto_auto] gap-2">
            <InputText
              className="h-10 min-w-0 rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Select CSV file..."
              type="text"
              value={csvPath}
              onChange={(event) => onCsvPathChange(event.target.value)}
            />
            <Button
              className="flex h-10 p-4 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700 hover:bg-slate-50"
              type="button"
              title="Browse CSV"
              onClick={onPickCsvFile}
            >
              <FolderOpen className="h-4 w-4 " />
            </Button>
            <Button
              className="flex h-10 items-center justify-center gap-2 rounded-md bg-slate-800 px-3 text-sm font-bold text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              disabled={isImporting}
              onClick={onImport}
            >
              <FileInput className="h-4 w-4" />
              Import
            </Button>
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
  const rows = importSummary?.projects.flatMap(buildImportProjectRows) ?? [];
  const [isDetailDialogOpen, setIsDetailDialogOpen] = useState(false);

  return (
    <>
      <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
        <div className="grid items-start gap-3 border-b border-stone-200 px-4 py-3">
          <div className="min-w-0">
            <h3 className="font-bold">Preview</h3>
            <p className="mt-1 truncate text-xs text-slate-500">
              {result
                ? `${result.source_file_name}${saveResult ? ` - saved batch #${saveResult.batch_id}` : ""}`
                : "No CSV data imported yet."}
            </p>
          </div>
          <Button
            className="inline-flex h-9 items-center justify-center gap-2 rounded-md border border-slate-200 bg-white px-3 text-xs font-bold text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            disabled={!result}
            title="Show detail list"
            onClick={() => setIsDetailDialogOpen(true)}
          >
            <List className="h-4 w-4" />
            Show detail
          </Button>
        </div>
        <DataTable
          className="app-data-table min-h-0"
          emptyMessage="No data. Select a CSV file, then click Import."
          rowClassName={(row: ImportProjectTableRow) => (row.kind === "project" ? "bg-emerald-50 font-bold" : "cursor-pointer")}
          scrollable
          scrollHeight="flex"
          tableStyle={{ minWidth: "920px" }}
          value={rows}
          onRowClick={(event) => {
            if (event.data.kind === "phase") {
              onOpenDetail({
                project_code: event.data.project.project_code,
                project_name: event.data.project.project_name,
                phase: event.data.phase,
              });
            }
          }}
        >
          <Column header="Project" body={importProjectBody} />
          <Column header="Phase" body={importPhaseBody} />
          <Column header="Rows" body={(row: ImportProjectTableRow) => row.rowCount.toLocaleString("en-US")} bodyClassName="num" headerClassName="num" />
          <Column header="Total (hour)" body={(row: ImportProjectTableRow) => formatHourValue(totalMinutes(row.totals))} bodyClassName="num font-bold text-brand" headerClassName="num" />
          <Column header="Detail" body={(row: ImportProjectTableRow) => detailButtonBody(row, onOpenDetail)} bodyClassName="num" headerClassName="num" />
        </DataTable>
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

type ImportProjectTableRow =
  | { id: string; kind: "project"; project: ProjectSummary; phase: null; rowCount: number; totals: ProjectSummary["totals"] }
  | { id: string; kind: "phase"; project: ProjectSummary; phase: ProjectSummary["phases"][number]; rowCount: number; totals: ProjectSummary["totals"] };

function buildImportProjectRows(project: ProjectSummary): ImportProjectTableRow[] {
  return [
    { id: `${project.project_code}-all`, kind: "project", project, phase: null, rowCount: project.row_count, totals: project.totals },
    ...project.phases.map((phase) => ({
      id: `${project.project_code}-${phase.process_code}`,
      kind: "phase" as const,
      project,
      phase,
      rowCount: phase.row_count,
      totals: phase.totals,
    })),
  ];
}

function importProjectBody(row: ImportProjectTableRow) {
  if (row.kind === "phase") {
    return "";
  }

  return <strong>{`${row.project.project_code} ${row.project.project_name}`.trim()}</strong>;
}

function importPhaseBody(row: ImportProjectTableRow) {
  if (row.kind === "project") {
    return "All phases";
  }

  return (
    <>
      <span className="mr-2 inline-block min-w-8 rounded bg-blue-100 px-1.5 py-0.5 text-center text-xs font-extrabold text-blue-800">
        {row.phase.process_code}
      </span>
      {row.phase.phase_name}
    </>
  );
}

function detailButtonBody(row: ImportProjectTableRow, onOpenDetail: (detail: SelectedPhaseDetail) => void) {
  if (row.kind === "project") {
    return "";
  }

  return (
    <Button
      className="inline-flex h-8 items-center justify-center rounded-md border border-slate-200 px-3 text-xs font-bold text-slate-600 hover:bg-slate-50"
      type="button"
      title="Open detail"
      onClick={(event) => {
        event.stopPropagation();
        onOpenDetail({
          project_code: row.project.project_code,
          project_name: row.project.project_name,
          phase: row.phase,
        });
      }}
    >
      Detail
    </Button>
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
    <Dialog
      className="w-full max-w-6xl overflow-hidden rounded-lg bg-white shadow-2xl"
      contentClassName="p-0 min-h-0 overflow-auto"
      header={
        <div className="min-w-0">
          <h3 className="truncate text-lg font-bold text-slate-900">CSV detail</h3>
          <p className="mt-1 truncate text-sm text-slate-500">
            {result.source_file_name}
            {saveResult ? ` - saved batch #${saveResult.batch_id}` : ""}
          </p>
        </div>
      }
      style={{ maxHeight: "86vh" }}
      visible={isOpen}
      onHide={onClose}
    >
        <DataTable
          className="app-data-table min-h-0"
          emptyMessage="No CSV rows."
          scrollable
          scrollHeight="flex"
          tableStyle={{ minWidth: `${tableMinWidth}px` }}
          value={result.raw_rows.map((cells, index) => ({ cells, id: index }))}
        >
          {visibleColumns.map((column) => {
            const isMinuteColumn = minuteColumns.has(column.index);
            return (
              <Column
                key={`${column.header}-${column.index}`}
                header={column.header}
                body={(row: { cells: string[] }) => (isMinuteColumn ? formatCsvMinuteValue(row.cells[column.index]) : row.cells[column.index] || "")}
                bodyClassName={isMinuteColumn ? "num" : undefined}
                headerClassName={isMinuteColumn ? "num" : undefined}
              />
            );
          })}
        </DataTable>
    </Dialog>
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
