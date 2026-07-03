<script setup lang="ts">
import { computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppShell } from "@/shared/composables/useAppShell";
import { useAuthStore } from "@/app/stores/auth";
import { appRoutes, defaultRoute, loginRoute } from "@/app/router/routes";
import type { MenuKey } from "@/shared/types/app";
import StartupScreen from "@/shared/components/StartupScreen.vue";
import AppSidebar from "@/shared/components/AppSidebar.vue";
import AppHeader from "@/shared/components/AppHeader.vue";
import AppBottomBar from "@/shared/components/AppBottomBar.vue";

const router = useRouter();
const route = useRoute();
const auth = useAuthStore();
const shell = useAppShell();

const activeMenu = computed<MenuKey>(() => (route.meta.key as MenuKey) ?? "overview");

const currentAppRoute = computed(() => {
  return appRoutes.find((r) => r.key === route.meta.key) ?? defaultRoute;
});

function handleMenuChange(key: MenuKey) {
  const target = appRoutes.find((r) => r.key === key);
  if (target) {
    router.push(target.path);
  }
}

function handleLogout() {
  auth.logout();
  router.push(loginRoute.path);
}

watch(
  () => route.path,
  () => {
    if (currentAppRoute.value.requiresAuth && !auth.isAuthenticated) {
      auth.setReturnPath(route.fullPath);
      router.push(loginRoute.path);
    }
  },
);
</script>

<template>
  <StartupScreen v-if="shell.isBootstrapping.value" />

  <template v-else-if="route.path === '/login'">
    <router-view />
  </template>

  <main v-else class="grid h-screen grid-rows-[minmax(0,1fr)_auto] overflow-hidden bg-canvas text-ink">
    <section
      :class="[
        'grid min-h-0 overflow-hidden transition-[grid-template-columns] duration-200',
        shell.isSidebarCollapsed.value
          ? 'grid-cols-[72px_minmax(0,1fr)]'
          : 'grid-cols-[240px_minmax(0,1fr)]',
      ]"
    >
      <AppSidebar
        :active-menu="activeMenu"
        :is-collapsed="shell.isSidebarCollapsed.value"
        @menu-change="handleMenuChange"
        @toggle-collapse="shell.toggleSidebar()"
      />

      <section class="min-h-0 overflow-hidden p-6">
        <div class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
          <AppHeader
            :route="currentAppRoute"
            :username="auth.user?.username"
            @logout="handleLogout"
          />

          <router-view />
        </div>
      </section>
    </section>

    <AppBottomBar :info="shell.systemInfo.value" />
  </main>
</template>
