<script setup lang="ts">
import { onMounted } from "vue";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Fieldset from "primevue/fieldset";
import { useAppConfig } from "../composables/useAppConfig";
import { useToast } from "@/shared/composables/useToast";

const ctrl = useAppConfig();
const toast = useToast();

async function handleSave() {
  if (await ctrl.save()) {
    toast.success("Config saved successfully.");
  }
}

onMounted(() => ctrl.init());
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-auto">
    <!-- Error banner -->
    <p
      v-if="ctrl.error.value"
      class="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300"
    >
      {{ ctrl.error.value }}
    </p>

    <!-- Action bar -->
    <section class="flex items-center justify-between rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="flex items-center gap-2 text-xs text-muted">
        
      </div>
      <div class="flex items-center gap-2">
        <Button
          icon="pi pi-refresh"
          label="Discard"
          severity="secondary"
          outlined
          :disabled="!ctrl.isDirty.value"
          @click="ctrl.discard()"
        />
        <Button
          icon="pi pi-save"
          label="Save"
          :disabled="!ctrl.isDirty.value"
          @click="handleSave"
        />
      </div>
    </section>

    <!-- Loading -->
    <p
      v-if="ctrl.loading.value"
      class="flex items-center gap-2 rounded-lg border border-divider bg-panel p-4 text-sm text-muted shadow-sm"
    >
      <i class="pi pi-spinner animate-spin" />
      Loading config...
    </p>

    <!-- Config sections -->
    <template v-if="!ctrl.loading.value">
      <Fieldset
        v-for="(section, si) in ctrl.sections.value"
        :key="si"
        class="rounded-lg border border-divider bg-panel shadow-sm fieldset-nested"
        toggleable
      >
        <template #legend>
          <div class="flex items-center gap-2 px-6 py-3">
            <i class="pi pi-cog text-sm text-brand" />
            <span class="text-sm font-bold">{{ section.name }}</span>
          </div>
        </template>

        <div class="mt-2 space-y-4">
          <div
            v-for="(entry, ei) in section.entries"
            :key="ei"
            class="flex items-center gap-2"
          >
            <InputText
              class="h-9 w-56 shrink-0 font-mono text-sm"
              :model-value="entry.key"
              placeholder="KEY"
              disabled
              @update:model-value="ctrl.updateEntryKey(si, ei, $event as string)"
            />
            <span class="text-muted">=</span>
            <InputText
              class="h-9 min-w-0 flex-1 font-mono text-sm"
              :model-value="entry.value"
              placeholder="value"
              @update:model-value="ctrl.updateEntry(si, ei, $event as string)"
            />
          </div>

          <Button
            icon="pi pi-plus"
            label="Add entry"
            severity="secondary"
            text
            size="small"
            @click="ctrl.addEntry(si)"
          />
        </div>
      </Fieldset>

      <!-- Add section button -->
      <div class="flex justify-center">
        <Button
          icon="pi pi-plus"
          label="Add section"
          severity="secondary"
          outlined
          @click="ctrl.addSection()"
        />
      </div>
    </template>
  </section>
</template>
