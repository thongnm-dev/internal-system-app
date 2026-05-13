import { Gauge } from "lucide-react";
import { ProjectTable } from "../components/ProjectTable";
import { formatHours, totalMinutes } from "../core/timeMath";
import type { AnalysisResult, SelectedPhaseDetail } from "../types/statistics";

type OverviewPageProps = {
  onPhaseClick: (detail: SelectedPhaseDetail) => void;
  result: AnalysisResult | null;
};

export function OverviewPage({ result, onPhaseClick }: OverviewPageProps) {
  const topProjects = result?.projects.slice(0, 5) ?? [];

  return (
    <section className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_320px] gap-4 overflow-hidden">
      <ProjectTable result={result} compact onPhaseClick={onPhaseClick} />
      <aside className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
        <div className="flex items-center gap-2">
          <Gauge className="h-5 w-5 text-brand" />
          <h3 className="font-bold">Top projects</h3>
        </div>
        <div className="mt-4 space-y-3">
          {topProjects.length === 0 ? (
            <p className="text-sm text-slate-500">No data yet.</p>
          ) : (
            topProjects.map((project) => (
              <div key={project.project_code} className="border-b border-stone-100 pb-3 last:border-b-0">
                <div className="flex items-center justify-between gap-3">
                  <span className="min-w-0 truncate text-sm font-bold">{project.project_code}</span>
                  <span className="text-sm font-bold text-brand">{formatHours(totalMinutes(project.totals))}</span>
                </div>
                <p className="mt-1 truncate text-xs text-slate-500">{project.project_name || "No project name"}</p>
              </div>
            ))
          )}
        </div>
      </aside>
    </section>
  );
}
