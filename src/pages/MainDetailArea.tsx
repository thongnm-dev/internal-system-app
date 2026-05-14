import { SummaryCards } from "../components/SummaryCards";
import { useImportCsvController } from "../controller/useImportCsvController";
import { useOverviewController } from "../controller/useOverviewController";
import { usePhasesController } from "../controller/usePhasesController";
import { useProjectsController } from "../controller/useProjectsController";
import type { MenuKey, SelectedPhaseDetail } from "../types/statistics";
import { ImportCsvPage } from "./ImportCsvPage";
import { OverviewPage } from "./OverviewPage";
import { PhasesPage } from "./PhasesPage";
import { ProjectsPage } from "./ProjectsPage";

type MainDetailAreaProps = {
  activeMenu: MenuKey;
  onPhaseClick: (detail: SelectedPhaseDetail) => void;
};

export function MainDetailArea({ activeMenu, onPhaseClick }: MainDetailAreaProps) {
  if (activeMenu === "importCsv") {
    return <ImportCsvRoute onPhaseClick={onPhaseClick} />;
  }

  if (activeMenu === "projects") {
    return <ProjectsRoute onPhaseClick={onPhaseClick} />;
  }

  if (activeMenu === "phases") {
    return <PhasesRoute />;
  }

  return <OverviewRoute onPhaseClick={onPhaseClick} />;
}

function OverviewRoute({ onPhaseClick }: { onPhaseClick: (detail: SelectedPhaseDetail) => void }) {
  const { result, summaryMetrics } = useOverviewController(onPhaseClick);

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <SummaryCards metrics={summaryMetrics} />
      <OverviewPage result={result} onPhaseClick={onPhaseClick} />
    </section>
  );
}

function ProjectsRoute({ onPhaseClick }: { onPhaseClick: (detail: SelectedPhaseDetail) => void }) {
  const { result, summaryMetrics } = useProjectsController(onPhaseClick);

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <SummaryCards metrics={summaryMetrics} />
      <ProjectsPage result={result} onPhaseClick={onPhaseClick} />
    </section>
  );
}

function PhasesRoute() {
  const { result, summaryMetrics } = usePhasesController();

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <SummaryCards metrics={summaryMetrics} />
      <PhasesPage result={result} />
    </section>
  );
}

function ImportCsvRoute({ onPhaseClick }: { onPhaseClick: (detail: SelectedPhaseDetail) => void }) {
  const {
    batches,
    csvPath,
    isImporting,
    isSaving,
    message,
    messageMode,
    pickCsvFile,
    previewCsv,
    previewResult,
    result,
    saveCsv,
    setCsvPath,
  } = useImportCsvController();

  return (
    <ImportCsvPage
      batches={batches}
      csvPath={csvPath}
      importPreviewResult={previewResult}
      importResult={result}
      isImporting={isImporting}
      isSavingImport={isSaving}
      message={message}
      messageMode={messageMode}
      onCsvPathChange={setCsvPath}
      onImport={() => void previewCsv()}
      onOpenDetail={onPhaseClick}
      onPickCsvFile={() => void pickCsvFile()}
      onSave={() => void saveCsv()}
    />
  );
}
