import { computed, ref, watch } from "vue";
import { applyTheme } from "@/shared/config/themeTokens";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { getSettings, saveSettings } from "@/tauri/commands/settings";
import type { AppSettings as TauriAppSettings, ApiKeySetting as TauriApiKeySetting } from "@/_/types/settings";

export type ThemeMode = "light" | "dark";
export type LanguageCode = "vi" | "en" | "ja";

export type UserSettings = {
  username: string;
  password: string;
  fullName: string;
  email: string;
  phone: string;
  address: string;
  position: string;
};

export type ApiKeySetting = {
  id: string;
  name: string;
  key: string;
  apiKey: string;
};

type StoredSettings = {
  apiKeys: ApiKeySetting[];
  language: LanguageCode;
  theme: ThemeMode;
  user: UserSettings;
};

const SETTINGS_KEY = "pjjyuji.app.settings";

const defaultSettings: StoredSettings = {
  user: {
    username: "",
    password: "",
    fullName: "",
    email: "",
    phone: "",
    address: "",
    position: "",
  },
  theme: "light",
  language: "vi",
  apiKeys: [{ id: "default", name: "", key: "", apiKey: "" }],
};

function fromTauri(ts: TauriAppSettings): StoredSettings {
  return {
    user: {
      username: ts.user.username,
      password: ts.user.password,
      fullName: ts.user.full_name,
      email: ts.user.email,
      phone: ts.user.phone,
      address: ts.user.address,
      position: ts.user.position,
    },
    theme: ts.theme === "dark" ? "dark" : "light",
    language: isLanguageCode(ts.language) ? ts.language : "vi",
    apiKeys:
      ts.api_keys.length > 0
        ? ts.api_keys.map((k: TauriApiKeySetting) => ({
            id: k.id,
            name: k.name,
            key: k.key,
            apiKey: k.api_key,
          }))
        : [{ id: "default", name: "", key: "", apiKey: "" }],
  };
}

function toTauriRequest(s: StoredSettings) {
  return {
    user: {
      username: s.user.username,
      password: s.user.password,
      full_name: s.user.fullName,
      email: s.user.email,
      phone: s.user.phone,
      address: s.user.address,
      position: s.user.position,
    },
    theme: s.theme,
    language: s.language,
    api_keys: s.apiKeys.map((k) => ({
      id: k.id,
      name: k.name,
      key: k.key,
      api_key: k.apiKey,
    })),
  };
}

function loadSettingsFromLocal(): StoredSettings {
  try {
    const saved = window.localStorage.getItem(SETTINGS_KEY);
    if (!saved) return { ...defaultSettings };
    const parsed = JSON.parse(saved) as Partial<StoredSettings>;
    return {
      user: { ...defaultSettings.user, ...parsed.user },
      theme: parsed.theme === "dark" ? "dark" : "light",
      language: isLanguageCode(parsed.language) ? parsed.language : defaultSettings.language,
      apiKeys:
        Array.isArray(parsed.apiKeys) && parsed.apiKeys.length > 0
          ? parsed.apiKeys.map((k) => ({ ...k, key: k.key ?? "" }))
          : [...defaultSettings.apiKeys],
    };
  } catch {
    return { ...defaultSettings };
  }
}

function isLanguageCode(v: unknown): v is LanguageCode {
  return v === "vi" || v === "en" || v === "ja";
}

function cloneSettings(s: StoredSettings): StoredSettings {
  return JSON.parse(JSON.stringify(s));
}

export function useSettings() {
  const savedSnapshot = ref<StoredSettings>(loadSettingsFromLocal());
  const settings = ref<StoredSettings>(cloneSettings(savedSnapshot.value));
  const loading = ref(false);
  const error = ref<string | null>(null);

  const apiKeyCount = computed(
    () => settings.value.apiKeys.filter((k) => k.name.trim() && k.key.trim() && k.apiKey.trim()).length,
  );

  const isDirty = computed(() => JSON.stringify(settings.value) !== JSON.stringify(savedSnapshot.value));

  watch(
    () => settings.value.theme,
    (theme) => applyTheme(theme),
  );

  async function loadFromBackend() {
    if (!canUseTauriRuntime()) return;
    loading.value = true;
    error.value = null;
    try {
      const result = await getSettings();
      const loaded = fromTauri(result);
      savedSnapshot.value = loaded;
      settings.value = cloneSettings(loaded);
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      loading.value = false;
    }
  }

  loadFromBackend();

  async function save() {
    if (canUseTauriRuntime()) {
      loading.value = true;
      error.value = null;
      try {
        const result = await saveSettings(toTauriRequest(settings.value));
        const saved = fromTauri(result);
        savedSnapshot.value = saved;
        settings.value = cloneSettings(saved);
      } catch (e) {
        error.value = friendlyError(e);
      } finally {
        loading.value = false;
      }
    } else {
      window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings.value));
      savedSnapshot.value = cloneSettings(settings.value);
    }
  }

  function discard() {
    settings.value = cloneSettings(savedSnapshot.value);
  }

  function updateUser(key: keyof UserSettings, value: string) {
    settings.value.user[key] = value;
  }

  function updateTheme(theme: ThemeMode) {
    settings.value.theme = theme;
  }

  function updateLanguage(language: LanguageCode) {
    settings.value.language = language;
  }

  function updateApiKey(id: string, key: keyof Omit<ApiKeySetting, "id">, value: string) {
    const item = settings.value.apiKeys.find((k) => k.id === id);
    if (item) item[key] = value;
  }

  function addApiKey() {
    settings.value.apiKeys.push({ id: crypto.randomUUID(), name: "", key: "", apiKey: "" });
  }

  function removeApiKey(id: string) {
    const next = settings.value.apiKeys.filter((k) => k.id !== id);
    settings.value.apiKeys = next.length > 0 ? next : [{ id: "default", name: "", key: "", apiKey: "" }];
  }

  return {
    settings,
    apiKeyCount,
    isDirty,
    loading,
    error,
    save,
    discard,
    updateUser,
    updateTheme,
    updateLanguage,
    updateApiKey,
    addApiKey,
    removeApiKey,
  };
}
