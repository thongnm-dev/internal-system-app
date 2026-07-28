import { computed, ref } from "vue";
import type { Router } from "vue-router";
import { appRoutes } from "@/app/router/routes";

export type TabItem = {
  key: string;
  path: string;
  currentPath: string;
  title: string;
};

const TAB_MODE_KEY = "msh.app.tabMode";
const tabMode = ref(
  typeof window !== "undefined" &&
    window.localStorage.getItem(TAB_MODE_KEY) === "true",
);
const tabs = ref<TabItem[]>([]);
const activeTabKey = ref<string | null>(null);
let routerRef: Router | null = null;

export function useTabNavigation() {
  function init(router: Router) {
    routerRef = router;
  }

  function setTabMode(enabled: boolean) {
    tabMode.value = enabled;
    window.localStorage.setItem(TAB_MODE_KEY, String(enabled));
    if (!enabled) {
      tabs.value = [];
      activeTabKey.value = null;
    }
  }

  function openTab(key: string) {
    const route = appRoutes.find((r) => r.key === key);
    if (!route) return;

    const existing = tabs.value.find((t) => t.key === key);
    if (existing) {
      if (activeTabKey.value !== key) {
        activeTabKey.value = key;
        routerRef?.push(existing.currentPath);
      }
      return;
    }

    tabs.value.push({
      key,
      path: route.path,
      currentPath: route.path,
      title: route.title,
    });
    activeTabKey.value = key;
    routerRef?.push(route.path);
  }

  function closeTab(key: string) {
    const idx = tabs.value.findIndex((t) => t.key === key);
    if (idx === -1) return;

    const wasActive = activeTabKey.value === key;
    tabs.value.splice(idx, 1);

    if (wasActive) {
      if (tabs.value.length > 0) {
        const next = tabs.value[Math.min(idx, tabs.value.length - 1)];
        activeTabKey.value = next.key;
        routerRef?.push(next.currentPath);
      } else {
        activeTabKey.value = null;
        routerRef?.push("/overview");
      }
    }
  }

  function activateTab(key: string) {
    const tab = tabs.value.find((t) => t.key === key);
    if (!tab || activeTabKey.value === key) return;
    activeTabKey.value = key;
    routerRef?.push(tab.currentPath);
  }

  function syncRoute(menuKey: string, fullPath: string) {
    let tab = tabs.value.find((t) => t.key === menuKey);
    if (!tab) {
      const route = appRoutes.find((r) => r.key === menuKey);
      if (route) {
        tab = {
          key: menuKey,
          path: route.path,
          currentPath: fullPath,
          title: route.title,
        };
        tabs.value.push(tab);
      }
    }
    activeTabKey.value = menuKey;
    if (tab) tab.currentPath = fullPath;
  }

  return {
    tabMode: computed(() => tabMode.value),
    tabs: computed(() => tabs.value),
    activeTabKey: computed(() => activeTabKey.value),
    init,
    setTabMode,
    openTab,
    closeTab,
    activateTab,
    syncRoute,
  };
}
