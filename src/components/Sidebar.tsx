import { BarChart3, FileSpreadsheet, Home, Table2 } from "lucide-react";
import type { AnalysisResult, MenuKey } from "../types/statistics";

type SidebarProps = {
  activeMenu: MenuKey;
  onMenuChange: (value: MenuKey) => void;
  result: AnalysisResult | null;
};

export function Sidebar({ activeMenu, onMenuChange, result }: SidebarProps) {
  const items = [
    { id: "overview" as const, label: "Overview", icon: Home },
    { id: "projects" as const, label: "Projects", icon: Table2 },
    { id: "phases" as const, label: "Phases", icon: BarChart3 },
  ];

  return (
    <aside className="flex min-h-0 flex-col border-r border-slate-200 bg-slate-900 text-white">
      <div className="border-b border-white/10 p-5">
        <div className="flex h-10 w-10 items-center justify-center rounded-md bg-brand font-bold">PJ</div>
        <h1 className="mt-4 text-xl font-bold leading-tight">PJ Yuji Statistics</h1>
        <p className="mt-2 text-sm text-slate-300">Project and phase statistics.</p>
      </div>

      <nav className="flex-1 space-y-1 p-3">
        {items.map((item) => {
          const Icon = item.icon;
          const isActive = activeMenu === item.id;
          return (
            <button
              key={item.id}
              className={[
                "flex h-10 w-full items-center gap-3 rounded-md px-3 text-left text-sm font-semibold transition",
                isActive ? "bg-white text-slate-900" : "text-slate-300 hover:bg-white/10 hover:text-white",
              ].join(" ")}
              type="button"
              onClick={() => onMenuChange(item.id)}
            >
              <Icon className="h-4 w-4" />
              {item.label}
            </button>
          );
        })}
      </nav>

      <div className="border-t border-white/10 p-4 text-sm text-slate-300">
        <div className="flex items-center gap-2">
          <FileSpreadsheet className="h-4 w-4" />
          <span>{result ? `${result.projects.length} project` : "No data"}</span>
        </div>
      </div>
    </aside>
  );
}
