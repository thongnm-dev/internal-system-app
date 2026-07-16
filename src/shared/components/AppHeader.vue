<script setup lang="ts">
import type { AppRoute } from "@/app/router/routes";
import { useSettings } from "@/features/settings/composables/useSettings";
import { useNavigationHistory } from "@/shared/composables/useNavigationHistory";

defineProps<{
  route: AppRoute;
  username?: string;
}>();

const emit = defineEmits<{
  logout: [];
}>();

const { settings, updateTheme } = useSettings();
const { canGoBack, backTitle, goBack } = useNavigationHistory();

function toggleTheme() {
  updateTheme(settings.value.theme === "dark" ? "light" : "dark");
}
</script>

<template>
  <header class="flex items-start justify-between gap-4">
    <div>
      <h2 class="text-2xl font-bold leading-tight">{{ route.title }}</h2>
      <p class="mt-2 text-sm text-secondary">{{ route.subtitle }}</p>
      <nav class="mt-3 flex items-center gap-2 text-xs font-semibold text-muted" aria-label="Breadcrumb">
        <span>Home</span>
        <template v-if="route.breadcrumbs?.length">
          <template v-for="(crumb, i) in route.breadcrumbs" :key="i">
            <span class="text-divider">/</span>
            <span :class="i === route.breadcrumbs.length - 1 ? 'text-brand' : ''">{{ crumb }}</span>
          </template>
        </template>
        <template v-else>
          <span class="text-divider">/</span>
          <span class="text-brand">{{ route.title }}</span>
        </template>
      </nav>
      <button
        v-if="canGoBack"
        class="mt-3 flex h-8 items-center gap-2 rounded-md border border-divider bg-panel px-3 text-xs font-bold text-secondary hover:bg-sidebar-hover"
        type="button"
        :title="backTitle ? `Back to ${backTitle}` : 'Back'"
        @click="goBack"
      >
        <i class="pi pi-arrow-left" />
        <span class="min-w-0 truncate">{{ backTitle ? `Back to ${backTitle}` : "Back" }}</span>
      </button>
    </div>
    <div class="flex shrink-0 items-center gap-2">
      <button
        class="flex h-9 w-9 items-center justify-center rounded-full border border-divider bg-panel text-secondary hover:bg-sidebar-hover"
        type="button"
        :title="settings.theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'"
        @click="toggleTheme"
      >
        <i :class="settings.theme === 'dark' ? 'pi pi-sun' : 'pi pi-moon'" />
      </button>
      <button
        v-if="username"
        class="flex h-9 shrink-0 items-center justify-center gap-2 rounded-md border border-divider bg-panel px-3 text-sm font-bold text-secondary hover:bg-sidebar-hover"
        type="button"
        title="Logout"
        @click="emit('logout')"
      >
        <i class="pi pi-sign-out" />
        Logout
      </button>
    </div>
  </header>
</template>
