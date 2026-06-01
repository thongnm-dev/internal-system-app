import { useEffect } from "react";
import { BottomBar } from "./components/BottomBar";
import { Header } from "./components/Header";
import { Sidebar } from "./components/Sidebar";
import { StartupScreen } from "./components/StartupScreen";
import { useAppShellController } from "./controller/useAppShellController";
import { defaultRoute, loginRoute } from "./router/routes";
import { MainDetailArea } from "./pages/MainDetailArea";
import { LoginPage } from "./pages/LoginPage";
import { AuthProvider, useAuthStore } from "./stores/authStore";
import type { MenuKey } from "./types/statistics";

export function App() {
  return (
    <AuthProvider>
      <AppShell />
    </AuthProvider>
  );
}

function AppShell() {
  const {
    activeMenu,
    isBootstrapping,
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

  if (isBootstrapping) {
    return <StartupScreen />;
  }

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
    <main className="grid h-screen grid-rows-[minmax(0,1fr)_auto] overflow-hidden bg-canvas text-ink">
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
          <div className="flex h-full min-h-0 overflow-hidden flex-col gap-4">
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
    </main>
  );
}
