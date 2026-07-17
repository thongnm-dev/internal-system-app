<script setup lang="ts">
import Button from "primevue/button";

defineProps<{
  isChecking: boolean;
}>();

const emit = defineEmits<{
  (event: "retry"): void;
}>();
</script>

<template>
  <div
    aria-live="assertive"
    role="alert"
    class="pointer-events-auto flex items-center gap-3 rounded-md border-l-4 border-red-600 bg-panel px-4 py-2.5 text-sm text-red-800 shadow-card"
  >
    <svg
      aria-hidden="true"
      class="h-5 w-5 shrink-0 text-red-600"
      fill="none"
      stroke="currentColor"
      stroke-width="1.6"
      viewBox="0 0 24 24"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        d="M8.5 16.5a5 5 0 0 1 7 0M5 13a10 10 0 0 1 14 0M2 9.5a15 15 0 0 1 20 0M12 20h.01"
      />
      <path stroke-linecap="round" stroke-linejoin="round" d="m3 3 18 18" />
    </svg>

    <span class="flex-1 font-medium">
      Mất kết nối mạng. Một số thao tác có thể không hoạt động cho đến khi kết
      nối được khôi phục.
    </span>

    <Button
      :icon="isChecking ? 'pi pi-spinner pi-spin' : undefined"
      :label="isChecking ? 'Đang thử...' : 'Thử lại'"
      severity="danger"
      outlined
      size="small"
      :disabled="isChecking"
      @click="emit('retry')"
    />
  </div>
</template>
