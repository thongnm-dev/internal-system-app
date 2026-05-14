import { SummaryCards } from "../components/SummaryCards";
import { useImportCsvController } from "../controller/useImportCsvController";
import { useOverviewController } from "../controller/useOverviewController";
import { usePhasesController } from "../controller/usePhasesController";
import { useProjectsController } from "../controller/useProjectsController";
import { useSettingsController } from "../controller/useSettingsController";
import type { MenuKey, SelectedPhaseDetail } from "../types/statistics";
import { ImportCsvPage } from "./ImportCsvPage";
import { OverviewPage } from "./OverviewPage";
import { ProjectDetailPage } from "./ProjectDetailPage";
import { PhasesPage } from "./PhasesPage";
import { ProjectsPage } from "./ProjectsPage";
import { SettingsPage } from "./SettingsPage";

type MainDetailAreaProps = {
  activeMenu: MenuKey;
  path: string;
  navigateToPath: (path: string) => void;
  onPhaseClick: (detail: SelectedPhaseDetail) => void;
};

export function MainDetailArea({ activeMenu, path, navigateToPath, onPhaseClick }: MainDetailAreaProps) {
  if (activeMenu === "importCsv") {
    return <ImportCsvRoute onPhaseClick={onPhaseClick} />;
  }

  if (activeMenu === "projects") {
    return <ProjectsRoute path={path} onNavigate={navigateToPath} />;
  }

  if (activeMenu === "phases") {
    return <PhasesRoute />;
  }

  if (activeMenu === "settings") {
    return <SettingsRoute />;
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

function ProjectsRoute({ path, onNavigate }: { path: string; onNavigate: (path: string) => void }) {
  const projectID = getProjectIDFromPath(path);

  if (path.startsWith("/projects/detail")) {
    return (
      <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
        <ProjectDetailPage projectID={projectID} onBack={() => onNavigate("/projects")} />
      </section>
    );
  }

  return <ProjectListRoute onNavigate={onNavigate} />;
}

function ProjectListRoute({ onNavigate }: { onNavigate: (path: string) => void }) {
  const {
    applicationName,
    filters,
    isSearching,
    result,
    resetFilters,
    searchError,
    searchProjects,
    setFilters,
  } = useProjectsController();

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <ProjectsPage
        applicationName={applicationName}
        filters={filters}
        isSearching={isSearching}
        result={result}
        searchError={searchError}
        onNavigate={onNavigate}
        onReset={resetFilters}
        onSearch={() => void searchProjects()}
        onSetFilters={setFilters}
      />
    </section>
  );
}

function getProjectIDFromPath(path: string) {
  const prefix = "/projects/detail/";
  if (!path.startsWith(prefix)) {
    return null;
  }

  return decodeURIComponent(path.slice(prefix.length));
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

function SettingsRoute() {
  const { addApiKey, apiKeyCount, removeApiKey, settings, updateApiKey, updateLanguage, updateTheme, updateUser } =
    useSettingsController();

  return (
    <SettingsPage
      apiKeyCount={apiKeyCount}
      apiKeys={settings.apiKeys}
      language={settings.language}
      onAddApiKey={addApiKey}
      onApiKeyChange={updateApiKey}
      onLanguageChange={updateLanguage}
      onRemoveApiKey={removeApiKey}
      onThemeChange={updateTheme}
      onUserChange={updateUser}
      theme={settings.theme}
      user={settings.user}
    />
  );
}
