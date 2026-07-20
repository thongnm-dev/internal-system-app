import { safeInvoke } from "./_base";
import type {
  EffectiveMenuPermission,
  SaveRoleMenuPermissionsRequest,
  SaveUserMenuPermissionsRequest,
  UserMenuPermission,
} from "@/_/types/menu-permission";

export function listRoleMenuPermissions(roleId: number) {
  return safeInvoke<string[]>("list_role_menu_permissions", { roleId });
}

export function saveRoleMenuPermissions(request: SaveRoleMenuPermissionsRequest) {
  return safeInvoke<string[]>("save_role_menu_permissions", { request });
}

export function listUserMenuPermissions(userId: number) {
  return safeInvoke<UserMenuPermission[]>("list_user_menu_permissions", { userId });
}

export function saveUserMenuPermissions(request: SaveUserMenuPermissionsRequest) {
  return safeInvoke<UserMenuPermission[]>("save_user_menu_permissions", { request });
}

export function listEffectiveMenuPermissions(userId: number) {
  return safeInvoke<EffectiveMenuPermission[]>("list_effective_menu_permissions", { userId });
}
