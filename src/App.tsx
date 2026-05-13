import { BottomBar } from "./components/BottomBar";
import { Header } from "./components/Header";
import { MessageBanner } from "./components/MessageBanner";
import { PhaseDetailDialog } from "./components/PhaseDetailDialog";
import { Sidebar } from "./components/Sidebar";
import { useDashboardController } from "./controller/useDashboardController";
import { MainDetailArea } from "./pages/MainDetailArea";

export function App() {
  const {
    activeMenu,
    analyze,
    csvPath,
    isSidebarCollapsed,
    isLoading,
    message,
    messageMode,
    pickCsvFile,
    result,
    setActiveMenu,
    setCsvPath,
    setIsSidebarCollapsed,
    selectedPhaseDetail,
    setSelectedPhaseDetail,
    summaryMetrics,
    systemInfo,
  } = useDashboardController();

  return (
    <main className="grid min-h-screen grid-rows-[minmax(0,1fr)_auto] bg-canvas text-ink">
      <section
        className={[
          "grid min-h-0 overflow-hidden transition-[grid-template-columns] duration-200",
          isSidebarCollapsed ? "grid-cols-[72px_minmax(0,1fr)]" : "grid-cols-[240px_minmax(0,1fr)]",
        ].join(" ")}
      >
        <Sidebar
          activeMenu={activeMenu}
          isCollapsed={isSidebarCollapsed}
          onMenuChange={setActiveMenu}
          onToggleCollapse={() => setIsSidebarCollapsed((value) => !value)}
          result={result}
        />

        <section className="min-h-0 overflow-hidden p-6">
          <div className="flex h-full min-h-0 flex-col gap-4">
            <Header
              csvPath={csvPath}
              isLoading={isLoading}
              onCsvPathChange={setCsvPath}
              onAnalyze={() => void analyze()}
              onPickCsvFile={() => void pickCsvFile()}
            />

            <MessageBanner message={message} mode={messageMode} />
            <MainDetailArea
              activeMenu={activeMenu}
              result={result}
              summaryMetrics={summaryMetrics}
              onPhaseClick={setSelectedPhaseDetail}
            />
          </div>
        </section>
      </section>

      <BottomBar info={systemInfo} />
      <PhaseDetailDialog detail={selectedPhaseDetail} onClose={() => setSelectedPhaseDetail(null)} />
    </main>
  );
}
