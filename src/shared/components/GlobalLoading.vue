<script setup lang="ts">
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";

const { isLoading } = useGlobalLoading();
</script>

<template>
  <Transition name="global-loading">
    <div v-if="isLoading" class="global-loading-overlay">
      <div class="global-loading-card">
        <div class="global-loading-spinner" />
        <span class="global-loading-text">Loading...</span>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.global-loading-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(0 0 0 / 0.25);
}

.global-loading-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 28px 36px;
  border-radius: 12px;
  background: rgb(var(--color-panel));
  box-shadow: 0 8px 32px rgb(0 0 0 / 0.18);
}

.global-loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgb(var(--color-border));
  border-top-color: rgb(var(--color-brand));
  border-radius: 50%;
  animation: global-loading-spin 0.7s linear infinite;
}

.global-loading-text {
  font-size: 13px;
  font-weight: 600;
  color: rgb(var(--color-text-secondary));
}

@keyframes global-loading-spin {
  to {
    transform: rotate(360deg);
  }
}

.global-loading-enter-active {
  transition: opacity 0.15s ease-out;
}

.global-loading-leave-active {
  transition: opacity 0.2s ease-in;
}

.global-loading-enter-from,
.global-loading-leave-to {
  opacity: 0;
}
</style>
