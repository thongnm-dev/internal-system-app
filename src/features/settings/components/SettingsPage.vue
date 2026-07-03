<script setup lang="ts">
import { useSettings } from "../composables/useSettings";
import type { UserSettings } from "../composables/useSettings";

const { settings, apiKeyCount, updateUser, updateTheme, updateLanguage, updateApiKey, addApiKey, removeApiKey } =
  useSettings();

const userFields: { key: keyof UserSettings; label: string; type?: string; placeholder: string }[] = [
  { key: "username", label: "Username", placeholder: "username" },
  { key: "password", label: "Password", type: "password", placeholder: "password" },
  { key: "fullName", label: "Name", placeholder: "full name" },
  { key: "email", label: "Mail", type: "email", placeholder: "mail@example.com" },
  { key: "phone", label: "Phone", placeholder: "phone number" },
  { key: "address", label: "Address", placeholder: "address" },
  { key: "position", label: "Position", placeholder: "position" },
];

const languageOptions = [
  { label: "Vietnamese", value: "vi" as const },
  { label: "English", value: "en" as const },
  { label: "Japanese", value: "ja" as const },
];

const themeOptions = [
  { label: "Light", value: "light" as const, icon: "pi-sun" },
  { label: "Dark", value: "dark" as const, icon: "pi-moon" },
];
</script>

<template>
  <section class="min-h-0 flex-1 overflow-auto">
    <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
      <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
        <div class="flex items-center gap-2">
          <i class="pi pi-user text-xl text-brand" />
          <h3 class="font-bold">User profile</h3>
        </div>

        <div class="mt-4 grid grid-cols-2 gap-3">
          <label v-for="field in userFields" :key="field.key" :class="field.key === 'address' ? 'col-span-2' : undefined">
            <span class="text-xs font-bold text-muted">{{ field.label }}</span>
            <input
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :placeholder="field.placeholder"
              :type="field.type ?? 'text'"
              :value="settings.user[field.key]"
              @input="updateUser(field.key, ($event.target as HTMLInputElement).value)"
            />
          </label>
        </div>
      </section>

      <div class="space-y-4">
        <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <div class="flex items-center gap-2">
            <i :class="`pi ${settings.theme === 'dark' ? 'pi-moon' : 'pi-sun'} text-xl text-brand`" />
            <h3 class="font-bold">Theme</h3>
          </div>
          <div class="mt-4 grid grid-cols-2 rounded-md border border-divider bg-canvas p-1">
            <button
              v-for="opt in themeOptions"
              :key="opt.value"
              :class="[
                'flex h-9 items-center justify-center gap-2 rounded-md text-sm font-bold transition',
                settings.theme === opt.value ? 'bg-panel text-ink shadow-sm' : 'text-muted hover:text-secondary',
              ]"
              type="button"
              @click="updateTheme(opt.value)"
            >
              <i :class="`pi ${opt.icon}`" />
              {{ opt.label }}
            </button>
          </div>
        </section>

        <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <div class="flex items-center gap-2">
            <i class="pi pi-language text-xl text-brand" />
            <h3 class="font-bold">Language</h3>
          </div>
          <select
            class="mt-4 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            :value="settings.language"
            @change="updateLanguage(($event.target as HTMLSelectElement).value as any)"
          >
            <option v-for="opt in languageOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </section>

        <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <span class="text-sm font-bold text-muted">API keys</span>
          <strong class="mt-2 block text-3xl text-ink">{{ apiKeyCount }}</strong>
          <p class="mt-1 text-sm text-muted">configured links</p>
        </section>
      </div>
    </div>

    <section class="mt-4 rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-2">
          <i class="pi pi-key text-xl text-brand" />
          <h3 class="font-bold">API key settings</h3>
        </div>
        <button
          class="flex h-9 items-center justify-center gap-2 rounded-md bg-brand px-3 text-sm font-bold text-white hover:opacity-90"
          type="button"
          @click="addApiKey"
        >
          <i class="pi pi-plus" />
          Add key
        </button>
      </div>

      <div class="mt-4 space-y-3">
        <div
          v-for="ak in settings.apiKeys"
          :key="ak.id"
          class="grid grid-cols-[minmax(160px,240px)_minmax(180px,260px)_minmax(0,1fr)_40px] gap-2"
        >
          <input
            class="h-10 rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            placeholder="Application name *"
            type="text"
            :value="ak.name"
            @input="updateApiKey(ak.id, 'name', ($event.target as HTMLInputElement).value)"
          />
          <input
            class="h-10 rounded-md border border-divider bg-panel px-3 font-mono text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            placeholder="KEY *"
            type="text"
            :value="ak.key"
            @input="updateApiKey(ak.id, 'key', ($event.target as HTMLInputElement).value.toUpperCase())"
          />
          <input
            class="h-10 min-w-0 rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            placeholder="API key *"
            type="password"
            :value="ak.apiKey"
            @input="updateApiKey(ak.id, 'apiKey', ($event.target as HTMLInputElement).value)"
          />
          <button
            class="flex h-10 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
            type="button"
            title="Remove API key"
            @click="removeApiKey(ak.id)"
          >
            <i class="pi pi-trash" />
          </button>
        </div>
      </div>
    </section>
  </section>
</template>
