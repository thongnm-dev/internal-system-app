import { safeInvoke } from "./_base";
import type {
  AddAiAccountRequest,
  AiAccount,
  AiUsageSettings,
  CapturedLogin,
  DetectedLogin,
  ReportUsageSignalRequest,
  UpdateAiAccountRequest,
} from "@/_/types/ai-usage";

export function aiUsageAddAccount(request: AddAiAccountRequest) {
  return safeInvoke<AiAccount>("ai_usage_add_account", { request });
}

export function aiUsageDetectLocal() {
  return safeInvoke<DetectedLogin[]>("ai_usage_detect_local");
}

export function aiUsageImportDetected() {
  return safeInvoke<AiAccount[]>("ai_usage_import_detected");
}

export function aiUsageCapturePreview() {
  return safeInvoke<CapturedLogin | null>("ai_usage_capture_preview");
}

export function aiUsageCaptureAdd(name?: string) {
  return safeInvoke<AiAccount>("ai_usage_capture_add", { name: name ?? null });
}

export function aiUsageConfigDirPreview(configDir: string) {
  return safeInvoke<CapturedLogin | null>("ai_usage_config_dir_preview", { configDir });
}

export function aiUsageAddConfigDir(configDir: string, name?: string) {
  return safeInvoke<AiAccount>("ai_usage_add_config_dir", { configDir, name: name ?? null });
}

export function aiUsageListAccounts() {
  return safeInvoke<AiAccount[]>("ai_usage_list_accounts");
}

export function aiUsageUpdateAccount(request: UpdateAiAccountRequest) {
  return safeInvoke<AiAccount>("ai_usage_update_account", { request });
}

export function aiUsageDeleteAccount(id: number) {
  return safeInvoke<void>("ai_usage_delete_account", { id });
}

export function aiUsageSetActive(id: number) {
  return safeInvoke<void>("ai_usage_set_active", { id });
}

export function aiUsageGetToken(id: number) {
  return safeInvoke<string>("ai_usage_get_token", { id });
}

export function aiUsageReportSignal(request: ReportUsageSignalRequest) {
  return safeInvoke<void>("ai_usage_report_signal", { request });
}

export function aiUsageRefresh() {
  return safeInvoke<AiAccount[]>("ai_usage_refresh");
}

export function aiUsageRefreshAccount(id: number) {
  return safeInvoke<AiAccount>("ai_usage_refresh_account", { id });
}

export function aiUsageGetSettings() {
  return safeInvoke<AiUsageSettings>("ai_usage_get_settings");
}

export function aiUsageSaveSettings(settings: AiUsageSettings) {
  return safeInvoke<void>("ai_usage_save_settings", { settings });
}

export function aiUsageOpenTerminal(configDir: string, workDir: string, prompt?: string) {
  return safeInvoke<void>("ai_usage_open_terminal", { configDir, workDir, prompt: prompt ?? null });
}

export function aiUsageOpenLogin(configDir: string, workDir: string) {
  return safeInvoke<void>("ai_usage_open_login", { configDir, workDir });
}
