import { computed, ref } from "vue";
import type { Router } from "vue-router";
import { loginRoute, routeByPath } from "@/app/router/routes";

type BackEntry = {
  path: string;
  title: string;
};

// Module-level state shared across the app shell and header.
const stack = ref<BackEntry[]>([]);
let routerRef: Router | null = null;
let navIntent: "menu" | "back" | null = null;
let installed = false;

/**
 * Flag the next navigation as a sidebar menu change. Menu navigation resets the
 * back stack so the back button only appears after cross-screen navigation.
 */
export function markMenuNavigation() {
  navIntent = "menu";
}

/**
 * Install the router guard that records cross-screen navigation. Call once from
 * the app shell with the active router instance.
 */
export function installNavigationHistory(router: Router) {
  if (installed) return;
  installed = true;
  routerRef = router;

  router.afterEach((to, from) => {
    const intent = navIntent;
    navIntent = null;

    if (intent === "menu") {
      stack.value = [];
      return;
    }
    if (intent === "back") {
      // Stack already popped by goBack().
      return;
    }

    const cameFromRealScreen =
      Boolean(from.meta?.key) &&
      from.path !== to.path &&
      from.path !== loginRoute.path &&
      to.path !== loginRoute.path;

    if (cameFromRealScreen) {
      stack.value = [...stack.value, { path: from.fullPath, title: routeByPath(from.path).title }];
    }
  });
}

export function useNavigationHistory() {
  const canGoBack = computed(() => stack.value.length > 0);
  const backTitle = computed(() => stack.value[stack.value.length - 1]?.title ?? "");

  function goBack() {
    const prev = stack.value[stack.value.length - 1];
    if (!prev || !routerRef) return;
    stack.value = stack.value.slice(0, -1);
    navIntent = "back";
    routerRef.push(prev.path);
  }

  return { canGoBack, backTitle, goBack };
}
