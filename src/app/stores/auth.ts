import { defineStore } from "pinia";
import { computed, ref } from "vue";

type AuthUser = {
  username: string;
};

const AUTH_STORAGE_KEY = "pjjyuji.auth.session";

export const useAuthStore = defineStore("auth", () => {
  const user = ref<AuthUser | null>(null);
  const returnPath = ref<string | null>(null);

  const isAuthenticated = computed(() => user.value !== null);

  loadFromStorage();

  function login(payload: { username: string }) {
    user.value = { username: payload.username };
    persistToStorage();
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
        user.value = { username: parsed.user.username };
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
    login,
    logout,
    setReturnPath,
  };
});
