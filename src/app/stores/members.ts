import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { listUsers } from "@/tauri/commands/user";
import type { ProjectMember } from "@/_/types/project";

/**
 * Nguồn danh sách member (lấy từ users) dùng cho dialog chọn member khi tạo/sửa
 * project. Thay cho dữ liệu mock trước đây — `load()` nạp users đang active từ
 * backend rồi giữ trong store để các màn hình dùng lại mà không gọi lặp.
 */
export const useMembersStore = defineStore("members", () => {
  const members = ref<ProjectMember[]>([]);
  const loading = ref(false);
  const loaded = ref(false);
  const error = ref("");

  /** Số member hiện có (đã lọc active). */
  const count = computed(() => members.value.length);

  /**
   * Nạp danh sách member từ backend. Đã nạp thành công thì bỏ qua trừ khi
   * `force = true`. Chỉ lấy user đang active và sắp theo username.
   */
  async function load(force = false) {
    if (loading.value) return;
    if (loaded.value && !force) return;
    if (!canUseTauriRuntime()) return;
    loading.value = true;
    error.value = "";
    try {
      const users = await listUsers();
      members.value = users
        .filter((u) => u.is_active)
        .map((u) => ({ username: u.username, name: u.full_name }))
        .sort((a, b) => a.username.localeCompare(b.username));
      loaded.value = true;
    } catch (e) {
      error.value = friendlyError(e);
      members.value = [];
      loaded.value = false;
    } finally {
      loading.value = false;
    }
  }

  function clear() {
    members.value = [];
    loaded.value = false;
    error.value = "";
  }

  return {
    members,
    loading,
    loaded,
    error,
    count,
    load,
    clear,
  };
});
