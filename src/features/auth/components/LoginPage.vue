<script setup lang="ts">
import { ref } from "vue";
import { RouterLink, useRouter } from "vue-router";
import { useAuthStore } from "@/app/stores/auth";
import { defaultRoute } from "@/app/router/routes";
import { login as tauriLogin, friendlyError } from "@/tauri/commands";

const router = useRouter();
const auth = useAuthStore();

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
    router.push(auth.returnPath ?? defaultRoute.path);
  } catch (e) {
    error.value = friendlyError(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <main class="grid min-h-screen place-items-center bg-canvas px-6 text-ink">
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
          <div class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 text-ink hover:border-brand focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
            <i class="pi pi-user shrink-0 text-muted" />
            <input
              v-model="username"
              class="h-full min-w-0 flex-1 border-0 bg-transparent text-sm shadow-none outline-none"
              autocomplete="username"
              autofocus
              placeholder="username"
              type="text"
            />
          </div>
        </label>

        <label class="block">
          <span class="text-xs font-bold text-muted">Password</span>
          <div class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 text-ink hover:border-brand focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
            <i class="pi pi-lock shrink-0 text-muted" />
            <input
              v-model="password"
              class="h-full min-w-0 flex-1 border-0 bg-transparent text-sm shadow-none outline-none"
              autocomplete="current-password"
              placeholder="password"
              type="password"
            />
          </div>
        </label>

        <label class="flex cursor-pointer items-center gap-2 select-none">
          <input
            v-model="rememberMe"
            type="checkbox"
            class="h-4 w-4 rounded border-divider accent-brand"
          />
          <span class="text-sm text-muted">Ghi nhớ thông tin đăng nhập</span>
        </label>

        <p
          v-if="error"
          class="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm font-semibold text-red-700"
        >
          {{ error }}
        </p>

        <button
          :disabled="loading"
          class="flex h-10 w-full items-center justify-center gap-2 rounded-md bg-brand px-3 text-sm font-bold text-white hover:opacity-90 disabled:opacity-60"
          type="submit"
        >
          <i :class="loading ? 'pi pi-spinner pi-spin' : 'pi pi-sign-in'" />
          {{ loading ? "Đang đăng nhập..." : "Login" }}
        </button>
      </form>

      <p class="mt-4 text-center text-sm">
        <RouterLink to="/forgot-password" class="text-brand hover:underline">
          Quên mật khẩu?
        </RouterLink>
      </p>
    </section>
  </main>
</template>
