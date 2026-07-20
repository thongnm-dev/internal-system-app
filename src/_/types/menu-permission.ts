/** Override quyền menu ở cấp user — `is_allowed = false` là thu hồi quyền role đã cấp. */
export type UserMenuPermission = {
  menu_key: string;
  is_allowed: boolean;
};

/** Quyền menu của user sau khi gộp role + override riêng. */
export type EffectiveMenuPermission = {
  menu_key: string;
  is_allowed: boolean;
  /** Quyền suy ra từ các role của user, trước khi áp override riêng. */
  role_allowed: boolean;
  source: "user" | "role";
};

export type SaveRoleMenuPermissionsRequest = {
  role_id: number;
  menu_keys: string[];
};

export type SaveUserMenuPermissionsRequest = {
  user_id: number;
  allow_keys: string[];
  deny_keys: string[];
};

/** Trạng thái một menu trong tab User: theo role, cấp thêm, hoặc thu hồi. */
export type UserMenuAccess = "inherit" | "allow" | "deny";
