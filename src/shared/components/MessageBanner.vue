<script setup lang="ts">
import type { MessageMode } from "@/shared/types/app";

defineProps<{
  message: string;
  mode: MessageMode;
}>();
</script>

<template>
  <div
    :class="[
      'border-l-4 bg-white px-4 py-2.5 text-sm shadow-sm',
      mode === 'error' ? 'border-red-600 text-red-800' : 'border-brand text-slate-700',
    ]"
  >
    {{ message }}
  </div>
</template>
