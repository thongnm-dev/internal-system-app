<script setup lang="ts">
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

    <button
      type="button"
      class="inline-flex shrink-0 items-center gap-1.5 rounded border border-red-600 px-3 py-1 text-xs font-semibold text-red-700 transition-colors hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-60"
      :disabled="isChecking"
      @click="emit('retry')"
    >
      <svg
        v-if="isChecking"
        class="h-3.5 w-3.5 animate-spin"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" d="M12 3a9 9 0 1 0 9 9" />
      </svg>
      {{ isChecking ? "Đang thử..." : "Thử lại" }}
    </button>
  </div>
</template>
