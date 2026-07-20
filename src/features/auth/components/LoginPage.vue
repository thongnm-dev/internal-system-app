<script setup lang="ts">
import { ref } from "vue";
import { RouterLink, useRouter } from "vue-router";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import InputText from "primevue/inputtext";
import Password from "primevue/password";
import { useAuthStore } from "@/app/stores/auth";
import { useMenuStore } from "@/app/stores/menu";
import { defaultRoute } from "@/app/router/routes";
import { friendlyError } from "@/tauri/commands/_base";
import { login as tauriLogin } from "@/tauri/commands/auth";

const router = useRouter();
const auth = useAuthStore();
const menu = useMenuStore();

const username = ref("");
const password = ref("");
const rememberMe = ref(false);
const loading = ref(false);
const error = ref("");

async function submitLogin() {
  if (!username.value.trim() || !password.value.trim()) {
    error.value = "Vui lòng nhập tên đăng nhập và mật khẩu.";
    return;
  }

  error.value = "";
  loading.value = true;

  try {
    const response = await tauriLogin({
      username: username.value.trim(),
      password: password.value.trim(),
    });
    auth.setUser(response, rememberMe.value);
    await menu.load(response.user_id);
    router.push(auth.returnPath ?? defaultRoute.path);
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
          <i class="pi pi-lock text-xl" />
        </div>
        <div>
          <h1 class="text-xl font-bold leading-tight">Manager System Helps</h1>
          <p class="mt-1 text-sm text-muted">Sign in to continue.</p>
        </div>
      </div>

      <form class="mt-6 space-y-4" @submit.prevent="submitLogin">
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

        <label class="block">
          <span class="text-xs font-bold text-muted">Password</span>
          <div class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 text-ink hover:border-brand focus-within:border-brand focus-within:ring-2 focus-within:ring-brand/20">
            <i class="pi pi-lock shrink-0 text-muted" />
            <Password
              v-model="password"
              class="h-full min-w-0 flex-1 border-0 bg-transparent text-sm shadow-none outline-none [&_input]:h-full [&_input]:w-full [&_input]:border-0 [&_input]:bg-transparent [&_input]:shadow-none [&_input]:outline-none"
              input-class="!border-0 !bg-transparent !shadow-none !outline-none !ring-0 !p-0"
              autocomplete="current-password"
              placeholder="password"
              :feedback="false"
              toggle-mask
            />
          </div>
        </label>

        <div class="flex cursor-pointer items-center gap-2 select-none">
          <Checkbox v-model="rememberMe" binary input-id="remember-me" />
          <label for="remember-me" class="cursor-pointer text-sm text-muted">Ghi nhớ thông tin đăng nhập</label>
        </div>

        <p
          v-if="error"
          class="rounded-md border border-red-500/20 bg-red-500/10 px-3 py-2 text-sm font-semibold text-red-500"
        >
          {{ error }}
        </p>

        <Button
          :disabled="loading"
          :icon="loading ? 'pi pi-spinner pi-spin' : 'pi pi-sign-in'"
          :label="loading ? 'Đang đăng nhập...' : 'Login'"
          class="w-full"
          type="submit"
        />
      </form>

      <p class="mt-4 text-center text-sm">
        <RouterLink to="/forgot-password" class="text-brand hover:underline">
          Quên mật khẩu?
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

:deep(.p-password) {
  height: 100%;
}
</style>
