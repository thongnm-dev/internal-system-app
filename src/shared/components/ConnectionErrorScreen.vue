<script setup lang="ts">
defineProps<{
  isChecking: boolean;
}>();

const emit = defineEmits<{
  (event: "retry"): void;
}>();
</script>

<template>
  <main
    class="flex h-screen min-h-[640px] min-w-[900px] items-center justify-center overflow-hidden bg-canvas text-ink"
  >
    <section
      aria-live="assertive"
      aria-label="Loi ket noi mang"
      class="flex max-w-md select-none flex-col items-center gap-6 px-8 text-center"
      role="alert"
    >
      <span
        aria-hidden="true"
        class="flex h-20 w-20 items-center justify-center rounded-full bg-red-50 text-red-600"
      >
        <svg
          class="h-10 w-10"
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
      </span>

      <div class="flex flex-col gap-2">
        <h1 class="text-lg font-semibold text-ink">Không có kết nối mạng</h1>
        <p class="text-sm text-secondary">
          Ứng dụng cần kết nối internet để hoạt động. Vui lòng kiểm tra kết nối
          mạng của bạn rồi thử lại.
        </p>
      </div>

      <button
        type="button"
        class="inline-flex items-center gap-2 rounded-md bg-brand px-5 py-2.5 text-sm font-semibold text-white shadow-card transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="isChecking"
        @click="emit('retry')"
      >
        <svg
          v-if="isChecking"
          class="h-4 w-4 animate-spin"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" d="M12 3a9 9 0 1 0 9 9" />
        </svg>
        {{ isChecking ? "Đang kiểm tra..." : "Thử lại" }}
      </button>
    </section>
  </main>
</template>
