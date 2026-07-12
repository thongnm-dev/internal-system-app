import type { RouteRecordRaw } from "vue-router";
import type { AppRouteKey, MenuKey } from "@/shared/types/app";

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
    key: "projectSkills",
    path: "/project-skills",
    title: "Skills",
    subtitle: "Manage the internal skill catalog by skill name, category, and usage.",
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
    key: "excel2md",
    path: "/excel2md",
    title: "Excel to Markdown",
    subtitle: "Convert uploaded Excel screen specs into Markdown files.",
  },
  {
    key: "dailyWorkNotes",
    path: "/daily-work-notes",
    title: "Daily Work Notes",
    subtitle: "Track completed, incomplete, and reserved work notes by date.",
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
export const loginRoute = appRoutes.find((r) => r.key === "login")!;

export function routeByKey(key: MenuKey): AppRoute {
  return appRoutes.find((r) => r.key === key) ?? defaultRoute;
}

export function routeByPath(path: string): AppRoute {
  const pathname = path.split("?")[0];

  if (pathname.startsWith("/projects/")) return routeByKey("projects");

  return appRoutes.find((r) => r.path === pathname) ?? defaultRoute;
}

export const vueRoutes: RouteRecordRaw[] = [
  { path: "/", redirect: "/overview" },
  {
    path: "/overview",
    component: () => import("@/features/overview/components/OverviewPage.vue"),
    meta: { key: "overview" as MenuKey },
  },
  {
    path: "/projects",
    component: () => import("@/features/projects/components/ProjectsPage.vue"),
    meta: { key: "projects" as MenuKey },
  },
  {
    path: "/projects/new",
    component: () => import("@/features/projects/components/ProjectDetailPage.vue"),
    meta: { key: "projects" as MenuKey },
  },
  {
    path: "/projects/:id",
    component: () => import("@/features/projects/components/ProjectDetailPage.vue"),
    meta: { key: "projects" as MenuKey },
  },
  {
    path: "/projects/:id/tasks",
    component: () => import("@/features/projects/components/ProjectTasksPage.vue"),
    meta: { key: "projects" as MenuKey },
  },
  {
    path: "/projects/:id/tasks/new",
    component: () => import("@/features/projects/components/ProjectAddTaskPage.vue"),
    meta: { key: "projects" as MenuKey },
  },
  {
    path: "/projects/:id/report",
    component: () => import("@/features/projects/components/ProjectReportPage.vue"),
    meta: { key: "projects" as MenuKey },
  },
  {
    path: "/project-skills",
    component: () => import("@/features/skills/components/ProjectSkillsPage.vue"),
    meta: { key: "projectSkills" as MenuKey },
  },
  {
    path: "/issue-backlog",
    component: () => import("@/features/issues/components/IssueBacklogPage.vue"),
    meta: { key: "issueBacklog" as MenuKey },
  },
  {
    path: "/import-issues",
    component: () => import("@/features/issues/components/ImportIssuesPage.vue"),
    meta: { key: "importIssues" as MenuKey },
  },
  {
    path: "/excel2md",
    component: () => import("@/features/excel2md/components/excel2mdPage.vue"),
    meta: { key: "excel2md" as MenuKey },
  },
  {
    path: "/daily-work-notes",
    component: () => import("@/features/daily-notes/components/DailyWorkNotesPage.vue"),
    meta: { key: "dailyWorkNotes" as MenuKey },
  },
  {
    path: "/daily-report",
    component: () => import("@/features/daily-report/components/DailyReportPage.vue"),
    meta: { key: "dailyReport" as MenuKey },
  },
  {
    path: "/import-csv",
    component: () => import("@/features/import-csv/components/ImportCsvPage.vue"),
    meta: { key: "importCsv" as MenuKey },
  },
  {
    path: "/settings",
    component: () => import("@/features/settings/components/SettingsPage.vue"),
    meta: { key: "settings" as MenuKey, requiresAuth: true },
  },
  {
    path: "/login",
    component: () => import("@/features/auth/components/LoginPage.vue"),
    meta: { key: "login" as AppRouteKey },
  },
];
