import { computed, ref } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { listMenuConfigs } from "@/tauri/commands/menu-config";
import { listRoleDetails } from "@/tauri/commands/role";
import { listUsers } from "@/tauri/commands/user";
import {
  listEffectiveMenuPermissions,
  listRoleMenuPermissions,
  listUserMenuPermissions,
  saveRoleMenuPermissions,
  saveUserMenuPermissions,
} from "@/tauri/commands/menu-permission";
import type { MenuConfig } from "@/_/types/menu-config";
import type { RoleSummary } from "@/_/types/role";
import type { UserMenuAccess } from "@/_/types/menu-permission";
import type { UserSummary } from "@/_/types/user";

export type PermissionTab = "roles" | "users";

/** Một nhóm menu hiển thị trên bảng phân quyền. */
export type MenuGroup = {
  name: string;
  items: MenuConfig[];
};

export function useMenuPermissions() {
  const globalLoading = useGlobalLoading();

  const tab = ref<PermissionTab>("roles");
  const menus = ref<MenuConfig[]>([]);
  const roles = ref<RoleSummary[]>([]);
  const users = ref<UserSummary[]>([]);

  const selectedRoleId = ref<number | null>(null);
  const selectedUserId = ref<number | null>(null);

  const searchQuery = ref("");
  const loading = ref(false);
  const saving = ref(false);
  const error = ref("");
  const savedMessage = ref("");

  // Tab Roles — danh sách menu key được cấp cho role đang chọn.
  const roleGrants = ref<string[]>([]);
  const savedRoleGrants = ref<string[]>([]);

  // Tab Users — trạng thái từng menu + quyền suy ra từ role để hiển thị.
  const userAccess = ref<Record<string, UserMenuAccess>>({});
  const savedUserAccess = ref<Record<string, UserMenuAccess>>({});
  const roleBaseline = ref<Record<string, boolean>>({});

  const selectedRole = computed(
    () => roles.value.find((r) => r.id === selectedRoleId.value) ?? null,
  );
  const selectedUser = computed(
    () => users.value.find((u) => u.id === selectedUserId.value) ?? null,
  );

  const hasSelection = computed(() =>
    tab.value === "roles" ? selectedRoleId.value !== null : selectedUserId.value !== null,
  );

  /** Menu đã lọc theo ô tìm kiếm, gom theo group và giữ thứ tự hiển thị. */
  const menuGroups = computed<MenuGroup[]>(() => {
    const q = searchQuery.value.trim().toLowerCase();
    const list = [...menus.value]
      .filter(
        (m) =>
          !q ||
          m.title.toLowerCase().includes(q) ||
          m.key.toLowerCase().includes(q) ||
          m.path.toLowerCase().includes(q),
      )
      .sort((a, b) => a.order - b.order);

    const grouped = new Map<string, MenuConfig[]>();
    for (const item of list) {
      const bucket = grouped.get(item.group);
      if (bucket) bucket.push(item);
      else grouped.set(item.group, [item]);
    }

    return Array.from(grouped, ([name, items]) => ({ name, items }));
  });

  const visibleMenuKeys = computed(() => menuGroups.value.flatMap((g) => g.items.map((i) => i.key)));

  const dirty = computed(() => {
    if (tab.value === "roles") {
      if (roleGrants.value.length !== savedRoleGrants.value.length) return true;
      return roleGrants.value.some((k) => !savedRoleGrants.value.includes(k));
    }
    return menus.value.some((m) => accessOf(m.key) !== (savedUserAccess.value[m.key] ?? "inherit"));
  });

  /** Số menu đang được cấp quyền trên lựa chọn hiện tại. */
  const grantedCount = computed(() => {
    if (tab.value === "roles") return roleGrants.value.length;
    return menus.value.filter((m) => effectiveAllowed(m.key)).length;
  });

  function accessOf(key: string): UserMenuAccess {
    return userAccess.value[key] ?? "inherit";
  }

  /** Quyền cuối cùng của user trên một menu sau khi áp override. */
  function effectiveAllowed(key: string): boolean {
    const access = accessOf(key);
    if (access === "allow") return true;
    if (access === "deny") return false;
    return roleBaseline.value[key] ?? false;
  }

  function inheritedAllowed(key: string): boolean {
    return roleBaseline.value[key] ?? false;
  }

  function isGranted(key: string): boolean {
    return roleGrants.value.includes(key);
  }

  function clearFeedback() {
    error.value = "";
    savedMessage.value = "";
  }

  function toggleRoleGrant(key: string) {
    clearFeedback();
    const idx = roleGrants.value.indexOf(key);
    if (idx >= 0) roleGrants.value.splice(idx, 1);
    else roleGrants.value.push(key);
  }

  function setUserAccess(key: string, access: UserMenuAccess) {
    clearFeedback();
    if (access === "inherit") delete userAccess.value[key];
    else userAccess.value[key] = access;
  }

  /** Tick/bỏ tick toàn bộ menu trong một group (chỉ áp cho tab Roles). */
  function toggleGroup(group: MenuGroup, granted: boolean) {
    clearFeedback();
    const keys = group.items.map((i) => i.key);
    if (granted) {
      for (const key of keys) {
        if (!roleGrants.value.includes(key)) roleGrants.value.push(key);
      }
    } else {
      roleGrants.value = roleGrants.value.filter((k) => !keys.includes(k));
    }
  }

  function groupState(group: MenuGroup): "all" | "some" | "none" {
    const keys = group.items.map((i) => i.key);
    const on = keys.filter((k) => isGranted(k)).length;
    if (on === 0) return "none";
    return on === keys.length ? "all" : "some";
  }

  function selectAll() {
    clearFeedback();
    if (tab.value === "roles") {
      roleGrants.value = Array.from(new Set([...roleGrants.value, ...visibleMenuKeys.value]));
    } else {
      for (const key of visibleMenuKeys.value) userAccess.value[key] = "allow";
    }
  }

  function clearAll() {
    clearFeedback();
    if (tab.value === "roles") {
      roleGrants.value = roleGrants.value.filter((k) => !visibleMenuKeys.value.includes(k));
    } else {
      for (const key of visibleMenuKeys.value) userAccess.value[key] = "deny";
    }
  }

  /** Đưa toàn bộ menu đang hiển thị về kế thừa quyền role (chỉ tab Users). */
  function resetToInherit() {
    clearFeedback();
    for (const key of visibleMenuKeys.value) delete userAccess.value[key];
  }

  function revert() {
    clearFeedback();
    if (tab.value === "roles") {
      roleGrants.value = [...savedRoleGrants.value];
    } else {
      userAccess.value = { ...savedUserAccess.value };
    }
  }

  async function loadRolePermissions(roleId: number) {
    loading.value = true;
    clearFeedback();
    try {
      const keys = await listRoleMenuPermissions(roleId);
      roleGrants.value = [...keys];
      savedRoleGrants.value = [...keys];
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadUserPermissions(userId: number) {
    loading.value = true;
    clearFeedback();
    try {
      const [overrides, effective] = await Promise.all([
        listUserMenuPermissions(userId),
        listEffectiveMenuPermissions(userId),
      ]);

      const baseline: Record<string, boolean> = {};
      for (const row of effective) baseline[row.menu_key] = row.role_allowed;
      roleBaseline.value = baseline;

      const access: Record<string, UserMenuAccess> = {};
      for (const row of overrides) access[row.menu_key] = row.is_allowed ? "allow" : "deny";
      userAccess.value = { ...access };
      savedUserAccess.value = { ...access };
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function selectRole(roleId: number) {
    selectedRoleId.value = roleId;
    await loadRolePermissions(roleId);
  }

  async function selectUser(userId: number) {
    selectedUserId.value = userId;
    await loadUserPermissions(userId);
  }

  async function switchTab(next: PermissionTab) {
    if (tab.value === next) return;
    tab.value = next;
    clearFeedback();
    searchQuery.value = "";

    if (next === "roles" && selectedRoleId.value === null && roles.value.length > 0) {
      await selectRole(roles.value[0].id);
    }
    if (next === "users" && selectedUserId.value === null && users.value.length > 0) {
      await selectUser(users.value[0].id);
    }
  }

  async function save(): Promise<boolean> {
    clearFeedback();
    saving.value = true;
    try {
      if (tab.value === "roles") {
        if (selectedRoleId.value === null) return false;
        const keys = await saveRoleMenuPermissions({
          role_id: selectedRoleId.value,
          menu_keys: roleGrants.value,
        });
        roleGrants.value = [...keys];
        savedRoleGrants.value = [...keys];
        savedMessage.value = `Đã lưu quyền menu cho role "${selectedRole.value?.name ?? ""}".`;
      } else {
        if (selectedUserId.value === null) return false;
        const allow_keys: string[] = [];
        const deny_keys: string[] = [];
        for (const [key, access] of Object.entries(userAccess.value)) {
          if (access === "allow") allow_keys.push(key);
          else if (access === "deny") deny_keys.push(key);
        }

        const overrides = await saveUserMenuPermissions({
          user_id: selectedUserId.value,
          allow_keys,
          deny_keys,
        });

        const access: Record<string, UserMenuAccess> = {};
        for (const row of overrides) access[row.menu_key] = row.is_allowed ? "allow" : "deny";
        userAccess.value = { ...access };
        savedUserAccess.value = { ...access };
        savedMessage.value = `Đã lưu quyền menu cho user "${selectedUser.value?.username ?? ""}".`;
      }
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      saving.value = false;
    }
  }

  async function init() {
    if (!canUseTauriRuntime()) return;
    await globalLoading.run(async () => {
      try {
        const [menuList, roleList, userList] = await Promise.all([
          listMenuConfigs(),
          listRoleDetails(),
          listUsers(),
        ]);
        menus.value = menuList;
        roles.value = roleList;
        users.value = userList;

        if (roleList.length > 0) await selectRole(roleList[0].id);
      } catch (e) {
        error.value = String(e);
      }
    });
  }

  return {
    tab,
    menus,
    roles,
    users,
    selectedRoleId,
    selectedUserId,
    selectedRole,
    selectedUser,
    hasSelection,
    menuGroups,
    searchQuery,
    loading,
    saving,
    error,
    savedMessage,
    dirty,
    grantedCount,
    accessOf,
    effectiveAllowed,
    inheritedAllowed,
    isGranted,
    toggleRoleGrant,
    setUserAccess,
    toggleGroup,
    groupState,
    selectAll,
    clearAll,
    resetToInherit,
    revert,
    selectRole,
    selectUser,
    switchTab,
    save,
    init,
  };
}
