import { ArrowLeft } from "lucide-react";
import { Button } from "primereact/button";
import { Column } from "primereact/column";
import { DataTable } from "primereact/datatable";
import { emptyTotals, formatHourValue, totalMinutes } from "../core/timeMath";
import type {
  AnalysisResult,
  ImportReportDetail,
  MessageMode,
  ProjectSummary,
  SelectedPhaseDetail,
} from "../types/statistics";

type ImportReportDetailPageProps = {
  detail: ImportReportDetail | null;
  isLoading: boolean;
  message: string;
  messageMode: MessageMode;
  onBack: () => void;
  onOpenDetail: (detail: SelectedPhaseDetail) => void;
};

export function ImportReportDetailPage({
  detail,
  isLoading,
  message,
  messageMode,
  onBack,
  onOpenDetail,
}: ImportReportDetailPageProps) {
  const summary = detail ? buildImportSummary(detail) : null;
  const rows = summary?.projects.flatMap(buildImportProjectRows) ?? [];

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <div className="flex items-center justify-between gap-3">
        <Button
          className="inline-flex h-10 items-center justify-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-700 hover:bg-slate-50"
          type="button"
          onClick={onBack}
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </Button>
      </div>

      <section className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
        <div className="border-b border-stone-200 pb-3">
          <h3 className="font-bold">Thông tin cơ bản</h3>
        </div>
        {!detail ? (
          <p className="py-8 text-center text-sm text-slate-500">
            {isLoading ? "Loading report detail..." : "No report detail."}
          </p>
        ) : (
          <div className="mt-4 grid grid-cols-4 gap-3 text-sm">
            <InfoItem label="SEQ" value={`#${detail.id}`} />
            <InfoItem label="Tên report" value={detail.report_name || "-"} />
            <InfoItem label="Tên file đã import" value={detail.source_file_name || "-"} />
            <InfoItem label="Ngày giờ import" value={detail.imported_at || "-"} />
            <InfoItem label="Người import" value={detail.imported_by || "-"} />
            <InfoItem label="Tháng target" value={formatTargetMonthRange(detail)} />
            <InfoItem label="Rows" value={detail.row_count.toLocaleString("en-US")} />
            <InfoItem label="Hours" value={formatHourValue(detail.total_minutes)} />
            <InfoItem className="col-span-2" label="Ghi chú" value={detail.note || "-"} />
            <InfoItem className="col-span-2" label="Source path" value={detail.source_path || "-"} />
          </div>
        )}
      </section>

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
        <div className="border-b border-stone-200 px-4 py-3">
          <h3 className="font-bold">Danh sách chi tiết của report</h3>
          <p className="mt-1 truncate text-xs text-slate-500">
            {detail ? `${detail.source_file_name} - saved batch #${detail.id}` : "No report data."}
          </p>
        </div>
        <DataTable
          className="app-data-table min-h-0"
          emptyMessage={isLoading ? "Loading report rows..." : "No report rows."}
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
    </section>
  );
}

function InfoItem({
  className = "",
  label,
  value,
}: {
  className?: string;
  label: string;
  value: string;
}) {
  return (
    <div className={`min-w-0 rounded-md border border-stone-200 bg-slate-50 px-3 py-2 ${className}`}>
      <span className="block text-xs font-bold text-slate-500">{label}</span>
      <strong className="mt-1 block truncate text-slate-900" title={value}>
        {value}
      </strong>
    </div>
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

function buildImportSummary(detail: ImportReportDetail): AnalysisResult {
  const projects = new Map<string, ProjectSummary>();

  for (const row of detail.preview_rows) {
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
    source_path: detail.source_path,
    row_count: detail.row_count,
    grand_total: {
      regular_minutes: detail.total_minutes,
      normal_overtime_minutes: 0,
      legal_holiday_overtime_minutes: 0,
      legal_public_holiday_overtime_minutes: 0,
      late_night_overtime_minutes: 0,
    },
    projects: sortedProjects,
  };
}

function formatTargetMonthRange(detail: ImportReportDetail) {
  if (!detail.target_month_from && !detail.target_month_to) {
    return "No target month";
  }

  if (detail.target_month_from === detail.target_month_to || !detail.target_month_to) {
    return detail.target_month_from;
  }

  if (!detail.target_month_from) {
    return detail.target_month_to;
  }

  return `${detail.target_month_from} - ${detail.target_month_to}`;
}
