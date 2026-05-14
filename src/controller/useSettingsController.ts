import { useEffect, useMemo, useState } from "react";

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

const settingsStorageKey = "pjjyuji.app.settings";

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
  apiKeys: [
    {
      id: "default",
      name: "",
      key: "",
      apiKey: "",
    },
  ],
};

export function useSettingsController() {
  const [settings, setSettings] = useState<StoredSettings>(() => loadSettings());

  useEffect(() => {
    document.documentElement.dataset.theme = settings.theme;
    window.localStorage.setItem(settingsStorageKey, JSON.stringify(settings));
  }, [settings]);

  const apiKeyCount = useMemo(
    () => settings.apiKeys.filter((apiKey) => apiKey.name.trim() && apiKey.key.trim() && apiKey.apiKey.trim()).length,
    [settings.apiKeys],
  );

  const updateUser = (key: keyof UserSettings, value: string) => {
    setSettings((current) => ({
      ...current,
      user: {
        ...current.user,
        [key]: value,
      },
    }));
  };

  const updateTheme = (theme: ThemeMode) => {
    setSettings((current) => ({ ...current, theme }));
  };

  const updateLanguage = (language: LanguageCode) => {
    setSettings((current) => ({ ...current, language }));
  };

  const updateApiKey = (id: string, key: keyof Omit<ApiKeySetting, "id">, value: string) => {
    setSettings((current) => ({
      ...current,
      apiKeys: current.apiKeys.map((apiKey) => (apiKey.id === id ? { ...apiKey, [key]: value } : apiKey)),
    }));
  };

  const addApiKey = () => {
    setSettings((current) => ({
      ...current,
      apiKeys: [
        ...current.apiKeys,
        {
          id: crypto.randomUUID(),
          name: "",
          key: "",
          apiKey: "",
        },
      ],
    }));
  };

  const removeApiKey = (id: string) => {
    setSettings((current) => {
      const nextApiKeys = current.apiKeys.filter((apiKey) => apiKey.id !== id);
      return {
        ...current,
        apiKeys: nextApiKeys.length > 0 ? nextApiKeys : defaultSettings.apiKeys,
      };
    });
  };

  return {
    addApiKey,
    apiKeyCount,
    removeApiKey,
    settings,
    updateApiKey,
    updateLanguage,
    updateTheme,
    updateUser,
  };
}

export function applyStoredThemePreference() {
  const settings = loadSettings();
  document.documentElement.dataset.theme = settings.theme;
}

function loadSettings(): StoredSettings {
  try {
    const saved = window.localStorage.getItem(settingsStorageKey);
    if (!saved) {
      return defaultSettings;
    }

    const parsed = JSON.parse(saved) as Partial<StoredSettings>;
    return {
      user: { ...defaultSettings.user, ...parsed.user },
      theme: parsed.theme === "dark" ? "dark" : "light",
      language: isLanguageCode(parsed.language) ? parsed.language : defaultSettings.language,
      apiKeys:
        Array.isArray(parsed.apiKeys) && parsed.apiKeys.length > 0
          ? parsed.apiKeys.map((apiKey) => ({ key: "", ...apiKey }))
          : defaultSettings.apiKeys,
    };
  } catch {
    return defaultSettings;
  }
}

function isLanguageCode(value: unknown): value is LanguageCode {
  return value === "vi" || value === "en" || value === "ja";
}
