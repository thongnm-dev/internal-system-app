<script setup lang="ts">
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { useSettings } from "../composables/useSettings";
import type { UserSettings } from "../composables/useSettings";

const { settings, isDirty, loading, error, save, discard, updateUser, updateTheme, updateLanguage } =
  useSettings();

const userFields: { key: keyof UserSettings; label: string; type?: string; placeholder: string; disabled?: boolean }[] = [
  { key: "username", label: "Username", placeholder: "username", disabled: true },
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
    <p v-if="loading" class="flex items-center gap-2 rounded-lg border border-divider bg-panel p-4 text-sm text-muted shadow-sm">
      <i class="pi pi-spinner animate-spin" />
      Loading settings...
    </p>
    <p v-if="error" class="mb-4 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300">
      {{ error }}
    </p>
    <template v-if="!loading">
    <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
      <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
        <div class="flex items-center gap-2">
          <i class="pi pi-user text-xl text-brand" />
          <h3 class="font-bold">User profile</h3>
        </div>

        <div class="mt-4 grid grid-cols-2 gap-3">
          <label v-for="field in userFields" :key="field.key" :class="field.key === 'address' ? 'col-span-2' : undefined">
            <span class="text-xs font-bold text-muted">{{ field.label }}</span>
            <InputText
              :class="['mt-1 h-10 w-full rounded-md border border-divider px-3 text-sm outline-none',
                field.disabled ? 'bg-canvas text-muted cursor-not-allowed' : 'bg-panel text-ink focus:border-brand focus:ring-2 focus:ring-emerald-100']"
              :placeholder="field.placeholder"
              :type="field.type ?? 'text'"
              :disabled="field.disabled"
              :model-value="settings.user[field.key]"
              @update:model-value="updateUser(field.key, $event as string)"
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
            <Button
              v-for="opt in themeOptions"
              :key="opt.value"
              :icon="`pi ${opt.icon}`"
              :label="opt.label"
              :class="[
                'flex h-9 items-center justify-center gap-2 rounded-md text-sm font-bold transition',
                settings.theme === opt.value ? 'bg-panel text-ink shadow-sm' : 'text-muted hover:text-secondary',
              ]"
              unstyled
              @click="updateTheme(opt.value)"
            />
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
      </div>
    </div>

    <div
      class="sticky bottom-0 mt-4 flex items-center justify-end gap-2 rounded-lg border border-divider bg-panel px-4 py-3 shadow-sm"
    >
      <template v-if="isDirty">
        <span class="mr-auto text-sm font-semibold text-brand">You have unsaved changes.</span>
        <Button label="Discard" severity="secondary" outlined @click="discard" />
      </template>
      <Button
        :icon="loading ? 'pi pi-spinner pi-spin' : undefined"
        :label="loading ? 'Saving...' : 'Save changes'"
        :disabled="!isDirty || loading"
        @click="save"
      />
    </div>
    </template>
  </section>
</template>
