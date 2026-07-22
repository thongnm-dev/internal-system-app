import { safeInvoke } from "./_base";
import type {
  AppConfigData,
  SaveAppConfigRequest,
  SpInfo,
  SpExecutionSummary,
} from "@/_/types/app-config";

export function getAppConfig() {
  return safeInvoke<AppConfigData>("get_app_config");
}

export function saveAppConfig(request: SaveAppConfigRequest) {
  return safeInvoke<AppConfigData>("save_app_config", { request });
}

export function listStoredProcedures() {
  return safeInvoke<SpInfo[]>("list_stored_procedures");
}

export function executeStoredProcedures() {
  return safeInvoke<SpExecutionSummary>("execute_stored_procedures");
}
