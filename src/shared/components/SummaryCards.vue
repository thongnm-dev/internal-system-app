<script setup lang="ts">
import type { SummaryMetric } from "@/shared/types/statistics";

defineProps<{
  metrics: SummaryMetric[];
}>();
</script>

<template>
  <div class="grid grid-cols-4 gap-3">
    <div v-for="metric in metrics" :key="metric.label" class="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
      <span class="text-sm font-bold text-slate-500">{{ metric.label }}</span>
      <strong class="mt-2 block text-2xl leading-tight text-slate-900">{{ metric.value }}</strong>
    </div>
  </div>
</template>
