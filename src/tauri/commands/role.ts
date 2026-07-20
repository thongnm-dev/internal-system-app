import { safeInvoke } from "./_base";
import type { CreateRoleRequest, RoleSummary, UpdateRoleRequest } from "@/_/types/role";

export function listRoleDetails() {
  return safeInvoke<RoleSummary[]>("list_role_details");
}

export function createRole(request: CreateRoleRequest) {
  return safeInvoke<RoleSummary>("create_role", { request });
}

export function updateRole(roleId: number, request: UpdateRoleRequest) {
  return safeInvoke<RoleSummary>("update_role", { roleId, request });
}

export function deleteRole(roleId: number) {
  return safeInvoke<void>("delete_role", { roleId });
}
