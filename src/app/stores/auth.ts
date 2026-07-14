import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { LoginResponse } from "@/tauri/commands";

type AuthUser = {
  user_id: number;
  username: string;
  full_name: string;
  email: string;
  roles: string[];
};

const AUTH_STORAGE_KEY = "pjjyuji.auth.session";

export const useAuthStore = defineStore("auth", () => {
  const user = ref<AuthUser | null>(null);
  const returnPath = ref<string | null>(null);

  const isAuthenticated = computed(() => user.value !== null);

  loadFromStorage();

  function setUser(response: LoginResponse, rememberMe: boolean) {
    user.value = {
      user_id: response.user_id,
      username: response.username,
      full_name: response.full_name,
      email: response.email,
      roles: response.roles,
    };
    if (rememberMe) {
      persistToStorage();
    } else {
      window.localStorage.removeItem(AUTH_STORAGE_KEY);
    }
  }

  function logout() {
    user.value = null;
    window.localStorage.removeItem(AUTH_STORAGE_KEY);
  }

  function setReturnPath(path: string | null) {
    returnPath.value = path;
  }

  function loadFromStorage() {
    try {
      const saved = window.localStorage.getItem(AUTH_STORAGE_KEY);
      if (!saved) return;

      const parsed = JSON.parse(saved) as Partial<{ user: AuthUser }>;
      if (parsed.user?.username) {
        user.value = parsed.user;
      }
    } catch {
      // ignore corrupt storage
    }
  }

  function persistToStorage() {
    window.localStorage.setItem(
      AUTH_STORAGE_KEY,
      JSON.stringify({
        isAuthenticated: true,
        user: user.value,
      }),
    );
  }

  return {
    user,
    returnPath,
    isAuthenticated,
    setUser,
    logout,
    setReturnPath,
  };
});
