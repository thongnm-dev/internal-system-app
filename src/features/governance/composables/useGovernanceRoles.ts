import { ref, computed } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { listRoleDetails, createRole, updateRole, deleteRole } from "@/tauri/commands/role";
import type { RoleSummary, CreateRoleRequest, UpdateRoleRequest } from "@/_/types/role";

interface RoleFilters {
  name: string;
  description: string;
}

const defaultFilters = (): RoleFilters => ({
  name: "",
  description: "",
});

export function useGovernanceRoles() {
  const globalLoading = useGlobalLoading();
  const roles = ref<RoleSummary[]>([]);
  const loading = ref(false);
  const error = ref("");

  const draft = ref<{
    id: number;
    name: string;
    description: string;
  } | null>(null);

  const isCreating = ref(false);
  const filters = ref<RoleFilters>(defaultFilters());

  const filteredRoles = computed(() => {
    let list = [...roles.value];
    const f = filters.value;

    if (f.name) {
      const q = f.name.toLowerCase();
      list = list.filter((r) => r.name.toLowerCase().includes(q));
    }
    if (f.description) {
      const q = f.description.toLowerCase();
      list = list.filter((r) => r.description.toLowerCase().includes(q));
    }

    return list.sort((a, b) => a.id - b.id);
  });

  function resetFilters() {
    filters.value = defaultFilters();
  }

  function search() {
    // filtering is reactive via filteredRoles computed — this is a no-op trigger
    // but kept as an explicit action point for future server-side search
  }

  async function loadRoles() {
    if (!canUseTauriRuntime()) return;
    loading.value = true;
    error.value = "";
    try {
      roles.value = await listRoleDetails();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  function startCreate() {
    isCreating.value = true;
    draft.value = {
      id: 0,
      name: "",
      description: "",
    };
  }

  function selectRole(id: number) {
    isCreating.value = false;
    const role = roles.value.find((r) => r.id === id);
    if (role) {
      draft.value = {
        id: role.id,
        name: role.name,
        description: role.description,
      };
    }
  }

  function updateDraft<K extends keyof NonNullable<typeof draft.value>>(
    field: K,
    value: NonNullable<typeof draft.value>[K],
  ) {
    if (draft.value) (draft.value as Record<string, unknown>)[field] = value;
  }

  async function saveDraft(): Promise<boolean> {
    if (!draft.value) return false;
    if (!draft.value.name.trim()) {
      error.value = "Role name is required.";
      return false;
    }

    error.value = "";
    globalLoading.start();
    try {
      if (isCreating.value) {
        const request: CreateRoleRequest = {
          name: draft.value.name,
          description: draft.value.description || undefined,
        };
        await createRole(request);
      } else {
        const request: UpdateRoleRequest = {
          name: draft.value.name,
          description: draft.value.description || undefined,
        };
        await updateRole(draft.value.id, request);
      }
      await loadRoles();
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      globalLoading.stop();
    }
  }

  async function removeRole(id: number): Promise<boolean> {
    error.value = "";
    try {
      await deleteRole(id);
      await loadRoles();
      if (draft.value?.id === id) draft.value = null;
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  function init() {
    globalLoading.run(() => loadRoles());
  }

  return {
    roles,
    filteredRoles,
    loading,
    error,
    draft,
    isCreating,
    filters,
    startCreate,
    selectRole,
    updateDraft,
    saveDraft,
    removeRole,
    resetFilters,
    search,
    loadRoles,
    init,
  };
}
