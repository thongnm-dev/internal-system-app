<script setup lang="ts">
import type { MessageMode } from "@/_/types/app";

defineProps<{
  message: string;
  mode: MessageMode;
}>();
</script>

<template>
  <div
    :class="[
      'border-l-4 bg-panel px-4 py-2.5 text-sm shadow-card',
      mode === 'error' ? 'border-red-600 text-red-800' : 'border-brand text-secondary',
    ]"
  >
    {{ message }}
  </div>
</template>
