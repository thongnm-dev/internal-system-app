<script setup lang="ts">
import Button from "primevue/button";
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
        class="flex h-20 w-20 items-center justify-center rounded-full bg-red-50 text-red-600 dark:bg-red-950 dark:text-red-400"
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
        <Button
          icon="pi pi-cog"
          label="Cấu hình lại"
          severity="secondary"
          outlined
          :disabled="database.isChecking.value"
          @click="database.requestReconfigure()"
        />

        <Button
          :icon="database.isChecking.value ? 'pi pi-spinner pi-spin' : undefined"
          :label="database.isChecking.value ? 'Đang kiểm tra...' : 'Thử lại'"
          :disabled="database.isChecking.value"
          @click="database.check()"
        />
      </div>
    </section>
  </main>
</template>
