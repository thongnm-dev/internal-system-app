<script setup lang="ts">
import { useDatabaseStatus } from "@/shared/composables/useDatabaseStatus";

const database = useDatabaseStatus();
</script>

<template>
  <main
    class="flex h-screen min-h-[640px] min-w-[900px] items-center justify-center overflow-hidden bg-canvas text-ink"
  >
    <section
      aria-live="assertive"
      aria-label="Loi ket noi database"
      class="flex max-w-md select-none flex-col items-center gap-6 px-8 text-center"
      role="alert"
    >
      <span
        aria-hidden="true"
        class="flex h-20 w-20 items-center justify-center rounded-full bg-red-50 text-red-600"
      >
        <i class="pi pi-database text-3xl" />
      </span>

      <div class="flex flex-col gap-2">
        <h1 class="text-lg font-semibold text-ink">Không kết nối được tới database</h1>
        <p class="text-sm text-secondary">
          Ứng dụng đã được cấu hình nhưng không thể kết nối tới database. Vui lòng
          kiểm tra database có đang chạy không, hoặc cập nhật lại thông tin cấu hình.
        </p>
        <p
          v-if="database.statusMessage.value"
          class="mt-1 break-words rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700"
        >
          {{ database.statusMessage.value }}
        </p>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="inline-flex items-center gap-2 rounded-md border border-divider bg-panel px-5 py-2.5 text-sm font-semibold text-secondary transition-colors hover:bg-canvas disabled:cursor-not-allowed disabled:opacity-60"
          :disabled="database.isChecking.value"
          @click="database.requestReconfigure()"
        >
          <i class="pi pi-cog" />
          Cấu hình lại
        </button>

        <button
          type="button"
          class="inline-flex items-center gap-2 rounded-md bg-brand px-5 py-2.5 text-sm font-semibold text-white shadow-card transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
          :disabled="database.isChecking.value"
          @click="database.check()"
        >
          <svg
            v-if="database.isChecking.value"
            class="h-4 w-4 animate-spin"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" d="M12 3a9 9 0 1 0 9 9" />
          </svg>
          {{ database.isChecking.value ? "Đang kiểm tra..." : "Thử lại" }}
        </button>
      </div>
    </section>
  </main>
</template>
