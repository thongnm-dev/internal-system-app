<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import {
  backlogGetConfig,
  backlogSaveConfig,
  type BacklogConfig,
} from "@/tauri/commands/backlog";
import { useToast } from "@/shared/composables/useToast";

defineProps<{
  isChecking: boolean;
}>();

const emit = defineEmits<{
  (event: "retry"): void;
}>();

const toast = useToast();
const showDialog = ref(false);
const isSaving = ref(false);
const loadError = ref("");

const form = ref<BacklogConfig>({
  url: "",
  apiKey: "",
});

const submitted = ref(false);

function fieldError(value: string): string {
  return submitted.value && !value.trim() ? "Trường này là bắt buộc." : "";
}

async function openConfigDialog() {
  loadError.value = "";
  submitted.value = false;
  try {
    const config = await backlogGetConfig();
    form.value = {
      url: config.url || "",
      apiKey: config.apiKey || "",
    };
  } catch {
    form.value = { url: "", apiKey: "" };
  }
  showDialog.value = true;
}

async function saveConfig() {
  submitted.value = true;
  if (!form.value.url.trim() || !form.value.apiKey.trim()) return;

  isSaving.value = true;
  loadError.value = "";
  try {
    await backlogSaveConfig({
      url: form.value.url.trim(),
      apiKey: form.value.apiKey.trim(),
    });
    toast.success("Đã lưu cấu hình Backlog thành công.");
    showDialog.value = false;
    emit("retry");
  } catch (e) {
    loadError.value = String(e);
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <section class="flex min-h-0 flex-1 items-center justify-center overflow-hidden">
    <div
      role="alert"
      aria-live="assertive"
      class="flex max-w-md select-none flex-col items-center gap-6 px-8 text-center"
    >
      <span
        aria-hidden="true"
        class="config-error-icon flex h-20 w-20 items-center justify-center rounded-full"
      >
        <svg class="h-10 w-10" fill="none" stroke="currentColor" stroke-width="1.6" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 0 0 1.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 0 0-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 0 0-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 0 0-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 0 0-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 0 0 1.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.573-1.066Z" />
          <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
        </svg>
      </span>

      <div class="flex flex-col gap-2">
        <h1 class="text-lg font-semibold text-ink">Lỗi cấu hình Backlog</h1>
        <p class="text-sm text-secondary">
          Chức năng Issue Backlog yêu cầu cấu hình kết nối Backlog API.
          Vui lòng thiết lập <strong>URL</strong> và <strong>API Key</strong> để sử dụng.
        </p>
      </div>

      <div class="flex gap-3">
        <Button
          icon="pi pi-cog"
          label="Cấu hình Backlog"
          @click="openConfigDialog()"
        />
        <Button
          :icon="isChecking ? 'pi pi-spinner pi-spin' : undefined"
          :label="isChecking ? 'Đang kiểm tra...' : 'Thử lại'"
          :disabled="isChecking"
          severity="secondary"
          outlined
          @click="emit('retry')"
        />
      </div>
    </div>
  </section>

  <!-- Backlog Config Dialog -->
  <Dialog
    v-model:visible="showDialog"
    header="Cấu hình kết nối Backlog"
    modal
    :style="{ width: '32rem' }"
    :closable="!isSaving"
  >
    <div class="flex flex-col gap-4">
      <div class="block">
        <span class="text-xs font-bold text-muted">
          Backlog URL <span class="text-red-500">*</span>
        </span>
        <InputText
          v-model="form.url"
          class="mt-1 w-full"
          :invalid="!!fieldError(form.url)"
          placeholder="https://your-space.backlog.com"
        />
        <small v-if="fieldError(form.url)" class="text-xs text-red-500">{{ fieldError(form.url) }}</small>
      </div>

      <div class="block">
        <span class="text-xs font-bold text-muted">
          API Key <span class="text-red-500">*</span>
        </span>
        <InputText
          v-model="form.apiKey"
          class="mt-1 w-full"
          type="password"
          :invalid="!!fieldError(form.apiKey)"
          placeholder="Your Backlog API key"
        />
        <small v-if="fieldError(form.apiKey)" class="text-xs text-red-500">{{ fieldError(form.apiKey) }}</small>
      </div>

      <p v-if="loadError" class="config-error-box rounded-md border p-2 text-sm">
        {{ loadError }}
      </p>
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2">
        <Button label="Huỷ" severity="secondary" outlined :disabled="isSaving" @click="showDialog = false" />
        <Button
          icon="pi pi-save"
          label="Lưu cấu hình"
          :loading="isSaving"
          @click="saveConfig()"
        />
      </div>
    </template>
  </Dialog>
</template>

<style scoped>
.config-error-icon {
  background: #fef2f2;
  color: #dc2626;
}
[data-theme='dark'] .config-error-icon {
  background: #450a0a;
  color: #f87171;
}

.config-error-box {
  border-color: #fecaca;
  background: #fef2f2;
  color: #b91c1c;
}
[data-theme='dark'] .config-error-box {
  border-color: #991b1b;
  background: #450a0a;
  color: #fca5a5;
}
</style>
