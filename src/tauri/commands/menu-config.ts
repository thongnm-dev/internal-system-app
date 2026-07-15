import { safeInvoke } from "./_base";
import type { MenuConfig, SaveAllMenuConfigsRequest, SaveMenuConfigRequest } from "@/_/types/menu-config";

export function listMenuConfigs() {
  return safeInvoke<MenuConfig[]>("list_menu_configs");
}

export function saveMenuConfig(request: SaveMenuConfigRequest) {
  return safeInvoke<void>("save_menu_config", { request });
}

export function saveAllMenuConfigs(request: SaveAllMenuConfigsRequest) {
  return safeInvoke<MenuConfig[]>("save_all_menu_configs", { request });
}
