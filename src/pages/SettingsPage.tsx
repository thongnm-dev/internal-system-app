import { KeyRound, Languages, Moon, Plus, Sun, Trash2, UserRound } from "lucide-react";
import type { ApiKeySetting, LanguageCode, ThemeMode, UserSettings } from "../controller/useSettingsController";

type SettingsPageProps = {
  apiKeyCount: number;
  apiKeys: ApiKeySetting[];
  language: LanguageCode;
  onAddApiKey: () => void;
  onApiKeyChange: (id: string, key: keyof Omit<ApiKeySetting, "id">, value: string) => void;
  onLanguageChange: (language: LanguageCode) => void;
  onRemoveApiKey: (id: string) => void;
  onThemeChange: (theme: ThemeMode) => void;
  onUserChange: (key: keyof UserSettings, value: string) => void;
  theme: ThemeMode;
  user: UserSettings;
};

const userFields: Array<{ key: keyof UserSettings; label: string; type?: string; placeholder: string }> = [
  { key: "username", label: "Username", placeholder: "username" },
  { key: "password", label: "Password", type: "password", placeholder: "password" },
  { key: "fullName", label: "Name", placeholder: "full name" },
  { key: "email", label: "Mail", type: "email", placeholder: "mail@example.com" },
  { key: "phone", label: "Phone", placeholder: "phone number" },
  { key: "address", label: "Address", placeholder: "address" },
  { key: "position", label: "Position", placeholder: "position" },
];

export function SettingsPage({
  apiKeyCount,
  apiKeys,
  language,
  onAddApiKey,
  onApiKeyChange,
  onLanguageChange,
  onRemoveApiKey,
  onThemeChange,
  onUserChange,
  theme,
  user,
}: SettingsPageProps) {
  return (
    <section className="min-h-0 flex-1 overflow-auto">
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
        <section className="rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
          <div className="flex items-center gap-2">
            <UserRound className="h-5 w-5 text-brand" />
            <h3 className="font-bold">User profile</h3>
          </div>

          <div className="mt-4 grid grid-cols-2 gap-3">
            {userFields.map((field) => (
              <label key={field.key} className={field.key === "address" ? "col-span-2" : undefined}>
                <span className="text-xs font-bold text-slate-500">{field.label}</span>
                <input
                  className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder={field.placeholder}
                  type={field.type ?? "text"}
                  value={user[field.key]}
                  onChange={(event) => onUserChange(field.key, event.target.value)}
                />
              </label>
            ))}
          </div>
        </section>

        <div className="space-y-4">
          <section className="rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
            <div className="flex items-center gap-2">
              {theme === "dark" ? <Moon className="h-5 w-5 text-brand" /> : <Sun className="h-5 w-5 text-brand" />}
              <h3 className="font-bold">Theme</h3>
            </div>
            <div className="mt-4 grid grid-cols-2 rounded-md border border-slate-200 bg-slate-100 p-1">
              <button
                className={[
                  "flex h-9 items-center justify-center gap-2 rounded px-3 text-sm font-bold",
                  theme === "light" ? "bg-white text-slate-900 shadow-sm" : "text-slate-600 hover:text-slate-900",
                ].join(" ")}
                type="button"
                onClick={() => onThemeChange("light")}
              >
                <Sun className="h-4 w-4" />
                Light
              </button>
              <button
                className={[
                  "flex h-9 items-center justify-center gap-2 rounded px-3 text-sm font-bold",
                  theme === "dark" ? "bg-slate-900 text-white shadow-sm" : "text-slate-600 hover:text-slate-900",
                ].join(" ")}
                type="button"
                onClick={() => onThemeChange("dark")}
              >
                <Moon className="h-4 w-4" />
                Dark
              </button>
            </div>
          </section>

          <section className="rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
            <div className="flex items-center gap-2">
              <Languages className="h-5 w-5 text-brand" />
              <h3 className="font-bold">Language</h3>
            </div>
            <select
              className="mt-4 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              value={language}
              onChange={(event) => onLanguageChange(event.target.value as LanguageCode)}
            >
              <option value="vi">Vietnamese</option>
              <option value="en">English</option>
              <option value="ja">Japanese</option>
            </select>
          </section>

          <section className="rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
            <span className="text-sm font-bold text-slate-500">API keys</span>
            <strong className="mt-2 block text-3xl text-ink">{apiKeyCount}</strong>
            <p className="mt-1 text-sm text-slate-500">configured links</p>
          </section>
        </div>
      </div>

      <section className="mt-4 rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <KeyRound className="h-5 w-5 text-brand" />
            <h3 className="font-bold">API key settings</h3>
          </div>
          <button
            className="flex h-9 items-center justify-center gap-2 rounded-md bg-brand px-3 text-sm font-bold text-white hover:opacity-90"
            type="button"
            onClick={onAddApiKey}
          >
            <Plus className="h-4 w-4" />
            Add key
          </button>
        </div>

        <div className="mt-4 space-y-3">
          {apiKeys.map((apiKey) => (
            <div key={apiKey.id} className="grid grid-cols-[minmax(160px,240px)_minmax(180px,260px)_minmax(0,1fr)_40px] gap-2">
              <input
                className="h-10 rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Application name *"
                type="text"
                value={apiKey.name}
                onChange={(event) => onApiKeyChange(apiKey.id, "name", event.target.value)}
              />
              <input
                className="h-10 rounded-md border border-slate-300 bg-white px-3 font-mono text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="KEY *"
                type="text"
                value={apiKey.key}
                onChange={(event) => onApiKeyChange(apiKey.id, "key", event.target.value.toUpperCase())}
              />
              <input
                className="h-10 min-w-0 rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="API key *"
                type="password"
                value={apiKey.apiKey}
                onChange={(event) => onApiKeyChange(apiKey.id, "apiKey", event.target.value)}
              />
              <button
                className="flex h-10 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
                type="button"
                title="Remove API key"
                onClick={() => onRemoveApiKey(apiKey.id)}
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          ))}
        </div>
      </section>
    </section>
  );
}
