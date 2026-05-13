import { formatHourValue, totalMinutes } from "../core/timeMath";
import type { AnalysisResult, MinuteTotals, PhaseSummary, ProjectSummary, SelectedPhaseDetail } from "../types/statistics";

type ProjectTableProps = {
  compact?: boolean;
  onPhaseClick?: (detail: SelectedPhaseDetail) => void;
  result: AnalysisResult | null;
};

export function ProjectTable({ result, compact = false, onPhaseClick }: ProjectTableProps) {
  return (
    <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
      <div className="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-4 border-b border-stone-200 px-4 py-3">
        <h3 className="font-bold">Project summary</h3>
        <span className="min-w-0 truncate text-right text-xs text-slate-500">{result?.source_path ?? ""}</span>
      </div>
      <div className="min-h-0 overflow-auto">
        <table className="w-full min-w-[980px] border-collapse">
          <thead>
            <tr>
              <th className="table-head">Project</th>
              <th className="table-head">Phase</th>
              <th className="table-head num">Regular (hour)</th>
              <th className="table-head num">Normal OT (hour)</th>
              <th className="table-head num">Holiday OT (hour)</th>
              <th className="table-head num">Public Holiday OT (hour)</th>
              <th className="table-head num">Late Night OT (hour)</th>
              <th className="table-head num">Total (hour)</th>
            </tr>
          </thead>
          <tbody>
            {!result || result.projects.length === 0 ? (
              <tr>
                <td className="table-cell h-40 text-center text-slate-500" colSpan={8}>
                  No analysis data yet.
                </td>
              </tr>
            ) : (
              result.projects.map((project) => (
                <ProjectRows
                  key={project.project_code}
                  project={project}
                  compact={compact}
                  onPhaseClick={onPhaseClick}
                />
              ))
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function ProjectRows({
  project,
  compact,
  onPhaseClick,
}: {
  project: ProjectSummary;
  compact: boolean;
  onPhaseClick?: (detail: SelectedPhaseDetail) => void;
}) {
  const phases = compact ? project.phases.slice(0, 4) : project.phases;
  const projectLabel = `${project.project_code} ${project.project_name}`.trim();
  const openPhaseDetail = (phase: PhaseSummary) => {
    onPhaseClick?.({
      project_code: project.project_code,
      project_name: project.project_name,
      phase,
    });
  };

  return (
    <>
      <tr className="bg-emerald-50 font-bold">
        <td className="table-cell">
          <strong>{projectLabel}</strong>
        </td>
        <td className="table-cell">All phases</td>
        <TotalsCells totals={project.totals} />
      </tr>
      {phases.map((phase) => (
        <tr
          key={`${project.project_code}-${phase.process_code}`}
          className={onPhaseClick ? "cursor-pointer hover:bg-slate-50" : undefined}
          title={onPhaseClick ? "Open phase details" : undefined}
          onClick={() => openPhaseDetail(phase)}
        >
          <td className="table-cell"></td>
          <td className="table-cell">
            <span className="mr-2 inline-block min-w-8 rounded bg-blue-100 px-1.5 py-0.5 text-center text-xs font-extrabold text-blue-800">
              {phase.process_code}
            </span>
            {phase.phase_name}
          </td>
          <TotalsCells totals={phase.totals} />
        </tr>
      ))}
    </>
  );
}

function TotalsCells({ totals }: { totals: MinuteTotals }) {
  return (
    <>
      <td className="table-cell num">{formatHourValue(totals.regular_minutes)}</td>
      <td className="table-cell num">{formatHourValue(totals.normal_overtime_minutes)}</td>
      <td className="table-cell num">{formatHourValue(totals.legal_holiday_overtime_minutes)}</td>
      <td className="table-cell num">{formatHourValue(totals.legal_public_holiday_overtime_minutes)}</td>
      <td className="table-cell num">{formatHourValue(totals.late_night_overtime_minutes)}</td>
      <td className="table-cell num font-extrabold text-brand">{formatHourValue(totalMinutes(totals))}</td>
    </>
  );
}
