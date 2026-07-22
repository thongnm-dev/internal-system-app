import { safeInvoke } from "./_base";
import type {
  AppConfigData,
  SaveAppConfigRequest,
  SpInfo,
  SpExecutionResult,
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

export function getStoredProcedureContent(name: string) {
  return safeInvoke<string>("get_stored_procedure_content", { name });
}

export function executeSingleStoredProcedure(name: string) {
  return safeInvoke<SpExecutionResult>("execute_single_stored_procedure", { name });
}

export function executeStoredProcedures() {
  return safeInvoke<SpExecutionSummary>("execute_stored_procedures");
}
