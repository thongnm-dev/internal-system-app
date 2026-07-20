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
        class="config-error-icon flex h-20 w-20 items-center justify-center rounded-full"
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

      <Button
        :icon="isChecking ? 'pi pi-spinner pi-spin' : undefined"
        :label="isChecking ? 'Đang kiểm tra...' : 'Thử lại'"
        :disabled="isChecking"
        @click="emit('retry')"
      />
    </section>
  </main>
</template>

<style scoped>
.config-error-icon {
  background: #fef2f2;
  color: #dc2626;
}
[data-theme='dark'] .config-error-icon {
  background: #450a0a;
  color: #f87171;
}
</style>
