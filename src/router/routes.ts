import type { AppRouteKey, MenuKey } from "../types/statistics";

export type AppRoute = {
  key: AppRouteKey;
  path: string;
  requiresAuth?: boolean;
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
    key: "issueBacklog",
    path: "/issue-backlog",
    title: "Issue Backlog",
    subtitle: "Search, review, and manage project backlog issues.",
  },
  {
    key: "importIssues",
    path: "/import-issues",
    title: "Import Issues",
    subtitle: "Import issue CSV data into a selected project.",
  },
  {
    key: "dailyReport",
    path: "/daily-report",
    title: "Daily Report",
    subtitle: "Enter daily work hours for each assigned project task.",
  },
  {
    key: "importCsv",
    path: "/import-csv",
    title: "Import CSV",
    subtitle: "Import exported system CSV data for monthly report checking.",
  },
  {
    key: "importReports",
    path: "/import-reports",
    title: "Reports",
    subtitle: "Search and review monthly report import history.",
  },
  {
    key: "settings",
    path: "/settings",
    requiresAuth: true,
    title: "Settings",
    subtitle: "Manage user profile, display preferences, language, and linked API keys.",
  },
  {
    key: "login",
    path: "/login",
    title: "Login",
    subtitle: "Authenticate to continue to protected screens.",
  },
];

export const defaultRoute = appRoutes[0];
export const loginRoute = appRoutes.find((route) => route.key === "login")!;

export function routeByKey(key: MenuKey) {
  return appRoutes.find((route) => route.key === key) ?? defaultRoute;
}

export function routeByPath(path: string) {
  if (path.startsWith("/projects/")) {
    return routeByKey("projects");
  }

  if (path.startsWith("/import-reports/")) {
    return routeByKey("importReports");
  }

  return appRoutes.find((route) => route.path === path) ?? defaultRoute;
}
