import { ref, computed } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import {
  listUsers,
  listRoles,
  createUser,
  updateUser,
  deleteUser,
  changeUserPassword,
} from "@/tauri/commands/user";
import type { UserSummary, CreateUserRequest, UpdateUserRequest } from "@/_/types/user";

export type UserStatus = "active" | "inactive";

interface UserFilters {
  username: string;
  fullName: string;
  email: string;
  phone: string;
  role: string;
  status: string;
}

const defaultFilters = (): UserFilters => ({
  username: "",
  fullName: "",
  email: "",
  phone: "",
  role: "All",
  status: "All",
});

export function useGovernanceUsers() {
  const globalLoading = useGlobalLoading();
  const users = ref<UserSummary[]>([]);
  const availableRoles = ref<string[]>([]);
  const loading = ref(false);
  const error = ref("");

  const draft = ref<{
    id: number;
    username: string;
    password: string;
    fullName: string;
    email: string;
    phone: string;
    address: string;
    position: string;
    isActive: boolean;
    roles: string[];
  } | null>(null);

  const isCreating = ref(false);
  const filters = ref<UserFilters>(defaultFilters());

  const roles = computed(() => {
    const set = new Set(users.value.flatMap((u) => u.roles));
    return ["All", ...Array.from(set).sort()];
  });

  const statuses = computed(() => ["All", "active", "inactive"]);

  const filteredUsers = computed(() => {
    let list = [...users.value];
    const f = filters.value;

    if (f.role !== "All") {
      list = list.filter((u) => u.roles.includes(f.role));
    }
    if (f.status !== "All") {
      const isActive = f.status === "active";
      list = list.filter((u) => u.is_active === isActive);
    }
    if (f.username) {
      const q = f.username.toLowerCase();
      list = list.filter((u) => u.username.toLowerCase().includes(q));
    }
    if (f.fullName) {
      const q = f.fullName.toLowerCase();
      list = list.filter((u) => u.full_name.toLowerCase().includes(q));
    }
    if (f.email) {
      const q = f.email.toLowerCase();
      list = list.filter((u) => u.email.toLowerCase().includes(q));
    }
    if (f.phone) {
      const q = f.phone.toLowerCase();
      list = list.filter((u) => u.phone.toLowerCase().includes(q));
    }

    return list.sort((a, b) => a.id - b.id);
  });

  function resetFilters() {
    filters.value = defaultFilters();
  }

  function search() {
    // filtering is reactive via filteredUsers computed — this is a no-op trigger
    // but kept as an explicit action point for future server-side search
  }

  async function loadUsers() {
    if (!canUseTauriRuntime()) return;
    loading.value = true;
    error.value = "";
    try {
      users.value = await listUsers();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadRoles() {
    if (!canUseTauriRuntime()) return;
    try {
      availableRoles.value = await listRoles();
    } catch {
      availableRoles.value = ["admin", "manager", "member", "viewer"];
    }
  }

  function startCreate() {
    isCreating.value = true;
    draft.value = {
      id: 0,
      username: "",
      password: "",
      fullName: "",
      email: "",
      phone: "",
      address: "",
      position: "",
      isActive: true,
      roles: ["member"],
    };
  }

  function selectUser(id: number) {
    isCreating.value = false;
    const user = users.value.find((u) => u.id === id);
    if (user) {
      draft.value = {
        id: user.id,
        username: user.username,
        password: "",
        fullName: user.full_name,
        email: user.email,
        phone: user.phone,
        address: "",
        position: user.position,
        isActive: user.is_active,
        roles: [...user.roles],
      };
    }
  }

  function updateDraft<K extends keyof NonNullable<typeof draft.value>>(
    field: K,
    value: NonNullable<typeof draft.value>[K],
  ) {
    if (draft.value) (draft.value as Record<string, unknown>)[field] = value;
  }

  function toggleDraftRole(role: string) {
    if (!draft.value) return;
    const idx = draft.value.roles.indexOf(role);
    if (idx >= 0) {
      if (draft.value.roles.length > 1) draft.value.roles.splice(idx, 1);
    } else {
      draft.value.roles.push(role);
    }
  }

  async function saveDraft(): Promise<boolean> {
    if (!draft.value) return false;
    if (!draft.value.username.trim() || !draft.value.fullName.trim()) return false;

    error.value = "";
    globalLoading.start();
    try {
      if (isCreating.value) {
        if (!draft.value.password.trim()) {
          error.value = "Password is required for new users.";
          return false;
        }
        const request: CreateUserRequest = {
          username: draft.value.username,
          password: draft.value.password,
          full_name: draft.value.fullName,
          email: draft.value.email || undefined,
          phone: draft.value.phone || undefined,
          address: draft.value.address || undefined,
          position: draft.value.position || undefined,
          roles: draft.value.roles,
        };
        await createUser(request);
      } else {
        const request: UpdateUserRequest = {
          full_name: draft.value.fullName,
          email: draft.value.email || undefined,
          phone: draft.value.phone || undefined,
          address: draft.value.address || undefined,
          position: draft.value.position || undefined,
          is_active: draft.value.isActive,
          roles: draft.value.roles,
        };
        await updateUser(draft.value.id, request);
      }
      await loadUsers();
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      globalLoading.stop();
    }
  }

  async function removeUser(id: number): Promise<boolean> {
    error.value = "";
    try {
      await deleteUser(id);
      await loadUsers();
      if (draft.value?.id === id) draft.value = null;
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  async function resetPassword(userId: number, newPassword: string): Promise<boolean> {
    error.value = "";
    try {
      await changeUserPassword(userId, { new_password: newPassword });
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  function init() {
    globalLoading.run(() => Promise.all([loadUsers(), loadRoles()]));
  }

  return {
    users,
    filteredUsers,
    availableRoles,
    loading,
    error,
    draft,
    isCreating,
    filters,
    roles,
    statuses,
    startCreate,
    selectUser,
    updateDraft,
    toggleDraftRole,
    saveDraft,
    removeUser,
    resetPassword,
    resetFilters,
    search,
    loadUsers,
    init,
  };
}
