import { SummaryCards } from "../components/SummaryCards";
import type {
  AnalysisResult,
  ImportBatchSummary,
  ImportCsvPreviewResult,
  ImportCsvResult,
  MenuKey,
  MessageMode,
  SelectedPhaseDetail,
  SummaryMetric,
} from "../types/statistics";
import { ImportCsvPage } from "./ImportCsvPage";
import { OverviewPage } from "./OverviewPage";
import { PhasesPage } from "./PhasesPage";
import { ProjectsPage } from "./ProjectsPage";

type MainDetailAreaProps = {
  activeMenu: MenuKey;
  importBatches: ImportBatchSummary[];
  importCsvPath: string;
  importMessage: string;
  importMessageMode: MessageMode;
  importPreviewResult: ImportCsvPreviewResult | null;
  importResult: ImportCsvResult | null;
  isImporting: boolean;
  isSavingImport: boolean;
  onPhaseClick: (detail: SelectedPhaseDetail) => void;
  onImportCsvPathChange: (value: string) => void;
  onImportMonthlyReportCsv: () => void;
  onPickImportCsvFile: () => void;
  onSaveMonthlyReportCsv: () => void;
  result: AnalysisResult | null;
  summaryMetrics: SummaryMetric[];
};

export function MainDetailArea({
  activeMenu,
  importBatches,
  importCsvPath,
  importMessage,
  importMessageMode,
  importPreviewResult,
  importResult,
  isImporting,
  isSavingImport,
  onImportCsvPathChange,
  onImportMonthlyReportCsv,
  onPhaseClick,
  onPickImportCsvFile,
  onSaveMonthlyReportCsv,
  result,
  summaryMetrics,
}: MainDetailAreaProps) {
  if (activeMenu === "importCsv") {
    return (
      <ImportCsvPage
        batches={importBatches}
        csvPath={importCsvPath}
        importPreviewResult={importPreviewResult}
        importResult={importResult}
        isImporting={isImporting}
        isSavingImport={isSavingImport}
        message={importMessage}
        messageMode={importMessageMode}
        onCsvPathChange={onImportCsvPathChange}
        onImport={onImportMonthlyReportCsv}
        onPickCsvFile={onPickImportCsvFile}
        onSave={onSaveMonthlyReportCsv}
      />
    );
  }

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <SummaryCards metrics={summaryMetrics} />
      {activeMenu === "overview" && <OverviewPage result={result} onPhaseClick={onPhaseClick} />}
      {activeMenu === "projects" && <ProjectsPage result={result} onPhaseClick={onPhaseClick} />}
      {activeMenu === "phases" && <PhasesPage result={result} />}
    </section>
  );
}
