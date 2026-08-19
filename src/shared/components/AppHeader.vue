<script setup lang="ts">
import type { AppRoute } from "@/app/router/routes";
import Button from "primevue/button";
import { useNavigationHistory } from "@/shared/composables/useNavigationHistory";
import { useTabNavigation } from "@/shared/composables/useTabNavigation";

defineProps<{
  route: AppRoute;
}>();

const { canGoBack, backTitle, goBack } = useNavigationHistory();
const { tabMode } = useTabNavigation();
</script>

<template>
  <header>
    <div>
      <h2 class="text-xl font-bold leading-tight">{{ route.title }}</h2>
      <nav class="mt-2 flex items-center gap-2 text-xs font-semibold text-muted" aria-label="Breadcrumb">
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
        v-if="canGoBack && !tabMode"
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
  </header>
</template>
