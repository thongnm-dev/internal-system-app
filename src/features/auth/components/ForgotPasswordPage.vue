<script setup lang="ts">
import { ref } from "vue";
import { RouterLink } from "vue-router";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { friendlyError } from "@/tauri/commands/_base";
import { requestPasswordReset, verifyPasswordReset } from "@/tauri/commands/auth";

type Step = "username" | "code" | "done";

const step = ref<Step>("username");
const username = ref("");
const code = ref("");
const maskedEmail = ref("");
const tempPassword = ref("");
const loading = ref(false);
const error = ref("");
async function submitUsername() {
  if (!username.value.trim()) {
    error.value = "Vui lòng nhập tên đăng nhập.";
    return;
  }

  error.value = "";
  loading.value = true;

  try {
    maskedEmail.value = await requestPasswordReset(username.value.trim());
    step.value = "code";
  } catch (e) {
    error.value = friendlyError(e);
  } finally {
    loading.value = false;
  }
}

async function submitCode() {
  if (!code.value.trim()) {
    error.value = "Vui lòng nhập mã xác nhận.";
    return;
  }

  error.value = "";
  loading.value = true;

  try {
    tempPassword.value = await verifyPasswordReset(username.value.trim(), code.value.trim());
    step.value = "done";
  } catch (e) {
    error.value = friendlyError(e);
  } finally {
    loading.value = false;
  }
}

async function resendCode() {
  error.value = "";
  loading.value = true;

  try {
    maskedEmail.value = await requestPasswordReset(username.value.trim());
    code.value = "";
    error.value = "";
  } catch (e) {
    error.value = friendlyError(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <main class="force-light grid min-h-screen place-items-center bg-canvas px-6 text-ink" data-theme="light">
    <section class="w-full max-w-[420px] rounded-lg border border-divider bg-panel p-6 shadow-sm">
      <div class="flex items-center gap-3">
        <div class="flex h-11 w-11 items-center justify-center rounded-md bg-brand text-white">
          <i class="pi pi-envelope text-xl" />
        </div>
        <div>
          <h1 class="text-xl font-bold leading-tight">Quên mật khẩu</h1>
          <p class="mt-1 text-sm text-muted">
            <template v-if="step === 'username'">Nhập tên đăng nhập để nhận mã xác nhận qua email.</template>
            <template v-else-if="step === 'code'">Nhập mã xác nhận đã gửi đến <strong>{{ maskedEmail }}</strong></template>
            <template v-else>Mật khẩu đã được đặt lại thành công.</template>
          </p>
        </div>
      </div>

      <!-- Step 1: Enter username -->
      <form v-if="step === 'username'" class="mt-6 space-y-4" @submit.prevent="submitUsername">
        <label class="block">
          <span class="text-xs font-bold text-muted">Username</span>
          <div class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 text-ink hover:border-brand focus-within:border-brand focus-within:ring-2 focus-within:ring-brand/20">
            <i class="pi pi-user shrink-0 text-muted" />
            <InputText
              v-model="username"
              class="h-full min-w-0 flex-1 border-0 bg-transparent text-sm shadow-none outline-none"
              autocomplete="username"
              autofocus
              placeholder="username"
            />
          </div>
        </label>

        <p
          v-if="error"
          class="rounded-md border border-red-500/20 bg-red-500/10 px-3 py-2 text-sm font-semibold text-red-500"
        >
          {{ error }}
        </p>

        <Button
          :disabled="loading"
          :icon="loading ? 'pi pi-spinner pi-spin' : 'pi pi-send'"
          :label="loading ? 'Đang gửi...' : 'Gửi mã xác nhận'"
          class="w-full"
          type="submit"
        />
      </form>

      <!-- Step 2: Enter verification code -->
      <form v-else-if="step === 'code'" class="mt-6 space-y-4" @submit.prevent="submitCode">
        <label class="block">
          <span class="text-xs font-bold text-muted">Mã xác nhận (6 chữ số)</span>
          <div class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 text-ink hover:border-brand focus-within:border-brand focus-within:ring-2 focus-within:ring-brand/20">
            <i class="pi pi-key shrink-0 text-muted" />
            <InputText
              v-model="code"
              class="h-full min-w-0 flex-1 border-0 bg-transparent text-sm tracking-[6px] font-bold shadow-none outline-none"
              maxlength="6"
              placeholder="000000"
              inputmode="numeric"
            />
          </div>
        </label>

        <p class="text-xs text-muted">
          Mã có hiệu lực trong 30 phút.
          <button type="button" class="text-brand hover:underline" :disabled="loading" @click="resendCode">
            Gửi lại mã
          </button>
        </p>

        <p
          v-if="error"
          class="rounded-md border border-red-500/20 bg-red-500/10 px-3 py-2 text-sm font-semibold text-red-500"
        >
          {{ error }}
        </p>

        <Button
          :disabled="loading"
          :icon="loading ? 'pi pi-spinner pi-spin' : 'pi pi-check'"
          :label="loading ? 'Đang xác nhận...' : 'Xác nhận'"
          class="w-full"
          type="submit"
        />
      </form>

      <!-- Step 3: Success -->
      <div v-else class="mt-6 space-y-3">
        <div class="rounded-md border border-green-500/20 bg-green-500/10 px-4 py-3 text-sm text-green-700">
          <p class="font-semibold">Mật khẩu đã được đặt lại thành công!</p>
          <p class="mt-1">Vui lòng đăng nhập bằng mật khẩu mới bên dưới và đổi lại mật khẩu sau khi đăng nhập.</p>
        </div>

        <div class="rounded-md border border-divider bg-canvas px-4 py-3">
          <span class="text-xs font-bold text-muted">Mật khẩu mới</span>
          <p class="mt-1 font-mono text-lg font-bold tracking-wider text-brand">{{ tempPassword }}</p>
        </div>
      </div>

      <p class="mt-4 text-center text-sm">
        <RouterLink to="/login" class="text-brand hover:underline">
          Quay lại đăng nhập
        </RouterLink>
      </p>
    </section>
  </main>
</template>

<style scoped>
:deep(.p-inputtext) {
  border: 0;
  background: transparent;
  box-shadow: none;
  outline: none;
  padding: 0;
  height: 100%;
  width: 100%;
}

:deep(.p-inputtext:enabled:focus),
:deep(.p-inputtext:focus) {
  border: 0;
  box-shadow: none;
  outline: none;
}
</style>
