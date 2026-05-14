import { BottomBar } from "./components/BottomBar";
import { Header } from "./components/Header";
import { MessageBanner } from "./components/MessageBanner";
import { PhaseDetailDialog } from "./components/PhaseDetailDialog";
import { Sidebar } from "./components/Sidebar";
import { useAppShellController } from "./controller/useAppShellController";
import { ReportDataProvider, useReportDataController } from "./controller/useReportDataController";
import { MainDetailArea } from "./pages/MainDetailArea";

export function App() {
  return (
    <ReportDataProvider>
      <AppShell />
    </ReportDataProvider>
  );
}

function AppShell() {
  const {
    activeMenu,
    isSidebarCollapsed,
    navigate,
    route,
    setIsSidebarCollapsed,
    selectedPhaseDetail,
    setSelectedPhaseDetail,
    systemInfo,
  } = useAppShellController();
  const { message, messageMode, result } = useReportDataController();

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
          onMenuChange={navigate}
          onToggleCollapse={() => setIsSidebarCollapsed((value) => !value)}
          result={result}
        />

        <section className="min-h-0 overflow-hidden p-6">
          <div className="flex h-full min-h-0 flex-col gap-4">
            <Header route={route} />

            {activeMenu !== "importCsv" && <MessageBanner message={message} mode={messageMode} />}
            <MainDetailArea activeMenu={activeMenu} onPhaseClick={setSelectedPhaseDetail} />
          </div>
        </section>
      </section>

      <BottomBar info={systemInfo} />
      <PhaseDetailDialog detail={selectedPhaseDetail} onClose={() => setSelectedPhaseDetail(null)} />
    </main>
  );
}
