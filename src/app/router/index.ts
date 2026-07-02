import { createRouter, createWebHashHistory } from "vue-router";
import { vueRoutes } from "./routes";
import { useAuthStore } from "@/app/stores/auth";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: vueRoutes,
});

router.beforeEach((to) => {
  const auth = useAuthStore();

  if (to.meta.requiresAuth && !auth.isAuthenticated) {
    auth.setReturnPath(to.fullPath);
    return { path: "/login" };
  }

  if (to.path === "/login" && auth.isAuthenticated) {
    return { path: auth.returnPath ?? "/overview" };
  }
});
