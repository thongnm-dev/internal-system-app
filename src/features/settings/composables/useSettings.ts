import { computed, ref, watch } from "vue";
import { applyTheme } from "@/shared/config/themeTokens";

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

function loadSettings(): StoredSettings {
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
  const savedSnapshot = ref<StoredSettings>(loadSettings());
  const settings = ref<StoredSettings>(cloneSettings(savedSnapshot.value));

  const apiKeyCount = computed(
    () => settings.value.apiKeys.filter((k) => k.name.trim() && k.key.trim() && k.apiKey.trim()).length,
  );

  const isDirty = computed(() => JSON.stringify(settings.value) !== JSON.stringify(savedSnapshot.value));

  watch(
    () => settings.value.theme,
    (theme) => applyTheme(theme),
  );

  function persist() {
    window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings.value));
    savedSnapshot.value = cloneSettings(settings.value);
  }

  function save() {
    persist();
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
