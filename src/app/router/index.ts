import { createRouter, createWebHashHistory } from "vue-router";
import { vueRoutes } from "./routes";
import { useAuthStore } from "@/app/stores/auth";
import { useMenuStore } from "@/app/stores/menu";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import type { MenuKey } from "@/_/types/app";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: vueRoutes,
});

const PUBLIC_PATHS = ["/login", "/forgot-password"];
const ACCESS_FALLBACK = "/overview";

router.beforeEach((to) => {
  const auth = useAuthStore();

  if (!PUBLIC_PATHS.includes(to.path) && !auth.isAuthenticated) {
    auth.setReturnPath(to.fullPath);
    return { path: "/login" };
  }

  if (to.path === "/login" && auth.isAuthenticated) {
    return { path: auth.returnPath ?? ACCESS_FALLBACK };
  }

  // Block navigation to a menu the user isn't allowed to reach. The fallback
  // itself is always let through to avoid a redirect loop when even it is denied.
  const menuKey = to.meta.key as MenuKey | undefined;
  if (
    !PUBLIC_PATHS.includes(to.path) &&
    to.path !== ACCESS_FALLBACK &&
    auth.isAuthenticated &&
    menuKey &&
    !useMenuStore().canAccess(menuKey)
  ) {
    return { path: ACCESS_FALLBACK };
  }

  useGlobalLoading().start();
});

router.afterEach(() => {
  useGlobalLoading().stop();
});
