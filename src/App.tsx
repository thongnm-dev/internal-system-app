import { useEffect } from "react";
import { BottomBar } from "./components/BottomBar";
import { Header } from "./components/Header";
import { MessageBanner } from "./components/MessageBanner";
import { PhaseDetailDialog } from "./components/PhaseDetailDialog";
import { Sidebar } from "./components/Sidebar";
import { useAppShellController } from "./controller/useAppShellController";
import { ReportDataProvider, useReportDataController } from "./controller/useReportDataController";
import { defaultRoute, loginRoute } from "./router/routes";
import { MainDetailArea } from "./pages/MainDetailArea";
import { LoginPage } from "./pages/LoginPage";
import { AuthProvider, useAuthStore } from "./stores/authStore";
import type { MenuKey } from "./types/statistics";

export function App() {
  return (
    <AuthProvider>
      <ReportDataProvider>
        <AppShell />
      </ReportDataProvider>
    </AuthProvider>
  );
}

function AppShell() {
  const {
    activeMenu,
    isSidebarCollapsed,
    navigate,
    navigateToPath,
    path,
    route,
    setIsSidebarCollapsed,
    selectedPhaseDetail,
    setSelectedPhaseDetail,
    systemInfo,
  } = useAppShellController();
  const { message, messageMode } = useReportDataController();
  const { isAuthenticated, login, logout, returnPath, setReturnPath, user } = useAuthStore();

  useEffect(() => {
    if (route.requiresAuth && !isAuthenticated) {
      setReturnPath(route.path);
      navigateToPath(loginRoute.path);
      return;
    }

    if (route.key === "login" && isAuthenticated) {
      navigateToPath(returnPath ?? defaultRoute.path);
    }
  }, [isAuthenticated, navigateToPath, returnPath, route, setReturnPath]);

  if (route.key === "login") {
    return (
      <LoginPage
        onLogin={(username, _password) => {
          login({ username });
          navigateToPath(returnPath ?? defaultRoute.path);
        }}
      />
    );
  }

  if (route.requiresAuth && !isAuthenticated) {
    return null;
  }

  const activeShellMenu = activeMenu as MenuKey;

  return (
    <main className="grid min-h-screen grid-rows-[minmax(0,1fr)_auto] bg-canvas text-ink">
      <section
        className={[
          "grid min-h-0 overflow-hidden transition-[grid-template-columns] duration-200",
          isSidebarCollapsed ? "grid-cols-[72px_minmax(0,1fr)]" : "grid-cols-[240px_minmax(0,1fr)]",
        ].join(" ")}
      >
        <Sidebar
          activeMenu={activeShellMenu}
          isCollapsed={isSidebarCollapsed}
          onMenuChange={navigate}
          onToggleCollapse={() => setIsSidebarCollapsed((value) => !value)}
        />

        <section className="min-h-0 overflow-hidden p-6">
          <div className="flex h-full min-h-0 flex-col gap-4">
            <Header
              route={route}
              username={user?.username}
              onLogout={() => {
                logout();
                navigateToPath(loginRoute.path);
              }}
            />

            <MainDetailArea
              activeMenu={activeShellMenu}
              path={path}
              navigateToPath={navigateToPath}
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
