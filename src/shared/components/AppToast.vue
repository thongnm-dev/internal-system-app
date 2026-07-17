<script setup lang="ts">
import Button from "primevue/button";
import { useToast } from "@/shared/composables/useToast";

const { toasts, dismiss } = useToast();

const icons: Record<string, string> = {
  success: "pi pi-check-circle",
  error: "pi pi-times-circle",
  info: "pi pi-info-circle",
};
</script>

<template>
  <Teleport to="body">
    <div class="fixed right-4 top-4 z-[9999] flex w-80 flex-col gap-2">
      <TransitionGroup
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="translate-x-full opacity-0"
        enter-to-class="translate-x-0 opacity-100"
        leave-active-class="transition duration-200 ease-in"
        leave-from-class="translate-x-0 opacity-100"
        leave-to-class="translate-x-full opacity-0"
      >
        <div
          v-for="toast in toasts"
          :key="toast.id"
          :class="['toast-item flex items-start gap-3 rounded-lg border p-3 shadow-lg', `toast-${toast.type}`]"
        >
          <i :class="[icons[toast.type], 'mt-0.5 shrink-0 text-base']" />
          <span class="min-w-0 flex-1 text-sm font-medium">{{ toast.message }}</span>
          <Button icon="pi pi-times" text rounded size="small" class="shrink-0 opacity-60 hover:opacity-100" @click="dismiss(toast.id)" />
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-item {
  backdrop-filter: blur(8px);
}

.toast-success {
  border-color: #6ee7b7;
  background: #ecfdf5;
  color: #065f46;
}
.toast-error {
  border-color: #fca5a5;
  background: #fef2f2;
  color: #991b1b;
}
.toast-info {
  border-color: #93c5fd;
  background: #eff6ff;
  color: #1e40af;
}

[data-theme="dark"] .toast-success {
  border-color: #065f46;
  background: #022c22;
  color: #6ee7b7;
}
[data-theme="dark"] .toast-error {
  border-color: #991b1b;
  background: #450a0a;
  color: #fca5a5;
}
[data-theme="dark"] .toast-info {
  border-color: #1e40af;
  background: #172554;
  color: #93c5fd;
}
</style>
