<script setup lang="ts">
import { computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppShell } from "@/shared/composables/useAppShell";
import { useNetworkStatus } from "@/shared/composables/useNetworkStatus";
import { useDatabaseStatus } from "@/shared/composables/useDatabaseStatus";
import { installNavigationHistory, markMenuNavigation } from "@/shared/composables/useNavigationHistory";
import { useAuthStore } from "@/app/stores/auth";
import { appRoutes, loginRoute, routeByPath } from "@/app/router/routes";
import type { MenuKey } from "@/_/types/app";
import StartupScreen from "@/shared/components/StartupScreen.vue";
import ConnectionErrorScreen from "@/shared/components/ConnectionErrorScreen.vue";
import DatabaseErrorScreen from "@/shared/components/DatabaseErrorScreen.vue";
import DatabaseConfigScreen from "@/shared/components/DatabaseConfigScreen.vue";
import NetworkStatusBanner from "@/shared/components/NetworkStatusBanner.vue";
import AppSidebar from "@/shared/components/AppSidebar.vue";
import AppHeader from "@/shared/components/AppHeader.vue";
import AppBottomBar from "@/shared/components/AppBottomBar.vue";
import AppToast from "@/shared/components/AppToast.vue";
import GlobalLoading from "@/shared/components/GlobalLoading.vue";

const router = useRouter();
const route = useRoute();
const auth = useAuthStore();
const shell = useAppShell();
const network = useNetworkStatus();
const database = useDatabaseStatus();

installNavigationHistory(router);

const activeMenu = computed<MenuKey>(() => (route.meta.key as MenuKey) ?? "overview");

const currentAppRoute = computed(() => {
  return routeByPath(route.path);
});

function handleMenuChange(key: MenuKey) {
  const target = appRoutes.find((r) => r.key === key);
  if (target) {
    markMenuNavigation();
    router.push(target.path);
  }
}

function handleLogout() {
  auth.logout();
  router.push(loginRoute.path);
}

const AUTH_PAGES = ["/login", "/forgot-password"];

const isAuthPage = computed(() => AUTH_PAGES.includes(route.path));

watch(
  () => route.path,
  () => {
    if (!isAuthPage.value && !auth.isAuthenticated) {
      auth.setReturnPath(route.fullPath);
      router.push(loginRoute.path);
    }
  },
);
</script>

<template>
  <StartupScreen v-if="shell.isBootstrapping.value" />

  <ConnectionErrorScreen
    v-else-if="!network.hasConnectedOnce.value && !network.isOnline.value"
    :is-checking="network.isChecking.value"
    @retry="network.retry()"
  />

  <DatabaseErrorScreen
    v-else-if="
      database.hasChecked.value &&
      database.isConfigured.value &&
      !database.isConnected.value &&
      !database.wantsReconfigure.value
    "
  />

  <DatabaseConfigScreen
    v-else-if="database.hasChecked.value && !database.isConnected.value"
  />

  <template v-else-if="isAuthPage">
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

    <div
      v-if="!network.isOnline.value"
      class="pointer-events-none fixed inset-x-0 top-3 z-50 flex justify-center px-4"
    >
      <NetworkStatusBanner
        class="w-full max-w-2xl"
        :is-checking="network.isChecking.value"
        @retry="network.retry()"
      />
    </div>
  </main>

  <GlobalLoading />
  <AppToast />
</template>
