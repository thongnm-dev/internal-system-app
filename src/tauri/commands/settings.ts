import { safeInvoke } from "./_base";
import type { AppSettings, SaveSettingsRequest } from "@/_/types/settings";

export function getSettings() {
  return safeInvoke<AppSettings>("get_settings");
}

export function saveSettings(request: SaveSettingsRequest) {
  return safeInvoke<AppSettings>("save_settings", { request });
}
