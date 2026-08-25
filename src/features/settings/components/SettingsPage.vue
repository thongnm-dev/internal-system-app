<script setup lang="ts">
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { useSettings } from "../composables/useSettings";
import type { UserSettings } from "../composables/useSettings";

const { settings, isDirty, loading, error, save, discard, updateUser } = useSettings();

const userFields: { key: keyof UserSettings; label: string; type?: string; placeholder: string; disabled?: boolean }[] = [
  { key: "username", label: "Username", placeholder: "username", disabled: true },
  { key: "password", label: "Password", type: "password", placeholder: "password" },
  { key: "fullName", label: "Name", placeholder: "full name" },
  { key: "email", label: "Mail", type: "email", placeholder: "mail@example.com" },
  { key: "phone", label: "Phone", placeholder: "phone number" },
  { key: "address", label: "Address", placeholder: "address" },
  { key: "position", label: "Position", placeholder: "position" },
];
</script>

<template>
  <section class="min-h-0 flex-1 overflow-auto">
    <p v-if="loading" class="flex items-center gap-2 rounded-lg border border-divider bg-panel p-4 text-sm text-muted shadow-sm">
      <i class="pi pi-spinner animate-spin" />
      Loading settings...
    </p>
    <p v-if="error" class="banner-danger mb-4">
      {{ error }}
    </p>
    <template v-if="!loading">
    <div class="grid gap-4">
      <section class="flex flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm">
        <div class="flex items-center gap-2">
          <i class="pi pi-user text-xl text-brand" />
          <h3 class="section-title">User profile</h3>
        </div>

        <div class="mt-4 grid grid-cols-2 gap-3">
          <label v-for="field in userFields" :key="field.key" :class="field.key === 'address' ? 'col-span-2' : undefined">
            <span class="text-xs font-bold text-muted">{{ field.label }}</span>
            <InputText
              :class="['mt-1 h-10 w-full rounded-md border border-divider px-3 text-sm outline-none',
                field.disabled ? 'bg-canvas text-muted cursor-not-allowed' : 'bg-panel text-ink focus:border-brand focus:ring-2 focus:ring-brand/20']"
              :placeholder="field.placeholder"
              :type="field.type ?? 'text'"
              :disabled="field.disabled"
              :model-value="settings.user[field.key]"
              @update:model-value="updateUser(field.key, $event as string)"
            />
          </label>
        </div>

        <div class="mt-4 flex items-center justify-end gap-2 border-t border-divider pt-3">
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
      </section>
    </div>
    </template>
  </section>
</template>
