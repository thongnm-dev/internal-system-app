import {
  CalendarDays,
  ChevronLeft,
  ChevronRight,
  ClipboardList,
  Database,
  FileUp,
  Home,
  ListTodo,
  Settings,
  Table2,
} from "lucide-react";
import { Button } from "primereact/button";
import { appRoutes } from "../router/routes";
import type { MenuKey } from "../types/statistics";

type SidebarProps = {
  activeMenu: MenuKey;
  isCollapsed: boolean;
  onMenuChange: (value: MenuKey) => void;
  onToggleCollapse: () => void;
};

export function Sidebar({ activeMenu, isCollapsed, onMenuChange, onToggleCollapse }: SidebarProps) {
  const items = [
    { id: "overview" as const, icon: Home },
    { id: "projects" as const, icon: Table2 },
    { id: "issueBacklog" as const, icon: ListTodo },
    { id: "importIssues" as const, icon: FileUp },
    { id: "dailyReport" as const, icon: CalendarDays },
    { id: "importCsv" as const, icon: Database },
    { id: "importReports" as const, icon: ClipboardList },
  ];
  const settingsRoute = appRoutes.find((route) => route.key === "settings");
  const settingsLabel = settingsRoute?.title ?? "Settings";

  return (
    <aside className="flex min-h-0 flex-col border-r border-slate-200 bg-slate-900 text-white">
      <div className={["border-b border-white/10", isCollapsed ? "p-3" : "p-5"].join(" ")}>
        <div className={["flex items-center gap-3", isCollapsed ? "justify-center" : "justify-between"].join(" ")}>
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-brand font-bold">PJ</div>
          {!isCollapsed && (
            <Button
              className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-slate-300 hover:bg-white/10 hover:text-white"
              type="button"
              title="Collapse sidebar"
              onClick={onToggleCollapse}
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
          )}
        </div>

        {isCollapsed ? (
          <Button
            className="mt-3 flex h-9 w-full items-center justify-center rounded-md text-slate-300 hover:bg-white/10 hover:text-white"
            type="button"
            title="Expand sidebar"
            onClick={onToggleCollapse}
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
        ) : (
          <>
            <h1 className="mt-4 text-xl font-bold leading-tight">PJ Yuji Statistics</h1>
            <p className="mt-2 text-sm text-slate-300">Project and phase statistics.</p>
          </>
        )}
      </div>

      <nav className={["flex-1 space-y-1", isCollapsed ? "p-2" : "p-3"].join(" ")}>
        {items.map((item) => {
          const Icon = item.icon;
          const isActive = activeMenu === item.id;
          const route = appRoutes.find((route) => route.key === item.id);
          const label = route?.title ?? item.id;
          return (
            <div key={item.id} className="group relative">
              <Button
                className={[
                  "flex h-10 w-full items-center rounded-md text-sm font-semibold transition",
                  isCollapsed ? "justify-center px-0" : "gap-3 px-3 text-left",
                  isActive ? "bg-white text-slate-900" : "text-slate-300 hover:bg-white/10 hover:text-white",
                ].join(" ")}
                type="button"
                title={isCollapsed ? undefined : label}
                onClick={() => onMenuChange(item.id)}
              >
                <Icon className="h-4 w-4 shrink-0" />
                {!isCollapsed && <span>{label}</span>}
              </Button>
              {isCollapsed && (
                <span className="pointer-events-none absolute left-[calc(100%+8px)] top-1/2 z-50 -translate-y-1/2 whitespace-nowrap rounded-md bg-slate-950 px-2.5 py-1.5 text-xs font-semibold text-white opacity-0 shadow-lg transition-opacity group-hover:opacity-100">
                  {label}
                </span>
              )}
            </div>
          );
        })}
      </nav>

      <div className={["border-t border-white/10 text-sm text-slate-300", isCollapsed ? "p-2" : "p-4"].join(" ")}>
        <div className="group relative">
          <Button
            className={[
              "flex h-10 w-full items-center rounded-md text-sm font-semibold transition",
              isCollapsed ? "justify-center px-0" : "gap-3 px-3 text-left",
              activeMenu === "settings" ? "bg-white text-slate-900" : "text-slate-300 hover:bg-white/10 hover:text-white",
            ].join(" ")}
            type="button"
            title={isCollapsed ? undefined : settingsLabel}
            onClick={() => onMenuChange("settings")}
          >
            <Settings className="h-4 w-4 shrink-0" />
            {!isCollapsed && <span>{settingsLabel}</span>}
          </Button>
          {isCollapsed && (
            <span className="pointer-events-none absolute left-[calc(100%+12px)] top-1/2 z-50 -translate-y-1/2 whitespace-nowrap rounded-md bg-slate-950 px-2.5 py-1.5 text-xs font-semibold text-white opacity-0 shadow-lg transition-opacity group-hover:opacity-100">
              {settingsLabel}
            </span>
          )}
        </div>
      </div>
    </aside>
  );
}
