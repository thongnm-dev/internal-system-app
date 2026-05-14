import { ArrowLeft } from "lucide-react";
import { MessageBanner } from "../components/MessageBanner";
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

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <div className="flex items-center justify-between gap-3">
        <button
          className="inline-flex h-10 items-center justify-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-700 hover:bg-slate-50"
          type="button"
          onClick={onBack}
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </button>
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
              {!summary ? (
                <tr>
                  <td className="table-cell h-48 text-center text-slate-500" colSpan={5}>
                    {isLoading ? "Loading report rows..." : "No report rows."}
                  </td>
                </tr>
              ) : (
                summary.projects.map((project) => (
                  <ImportProjectRows key={project.project_code} project={project} onOpenDetail={onOpenDetail} />
                ))
              )}
            </tbody>
          </table>
        </div>
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
