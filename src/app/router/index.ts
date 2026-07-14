import { createRouter, createWebHashHistory } from "vue-router";
import { vueRoutes } from "./routes";
import { useAuthStore } from "@/app/stores/auth";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: vueRoutes,
});

const PUBLIC_PATHS = ["/login", "/forgot-password"];

router.beforeEach((to) => {
  const auth = useAuthStore();

  if (!PUBLIC_PATHS.includes(to.path) && !auth.isAuthenticated) {
    auth.setReturnPath(to.fullPath);
    return { path: "/login" };
  }

  if (to.path === "/login" && auth.isAuthenticated) {
    return { path: auth.returnPath ?? "/overview" };
  }
});
