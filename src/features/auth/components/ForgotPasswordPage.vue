<script setup lang="ts">
import { ref } from "vue";
import { RouterLink } from "vue-router";
import Button from "primevue/button";
import InputText from "primevue/inputtext";

const email = ref("");
const submitted = ref(false);
const error = ref("");

function submitForgotPassword() {
  if (!email.value.trim()) {
    error.value = "Vui lòng nhập địa chỉ email.";
    return;
  }

  error.value = "";
  submitted.value = true;
}
</script>

<template>
  <main class="grid min-h-screen place-items-center bg-canvas px-6 text-ink">
    <section class="w-full max-w-[420px] rounded-lg border border-divider bg-panel p-6 shadow-sm">
      <div class="flex items-center gap-3">
        <div class="flex h-11 w-11 items-center justify-center rounded-md bg-brand text-white">
          <i class="pi pi-envelope text-xl" />
        </div>
        <div>
          <h1 class="text-xl font-bold leading-tight">Quên mật khẩu</h1>
          <p class="mt-1 text-sm text-muted">Nhập email để nhận hướng dẫn đặt lại mật khẩu.</p>
        </div>
      </div>

      <template v-if="!submitted">
        <form class="mt-6 space-y-4" @submit.prevent="submitForgotPassword">
          <label class="block">
            <span class="text-xs font-bold text-muted">Email</span>
            <div class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 text-ink hover:border-brand focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
              <i class="pi pi-envelope shrink-0 text-muted" />
              <InputText
                v-model="email"
                class="h-full min-w-0 flex-1 border-0 bg-transparent text-sm shadow-none outline-none"
                autocomplete="email"
                autofocus
                placeholder="your@email.com"
                type="email"
              />
            </div>
          </label>

          <p
            v-if="error"
            class="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm font-semibold text-red-700"
          >
            {{ error }}
          </p>

          <Button
            icon="pi pi-send"
            label="Gửi yêu cầu"
            class="w-full"
            type="submit"
          />
        </form>
      </template>

      <template v-else>
        <div class="mt-6 rounded-md border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          <p class="font-semibold">Yêu cầu đã được gửi!</p>
          <p class="mt-1">Vui lòng kiểm tra hộp thư email để nhận hướng dẫn đặt lại mật khẩu.</p>
        </div>
      </template>

      <p class="mt-4 text-center text-sm">
        <RouterLink to="/login" class="text-brand hover:underline">
          Quay lại đăng nhập
        </RouterLink>
      </p>
    </section>
  </main>
</template>
