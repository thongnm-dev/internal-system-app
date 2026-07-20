export type MenuKey =
  | "overview"
  | "projects"
  | "projectSkills"
  | "issueBacklog"
  | "importIssues"
  | "excel2md"
  | "sqlEditor"
  | "exploreFaster"
  | "dailyWorkNotes"
  | "dailyReport"
  | "checkMonthlyReport"
  | "cloudS3"
  | "cloudS3Upload"
  | "cloudS3Download"
  | "aiChat"
  | "aiUsage"
  | "governanceMenus"
  | "governanceUsers"
  | "governanceRoles"
  | "governancePermissions"
  | "governanceLogs"
  | "copyTools"
  | "settings";

export type AppRouteKey = MenuKey | "login" | "forgotPassword";

export type MessageMode = "info" | "error";

export type SummaryMetric = {
  label: string;
  value: string;
};
