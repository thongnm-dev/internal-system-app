import { safeInvoke } from "./_base";
import type { AppSettings, SaveSettingsRequest } from "@/_/types/settings";

export function getSettings(userId: number) {
  return safeInvoke<AppSettings>("get_settings", { userId });
}

export function saveSettings(request: SaveSettingsRequest) {
  return safeInvoke<AppSettings>("save_settings", { request });
}
