import type { RouteRecordRaw } from "vue-router";
import type { AppRouteKey, MenuKey } from "@/_/types/app";

export type AppRoute = {
  key: AppRouteKey;
  path: string;
  requiresAuth?: boolean;
  title: string;
  subtitle: string;
  breadcrumbs?: string[];
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
    key: "excel2md",
    path: "/excel2md",
    title: "Excel to Markdown",
    subtitle: "Convert uploaded Excel screen specs into Markdown files.",
  },
  {
    key: "copyTools",
    path: "/copy-tools",
    title: "Copy Tools",
    subtitle: "Collect and copy source documents by keyword or folder list.",
  },
  {
    key: "sqlEditor",
    path: "/sql-editor",
    title: "SQL Editor",
    subtitle: "Write and run SQL queries against the internal database.",
  },
  {
    key: "exploreFaster",
    path: "/explore-faster",
    title: "Explore Faster",
    subtitle: "Fast file browser — navigate folders and search documents quickly.",
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
    key: "checkMonthlyReport",
    path: "/check-monthly-report",
    title: "Check Monthly Report",
    subtitle: "Import exported system CSV data for monthly report checking.",
  },
  {
    key: "cloudS3",
    path: "/cloud/s3",
    title: "S3 Browser",
    subtitle: "Browse, upload, and manage files in S3-compatible storage buckets.",
  },
  {
    key: "cloudS3Upload",
    path: "/cloud/s3-upload",
    title: "S3 Upload",
    subtitle: "Upload files and folders to S3 storage destinations.",
  },
  {
    key: "cloudS3Download",
    path: "/cloud/s3-download",
    title: "S3 Download",
    subtitle: "Download files from S3 storage to local directories.",
  },
  {
    key: "cloudS3DownloadHistory",
    path: "/cloud/s3-download-history",
    title: "S3 Download History",
    subtitle: "Search and review S3 download history records.",
  },
  {
    key: "cloudS3UploadHistory",
    path: "/cloud/s3-upload-history",
    title: "S3 Upload History",
    subtitle: "Search and review S3 upload history records.",
  },
  {
    key: "aiChat",
    path: "/ai/chat",
    title: "AI Chat",
    subtitle: "Chat with AI assistants for code review, documentation, and task automation.",
  },
  {
    key: "aiUsage",
    path: "/ai/usage",
    title: "AI Usage",
    subtitle: "Track AI usage, token consumption, and cost statistics across the team.",
  },
  {
    key: "aiWorkflow",
    path: "/ai/workflow",
    title: "AI Workflow",
    subtitle: "Manage step-by-step AI workflows for Claude-based feature development.",
  },
  {
    key: "aiCowork",
    path: "/ai/cowork",
    title: "AI Cowork",
    subtitle: "Pick a project directory, apply an AI workflow, and choose an AI account to work with.",
  },
  {
    key: "aiTranslateCowork",
    path: "/ai/translate-cowork",
    title: "AI Translate Cowork",
    subtitle: "Pick a project directory, prepare input files, and run translation skills with an AI account.",
  },
  {
    key: "aiTasks",
    path: "/ai/tasks",
    title: "AI Tasks",
    subtitle: "Manage task codes and categories used by AI Cowork sessions.",
  },
  {
    key: "governanceMenus",
    path: "/governance/menus",
    requiresAuth: true,
    title: "Menus",
    subtitle: "Configure sidebar menu items, groups, visibility, and display order.",
  },
  {
    key: "governanceUsers",
    path: "/governance/users",
    requiresAuth: true,
    title: "Users",
    subtitle: "Manage system users, roles, and access permissions.",
  },
  {
    key: "governanceRoles",
    path: "/governance/roles",
    requiresAuth: true,
    title: "Roles",
    subtitle: "Manage system roles and their descriptions.",
  },
  {
    key: "governancePermissions",
    path: "/governance/permissions",
    requiresAuth: true,
    title: "Permissions",
    subtitle: "Grant or revoke sidebar menu access per role and per user.",
  },
  {
    key: "governanceLogs",
    path: "/governance/logs",
    requiresAuth: true,
    title: "Logs",
    subtitle: "View system activity logs and audit trails.",
  },
  {
    key: "governanceAppConfig",
    path: "/governance/app-config",
    requiresAuth: true,
    title: "App Config",
    subtitle: "View and edit application configuration (config.ini).",
  },
  {
    key: "governanceStoreProcedure",
    path: "/governance/store-procedure",
    requiresAuth: true,
    title: "Store Procedure",
    subtitle: "Manage and execute CREATE OR REPLACE for all stored procedures.",
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
  {
    key: "forgotPassword",
    path: "/forgot-password",
    title: "Forgot Password",
    subtitle: "Request a password reset link via email.",
  },
];

export const defaultRoute = appRoutes[0];
export const loginRoute = appRoutes.find((r) => r.key === "login")!;

export function routeByKey(key: MenuKey): AppRoute {
  return appRoutes.find((r) => r.key === key) ?? defaultRoute;
}

export function routeByPath(path: string): AppRoute {
  const pathname = path.split("?")[0];

  if (pathname.startsWith("/projects/")) {
    const base = routeByKey("projects");
    if (/^\/projects\/[^/]+\/tasks\/new$/.test(pathname)) {
      return { ...base, title: "Import Tasks", subtitle: "Import tasks from CSV or Backlog into the project.", breadcrumbs: ["Projects", "Import Tasks"] };
    }
    if (/^\/projects\/[^/]+\/tasks$/.test(pathname)) {
      return { ...base, title: "Project Tasks", subtitle: "Manage tasks assigned to this project.", breadcrumbs: ["Projects", "Tasks"] };
    }
    if (/^\/projects\/[^/]+\/report$/.test(pathname)) {
      return { ...base, title: "Project Report", subtitle: "View monthly report for this project.", breadcrumbs: ["Projects", "Report"] };
    }
    if (pathname === "/projects/new") {
      return { ...base, title: "New Project", subtitle: "Create a new project.", breadcrumbs: ["Projects", "New"] };
    }
    if (/^\/projects\/[^/]+$/.test(pathname)) {
      return { ...base, title: "Project Detail", subtitle: "View and edit project information.", breadcrumbs: ["Projects", "Detail"] };
    }
    return base;
  }

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
    component: () => import("@/features/governance/components/ProjectSkillsPage.vue"),
    meta: { key: "projectSkills" as MenuKey },
  },
  {
    path: "/issue-backlog",
    component: () => import("@/features/issues/components/IssueBacklogPage.vue"),
    meta: { key: "issueBacklog" as MenuKey },
  },
  {
    path: "/excel2md",
    component: () => import("@/features/tools/components/Excel2mdPage.vue"),
    meta: { key: "excel2md" as MenuKey },
  },
  {
    path: "/copy-tools",
    component: () => import("@/features/tools/components/CopyToolsPage.vue"),
    meta: { key: "copyTools" as MenuKey },
  },
  {
    path: "/sql-editor",
    component: () => import("@/features/tools/components/SqlEditorPage.vue"),
    meta: { key: "sqlEditor" as MenuKey },
  },
  {
    path: "/explore-faster",
    component: () => import("@/features/tools/components/ExploreFasterPage.vue"),
    meta: { key: "exploreFaster" as MenuKey },
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
    path: "/check-monthly-report",
    component: () => import("@/features/tools/components/CheckMonthlyReportPage.vue"),
    meta: { key: "checkMonthlyReport" as MenuKey },
  },
  {
    path: "/cloud/s3",
    component: () => import("@/features/cloud/components/S3BrowserPage.vue"),
    meta: { key: "cloudS3" as MenuKey },
  },
  {
    path: "/cloud/s3-upload",
    component: () => import("@/features/cloud/components/S3UploadPage.vue"),
    meta: { key: "cloudS3Upload" as MenuKey },
  },
  {
    path: "/cloud/s3-download",
    component: () => import("@/features/cloud/components/S3DownloadPage.vue"),
    meta: { key: "cloudS3Download" as MenuKey },
  },
  {
    path: "/cloud/s3-download-history",
    component: () => import("@/features/cloud/components/S3DownloadHistoryPage.vue"),
    meta: { key: "cloudS3DownloadHistory" as MenuKey },
  },
  {
    path: "/cloud/s3-upload-history",
    component: () => import("@/features/cloud/components/S3UploadHistoryPage.vue"),
    meta: { key: "cloudS3UploadHistory" as MenuKey },
  },
  {
    path: "/ai/chat",
    component: () => import("@/features/ai-agent/components/AiChatPage.vue"),
    meta: { key: "aiChat" as MenuKey },
  },
  {
    path: "/ai/usage",
    component: () => import("@/features/ai-agent/components/AiUsagePage.vue"),
    meta: { key: "aiUsage" as MenuKey },
  },
  {
    path: "/ai/workflow",
    component: () => import("@/features/ai-agent/components/AiWorkflowPage.vue"),
    meta: { key: "aiWorkflow" as MenuKey },
  },
  {
    path: "/ai/cowork",
    component: () => import("@/features/ai-agent/components/AiCoworkPage.vue"),
    meta: { key: "aiCowork" as MenuKey },
  },
  {
    path: "/ai/translate-cowork",
    component: () => import("@/features/ai-agent/components/AiTranslateCoworkPage.vue"),
    meta: { key: "aiTranslateCowork" as MenuKey },
  },
  {
    path: "/ai/tasks",
    component: () => import("@/features/ai-agent/components/AiTasksPage.vue"),
    meta: { key: "aiTasks" as MenuKey },
  },
  {
    path: "/governance/menus",
    component: () => import("@/features/governance/components/GovernanceMenusPage.vue"),
    meta: { key: "governanceMenus" as MenuKey, requiresAuth: true },
  },
  {
    path: "/governance/users",
    component: () => import("@/features/governance/components/GovernanceUsersPage.vue"),
    meta: { key: "governanceUsers" as MenuKey, requiresAuth: true },
  },
  {
    path: "/governance/roles",
    component: () => import("@/features/governance/components/GovernanceRolesPage.vue"),
    meta: { key: "governanceRoles" as MenuKey, requiresAuth: true },
  },
  {
    path: "/governance/permissions",
    component: () => import("@/features/governance/components/GovernancePermissionsPage.vue"),
    meta: { key: "governancePermissions" as MenuKey, requiresAuth: true },
  },
  {
    path: "/governance/logs",
    component: () => import("@/features/governance/components/GovernanceLogsPage.vue"),
    meta: { key: "governanceLogs" as MenuKey, requiresAuth: true },
  },
  {
    path: "/governance/app-config",
    component: () => import("@/features/governance/components/AppConfigPage.vue"),
    meta: { key: "governanceAppConfig" as MenuKey, requiresAuth: true },
  },
  {
    path: "/governance/store-procedure",
    component: () => import("@/features/governance/components/StoreProcedurePage.vue"),
    meta: { key: "governanceStoreProcedure" as MenuKey, requiresAuth: true },
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
  {
    path: "/forgot-password",
    component: () => import("@/features/auth/components/ForgotPasswordPage.vue"),
    meta: { key: "forgotPassword" as AppRouteKey },
  },
];
