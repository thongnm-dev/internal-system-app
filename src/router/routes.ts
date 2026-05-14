import type { MenuKey } from "../types/statistics";

export type AppRoute = {
  key: MenuKey;
  path: string;
  title: string;
  subtitle: string;
};

export const appRoutes: AppRoute[] = [
  {
    key: "overview",
    path: "/overview",
    title: "Overview",
    subtitle: "Project and phase summary for the selected work data.",
  },
  {
    key: "projects",
    path: "/projects",
    title: "Projects",
    subtitle: "Detailed project breakdown grouped by development phase.",
  },
  {
    key: "phases",
    path: "/phases",
    title: "Phases",
    subtitle: "Phase-level total hours across projects.",
  },
  {
    key: "importCsv",
    path: "/import-csv",
    title: "Import CSV",
    subtitle: "Import exported system CSV data for monthly report checking.",
  },
  {
    key: "settings",
    path: "/settings",
    title: "Settings",
    subtitle: "Manage user profile, display preferences, language, and linked API keys.",
  },
];

export const defaultRoute = appRoutes[0];

export function routeByKey(key: MenuKey) {
  return appRoutes.find((route) => route.key === key) ?? defaultRoute;
}

export function routeByPath(path: string) {
  return appRoutes.find((route) => route.path === path) ?? defaultRoute;
}
