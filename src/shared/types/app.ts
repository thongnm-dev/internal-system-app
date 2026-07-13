export type MenuKey =
  | "overview"
  | "projects"
  | "projectSkills"
  | "issueBacklog"
  | "importIssues"
  | "excel2md"
  | "dailyWorkNotes"
  | "dailyReport"
  | "importCsv"
  | "governanceMenus"
  | "governanceUsers"
  | "governanceLogs"
  | "settings";

export type AppRouteKey = MenuKey | "login";

export type MessageMode = "info" | "error";

export type SummaryMetric = {
  label: string;
  value: string;
};
