<script setup lang="ts">
import type { SummaryMetric } from "@/_/types/app";

defineProps<{
  metrics: SummaryMetric[];
}>();
</script>

<template>
  <div class="grid grid-cols-4 gap-3">
    <div v-for="metric in metrics" :key="metric.label" class="rounded-lg border border-divider bg-panel p-4 shadow-card">
      <span class="text-sm font-bold text-muted">{{ metric.label }}</span>
      <strong class="mt-2 block text-2xl leading-tight text-ink">{{ metric.value }}</strong>
    </div>
  </div>
</template>
