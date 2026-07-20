import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { listMenuConfigs } from "@/tauri/commands/menu-config";
import { listEffectiveMenuPermissions } from "@/tauri/commands/menu-permission";
import type { MenuConfig } from "@/_/types/menu-config";

/** Menu key dùng cho các mục top-level không thuộc nhóm nào. */
const UNGROUPED = "—";
/** Key của mục Settings — luôn render riêng ở chân sidebar. */
const SETTINGS_KEY = "settings";

/** Một mục menu đã sẵn sàng để render trên sidebar. */
export type MenuItem = {
  key: string;
  title: string;
  path: string;
  icon: string;
  group: string;
  order: number;
};

/** Một nhóm menu (menu_group) cùng danh sách mục con đã lọc quyền. */
export type MenuGroup = {
  label: string;
  items: MenuItem[];
};

/**
 * Nguồn menu của client. Sau khi login thành công, `load(userId)` nạp
 * menu_configs + quyền hiệu lực từ backend rồi giữ trong store; sidebar và
 * router guard đọc lại từ đây thay vì hardcode.
 */
export const useMenuStore = defineStore("menu", () => {
  const configs = ref<MenuConfig[]>([]);
  const allowedKeys = ref<Set<string>>(new Set());
  const loaded = ref(false);

  /** Toàn bộ key có trong menu_configs (được governance quản lý). */
  const managedKeys = computed(() => new Set(configs.value.map((m) => m.key)));

  const toItem = (m: MenuConfig): MenuItem => ({
    key: m.key,
    title: m.title,
    path: m.path,
    icon: m.icon,
    group: m.group,
    order: m.order,
  });

  /** Menu hiển thị được: đang bật `visible` và user có quyền, sắp theo `order`. */
  const accessibleMenus = computed<MenuItem[]>(() =>
    configs.value
      .filter((m) => m.visible && allowedKeys.value.has(m.key))
      .slice()
      .sort((a, b) => a.order - b.order)
      .map(toItem),
  );

  /** Mục top-level (không nhóm), trừ Settings vì render riêng ở chân sidebar. */
  const topLevelItems = computed(() =>
    accessibleMenus.value.filter((m) => m.group === UNGROUPED && m.key !== SETTINGS_KEY),
  );

  /** Các nhóm menu, giữ thứ tự xuất hiện theo `order` của mục đầu tiên. */
  const groups = computed<MenuGroup[]>(() => {
    const map = new Map<string, MenuItem[]>();
    for (const m of accessibleMenus.value) {
      if (m.group === UNGROUPED) continue;
      const bucket = map.get(m.group);
      if (bucket) bucket.push(m);
      else map.set(m.group, [m]);
    }
    return Array.from(map, ([label, items]) => ({ label, items }));
  });

  /** Mục Settings nếu user được phép truy cập (dùng cho chân sidebar). */
  const settingsMenu = computed(
    () => accessibleMenus.value.find((m) => m.key === SETTINGS_KEY) ?? null,
  );

  /**
   * User có được vào một menu không. Trước khi load hoặc với key không nằm
   * trong menu_configs (route con chưa được governance quản lý) → mặc định cho
   * phép, nên guard không chặn nhầm các trang phụ.
   */
  function canAccess(key: string): boolean {
    if (!loaded.value) return true;
    if (!managedKeys.value.has(key)) return true;
    return allowedKeys.value.has(key);
  }

  async function load(userId: number) {
    if (!canUseTauriRuntime()) {
      clear();
      return;
    }
    try {
      const [menuList, effective] = await Promise.all([
        listMenuConfigs(),
        listEffectiveMenuPermissions(userId),
      ]);
      configs.value = menuList;
      allowedKeys.value = new Set(
        effective.filter((r) => r.is_allowed).map((r) => r.menu_key),
      );
      loaded.value = true;
    } catch {
      // Fail open: để trống → không ẩn/chặn gì cho tới lần load thành công.
      clear();
    }
  }

  function clear() {
    configs.value = [];
    allowedKeys.value = new Set();
    loaded.value = false;
  }

  return {
    configs,
    allowedKeys,
    loaded,
    accessibleMenus,
    topLevelItems,
    groups,
    settingsMenu,
    canAccess,
    load,
    clear,
  };
});
