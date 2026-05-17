import { Column } from "primereact/column";
import { DataTable } from "primereact/datatable";
import { formatHourValue, totalMinutes } from "../core/timeMath";
import type { AnalysisResult, MinuteTotals, PhaseSummary, ProjectSummary, SelectedPhaseDetail } from "../types/statistics";

type ProjectTableProps = {
  compact?: boolean;
  onPhaseClick?: (detail: SelectedPhaseDetail) => void;
  result: AnalysisResult | null;
};

export function ProjectTable({ result, compact = false, onPhaseClick }: ProjectTableProps) {
  const rows = result?.projects.flatMap((project) => buildProjectRows(project, compact)) ?? [];

  return (
    <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
      <div className="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-4 border-b border-stone-200 px-4 py-3">
        <h3 className="font-bold">Project summary</h3>
        <span className="min-w-0 truncate text-right text-xs text-slate-500">{result?.source_path ?? ""}</span>
      </div>
      <DataTable
        className="app-data-table min-h-0"
        emptyMessage="No analysis data yet."
        rowClassName={(row: ProjectTableRow) =>
          [row.kind === "project" ? "bg-emerald-50 font-bold" : "", row.kind === "phase" && onPhaseClick ? "cursor-pointer" : ""].join(" ")
        }
        scrollable
        scrollHeight="flex"
        tableStyle={{ minWidth: "980px" }}
        value={rows}
        onRowClick={(event) => {
          if (event.data.kind === "phase") {
            onPhaseClick?.({
              project_code: event.data.project.project_code,
              project_name: event.data.project.project_name,
              phase: event.data.phase,
            });
          }
        }}
      >
        <Column header="Project" body={projectBody} />
        <Column header="Phase" body={phaseBody} />
        <Column header="Regular (hour)" body={(row: ProjectTableRow) => formatHourValue(row.totals.regular_minutes)} bodyClassName="num" headerClassName="num" />
        <Column header="Normal OT (hour)" body={(row: ProjectTableRow) => formatHourValue(row.totals.normal_overtime_minutes)} bodyClassName="num" headerClassName="num" />
        <Column header="Holiday OT (hour)" body={(row: ProjectTableRow) => formatHourValue(row.totals.legal_holiday_overtime_minutes)} bodyClassName="num" headerClassName="num" />
        <Column header="Public Holiday OT (hour)" body={(row: ProjectTableRow) => formatHourValue(row.totals.legal_public_holiday_overtime_minutes)} bodyClassName="num" headerClassName="num" />
        <Column header="Late Night OT (hour)" body={(row: ProjectTableRow) => formatHourValue(row.totals.late_night_overtime_minutes)} bodyClassName="num" headerClassName="num" />
        <Column header="Total (hour)" body={(row: ProjectTableRow) => formatHourValue(totalMinutes(row.totals))} bodyClassName="num font-extrabold text-brand" headerClassName="num" />
      </DataTable>
    </section>
  );
}

type ProjectTableRow =
  | { id: string; kind: "project"; project: ProjectSummary; phase: null; totals: MinuteTotals }
  | { id: string; kind: "phase"; project: ProjectSummary; phase: PhaseSummary; totals: MinuteTotals };

function buildProjectRows(project: ProjectSummary, compact: boolean): ProjectTableRow[] {
  const phases = compact ? project.phases.slice(0, 4) : project.phases;

  return [
    { id: `${project.project_code}-all`, kind: "project", project, phase: null, totals: project.totals },
    ...phases.map((phase) => ({
      id: `${project.project_code}-${phase.process_code}`,
      kind: "phase" as const,
      project,
      phase,
      totals: phase.totals,
    })),
  ];
}

function projectBody(row: ProjectTableRow) {
  if (row.kind === "phase") {
    return "";
  }

  return <strong>{`${row.project.project_code} ${row.project.project_name}`.trim()}</strong>;
}

function phaseBody(row: ProjectTableRow) {
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
