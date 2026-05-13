import { ProjectTable } from "../components/ProjectTable";
import type { AnalysisResult, SelectedPhaseDetail } from "../types/statistics";

type ProjectsPageProps = {
  onPhaseClick: (detail: SelectedPhaseDetail) => void;
  result: AnalysisResult | null;
};

export function ProjectsPage({ result, onPhaseClick }: ProjectsPageProps) {
  return <ProjectTable result={result} onPhaseClick={onPhaseClick} />;
}
