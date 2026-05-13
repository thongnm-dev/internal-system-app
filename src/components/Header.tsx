import type { MenuKey } from "../types/statistics";

type HeaderProps = {
  activeMenu: MenuKey;
};

const screenTitles: Record<MenuKey, { title: string; subtitle: string }> = {
  overview: {
    title: "Overview",
    subtitle: "Project and phase summary for the selected work data.",
  },
  projects: {
    title: "Projects",
    subtitle: "Detailed project breakdown grouped by development phase.",
  },
  phases: {
    title: "Phases",
    subtitle: "Phase-level total hours across projects.",
  },
  importCsv: {
    title: "Import CSV",
    subtitle: "Import exported system CSV data for monthly report checking.",
  },
};

export function Header({ activeMenu }: HeaderProps) {
  const screen = screenTitles[activeMenu];

  return (
    <header>
      <h2 className="text-2xl font-bold leading-tight">{screen.title}</h2>
      <p className="mt-2 text-sm text-slate-600">{screen.subtitle}</p>
      <nav className="mt-3 flex items-center gap-2 text-xs font-semibold text-slate-500" aria-label="Breadcrumb">
        <span>Home</span>
        <span className="text-slate-300">/</span>
        <span className="text-brand">{screen.title}</span>
      </nav>
    </header>
  );
}
