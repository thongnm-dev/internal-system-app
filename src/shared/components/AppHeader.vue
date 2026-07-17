<script setup lang="ts">
import type { AppRoute } from "@/app/router/routes";
import Button from "primevue/button";
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
      <Button
        v-if="canGoBack"
        icon="pi pi-arrow-left"
        :label="backTitle ? `Back to ${backTitle}` : 'Back'"
        severity="secondary"
        outlined
        size="small"
        :title="backTitle ? `Back to ${backTitle}` : 'Back'"
        class="mt-3"
        @click="goBack"
      />
    </div>
    <div class="flex shrink-0 items-center gap-2">
      <Button
        :icon="settings.theme === 'dark' ? 'pi pi-sun' : 'pi pi-moon'"
        severity="secondary"
        outlined
        rounded
        :title="settings.theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'"
        @click="toggleTheme"
      />
      <Button
        v-if="username"
        icon="pi pi-sign-out"
        label="Logout"
        severity="secondary"
        outlined
        size="small"
        title="Logout"
        @click="emit('logout')"
      />
    </div>
  </header>
</template>
