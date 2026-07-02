<script setup lang="ts">
import type { AppRoute } from "@/app/router/routes";

defineProps<{
  route: AppRoute;
  username?: string;
}>();

const emit = defineEmits<{
  logout: [];
}>();
</script>

<template>
  <header class="flex items-start justify-between gap-4">
    <div>
      <h2 class="text-2xl font-bold leading-tight">{{ route.title }}</h2>
      <p class="mt-2 text-sm text-slate-600">{{ route.subtitle }}</p>
      <nav class="mt-3 flex items-center gap-2 text-xs font-semibold text-slate-500" aria-label="Breadcrumb">
        <span>Home</span>
        <span class="text-slate-300">/</span>
        <span class="text-brand">{{ route.title }}</span>
      </nav>
    </div>
    <button
      v-if="username"
      class="flex h-9 shrink-0 items-center justify-center gap-2 rounded-md border border-slate-300 bg-panel px-3 text-sm font-bold text-slate-700 hover:bg-slate-50"
      type="button"
      title="Logout"
      @click="emit('logout')"
    >
      <i class="pi pi-sign-out" />
      Logout
    </button>
  </header>
</template>
