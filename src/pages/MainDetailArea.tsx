import { SummaryCards } from "../components/SummaryCards";
import type { AnalysisResult, MenuKey, SelectedPhaseDetail, SummaryMetric } from "../types/statistics";
import { OverviewPage } from "./OverviewPage";
import { PhasesPage } from "./PhasesPage";
import { ProjectsPage } from "./ProjectsPage";

type MainDetailAreaProps = {
  activeMenu: MenuKey;
  onPhaseClick: (detail: SelectedPhaseDetail) => void;
  result: AnalysisResult | null;
  summaryMetrics: SummaryMetric[];
};

export function MainDetailArea({ activeMenu, onPhaseClick, result, summaryMetrics }: MainDetailAreaProps) {
  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <SummaryCards metrics={summaryMetrics} />
      {activeMenu === "overview" && <OverviewPage result={result} onPhaseClick={onPhaseClick} />}
      {activeMenu === "projects" && <ProjectsPage result={result} onPhaseClick={onPhaseClick} />}
      {activeMenu === "phases" && <PhasesPage result={result} />}
    </section>
  );
}
