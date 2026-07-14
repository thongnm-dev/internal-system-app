import { safeInvoke } from "./_base";
import type { SystemInfo } from "@/shared/types/system";

export function getSystemInfo() {
  return safeInvoke<SystemInfo>("get_system_info");
}

export function checkInternetConnection() {
  return safeInvoke<boolean>("check_internet_connection");
}
