export type MenuKey =
  | "overview"
  | "projects"
  | "projectSkills"
  | "issueBacklog"
  | "excel2md"
  | "sqlEditor"
  | "exploreFaster"
  | "dailyWorkNotes"
  | "dailyReport"
  | "checkMonthlyReport"
  | "cloudS3"
  | "cloudS3Upload"
  | "cloudS3Download"
  | "cloudS3BugFolders"
  | "cloudS3DownloadHistory"
  | "cloudS3UploadHistory"
  | "aiChat"
  | "aiUsage"
  | "aiWorkflow"
  | "aiCowork"
  | "aiTranslateCowork"
  | "aiTasks"
  | "governanceMenus"
  | "governanceUsers"
  | "governanceRoles"
  | "governancePermissions"
  | "governanceLogs"
  | "governanceAppConfig"
  | "governanceStoreProcedure"
  | "copyTools"
  | "settings";

export type AppRouteKey = MenuKey | "login" | "forgotPassword";

export type MessageMode = "info" | "error";

export type SummaryMetric = {
  label: string;
  value: string;
};
