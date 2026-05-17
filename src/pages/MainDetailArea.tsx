import { useImportCsvController } from "../controller/useImportCsvController";
import { useImportReportDetailController, useImportReportsController } from "../controller/useImportReportsController";
import { useDailyReportController } from "../controller/useDailyReportController";
import { useProjectSkillsController } from "../controller/useProjectSkillsController";
import { useProjectsController } from "../controller/useProjectsController";
import { useSettingsController } from "../controller/useSettingsController";
import { useAuthStore } from "../stores/authStore";
import type { MenuKey, SelectedPhaseDetail } from "../types/statistics";
import { DailyReportPage } from "./DailyReportPage";
import { ImportCsvPage } from "./ImportCsvPage";
import { ImportReportDetailPage } from "./ImportReportDetailPage";
import { ImportReportsPage } from "./ImportReportsPage";
import { ImportIssuesPage } from "./ImportIssuesPage";
import { IssueBacklogPage } from "./IssueBacklogPage";
import { OverviewPage } from "./OverviewPage";
import { ProjectDetailPage } from "./ProjectDetailPage";
import { ProjectSkillsPage } from "./ProjectSkillsPage";
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

  if (activeMenu === "importReports") {
    return <ImportReportsRoute path={path} onNavigate={navigateToPath} onPhaseClick={onPhaseClick} />;
  }

  if (activeMenu === "projects") {
    return <ProjectsRoute path={path} onNavigate={navigateToPath} />;
  }

  if (activeMenu === "issueBacklog") {
    return <IssueBacklogRoute />;
  }

  if (activeMenu === "importIssues") {
    return <ImportIssuesRoute />;
  }

  if (activeMenu === "dailyReport") {
    return <DailyReportRoute />;
  }

  if (activeMenu === "settings") {
    return <SettingsRoute />;
  }

  return <OverviewRoute onPhaseClick={onPhaseClick} />;
}

function ImportReportsRoute({
  onNavigate,
  onPhaseClick,
  path,
}: {
  onNavigate: (path: string) => void;
  onPhaseClick: (detail: SelectedPhaseDetail) => void;
  path: string;
}) {
  const reportId = getImportReportIdFromPath(path);
  if (reportId !== null) {
    return (
      <ImportReportDetailRoute
        reportId={reportId}
        onBack={() => onNavigate("/import-reports")}
        onPhaseClick={onPhaseClick}
      />
    );
  }

  return <ImportReportsListRoute onNavigate={onNavigate} />;
}

function ImportReportsListRoute({ onNavigate }: { onNavigate: (path: string) => void }) {
  const { criteria, isSearching, items, message, messageMode, reset, search, setCriteria } =
    useImportReportsController();

  return (
    <ImportReportsPage
      criteria={criteria}
      isSearching={isSearching}
      items={items}
      message={message}
      messageMode={messageMode}
      onReset={reset}
      onSearch={() => void search()}
      onSetCriteria={setCriteria}
      onOpenReport={(reportId) => onNavigate(`/import-reports/detail/${reportId}`)}
    />
  );
}

function ImportReportDetailRoute({
  onBack,
  onPhaseClick,
  reportId,
}: {
  onBack: () => void;
  onPhaseClick: (detail: SelectedPhaseDetail) => void;
  reportId: number;
}) {
  const { detail, isLoading, message, messageMode } = useImportReportDetailController(reportId);

  return (
    <ImportReportDetailPage
      detail={detail}
      isLoading={isLoading}
      message={message}
      messageMode={messageMode}
      onBack={onBack}
      onOpenDetail={onPhaseClick}
    />
  );
}

function OverviewRoute({  }: { onPhaseClick: (detail: SelectedPhaseDetail) => void }) {

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">

      <OverviewPage />
    </section>
  );
}

function ProjectsRoute({ path, onNavigate }: { path: string; onNavigate: (path: string) => void }) {
  const projectID = getProjectIDFromPath(path);
  const projectSkillCode = getProjectSkillCodeFromPath(path);

  if (projectSkillCode !== null) {
    return (
      <ProjectSkillsRoute
        projectCode={projectSkillCode}
        projectName={getProjectNameFromPath(path)}
        onBack={() => onNavigate("/projects")}
      />
    );
  }

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

function ProjectSkillsRoute({
  onBack,
  projectCode,
  projectName,
}: {
  onBack: () => void;
  projectCode: string;
  projectName: string;
}) {
  const projectSkills = useProjectSkillsController(projectCode, projectName);

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <ProjectSkillsPage
        draft={projectSkills.draft}
        generatedMarkdown={projectSkills.generatedMarkdown}
        message={projectSkills.message}
        messageMode={projectSkills.messageMode}
        onBack={onBack}
        onReset={projectSkills.resetDraft}
        onSave={projectSkills.saveDraft}
        onUpdateDraft={projectSkills.updateDraft}
        onUpdateRole={projectSkills.updateRole}
      />
    </section>
  );
}

function IssueBacklogRoute() {
  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <IssueBacklogPage />
    </section>
  );
}

function ImportIssuesRoute() {
  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <ImportIssuesPage />
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

function getProjectSkillCodeFromPath(path: string) {
  const prefix = "/projects/skills/";
  if (!path.startsWith(prefix)) {
    return null;
  }

  return decodeURIComponent(path.slice(prefix.length).split("?")[0]);
}

function getProjectNameFromPath(path: string) {
  const query = path.split("?")[1];
  if (!query) {
    return "";
  }

  return new URLSearchParams(query).get("name") ?? "";
}

function getImportReportIdFromPath(path: string) {
  const prefix = "/import-reports/detail/";
  if (!path.startsWith(prefix)) {
    return null;
  }

  const reportId = Number(decodeURIComponent(path.slice(prefix.length)));
  return Number.isInteger(reportId) && reportId > 0 ? reportId : null;
}

function DailyReportRoute() {
  const { user } = useAuthStore();
  const dailyReport = useDailyReportController(user?.username);

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <DailyReportPage
        availableProjects={dailyReport.availableProjects}
        canGoNextMonth={dailyReport.canGoNextMonth}
        days={dailyReport.days}
        entries={dailyReport.entries}
        maxMonthValue={dailyReport.maxMonthValue}
        monthLabel={dailyReport.monthLabel}
        monthValue={dailyReport.monthValue}
        projects={dailyReport.projects}
        totalHours={dailyReport.totalHours}
        onAddProject={dailyReport.addProject}
        onNextMonth={dailyReport.nextMonth}
        onPreviousMonth={dailyReport.previousMonth}
        onRemoveProject={dailyReport.removeProject}
        onSelectMonth={dailyReport.selectMonth}
        onUpdateEntry={dailyReport.updateEntry}
      />
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
    note,
    pickCsvFile,
    previewCsv,
    reportName,
    previewResult,
    result,
    saveCsv,
    setCsvPath,
    setNote,
    setReportName,
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
      note={note}
      reportName={reportName}
      onCsvPathChange={setCsvPath}
      onImport={() => void previewCsv()}
      onNoteChange={setNote}
      onOpenDetail={onPhaseClick}
      onPickCsvFile={() => void pickCsvFile()}
      onReportNameChange={setReportName}
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
