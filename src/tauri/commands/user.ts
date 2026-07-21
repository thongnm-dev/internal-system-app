import { safeInvoke } from "./_base";
import type {
  ChangePasswordRequest,
  CreateUserRequest,
  UpdateUserRequest,
  UserDetail,
  UserSummary,
} from "@/_/types/user";

export function createUser(request: CreateUserRequest) {
  return safeInvoke<UserDetail>("create_user", { request });
}

export function updateUser(userId: number, request: UpdateUserRequest) {
  return safeInvoke<UserDetail>("update_user", { userId, request });
}

export function getUserDetail(userId: number) {
  return safeInvoke<UserDetail>("get_user_detail", { userId });
}

export function listUsers() {
  return safeInvoke<UserSummary[]>("list_users");
}

export function deleteUser(userId: number) {
  return safeInvoke<void>("delete_user", { userId });
}

export function changeUserPassword(userId: number, request: ChangePasswordRequest) {
  return safeInvoke<void>("change_user_password", { userId, request });
}

export function listRoles() {
  return safeInvoke<string[]>("list_roles");
}

export function getStaffNo(username: string) {
  return safeInvoke<string>("get_staff_no", { username });
}
