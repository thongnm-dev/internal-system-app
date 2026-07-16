<script setup lang="ts">
import { onMounted, ref } from "vue";
import Dialog from "primevue/dialog";
import { useToast } from "@/shared/composables/useToast";
import { useAiUsage } from "../composables/useAiUsage";
import type { AiAccountType } from "@/_/types/ai-usage";

const ctrl = useAiUsage();
const toast = useToast();

const isDialogOpen = ref(false);
const accountName = ref("");
const apiKey = ref("");
const showApiKey = ref(false);

onMounted(() => {
  void ctrl.loadAccounts();
});

function openDialog() {
  accountName.value = "";
  apiKey.value = "";
  showApiKey.value = false;
  isDialogOpen.value = true;
}

async function saveAccount() {
  if (!accountName.value.trim() || !apiKey.value.trim()) return;
  const ok = await ctrl.addAccount(accountName.value.trim(), apiKey.value.trim());
  if (ok) isDialogOpen.value = false;
}

function openAccountSettings(name: string) {
  toast.info(`Settings for "${name}" coming soon.`);
}

function typeLabel(type: AiAccountType): string {
  switch (type) {
    case "api":
      return "API";
    case "admin":
      return "Admin";
    case "oauth":
      return "OAuth";
    default:
      return "Unknown";
  }
}

function typeBadgeClass(type: AiAccountType): string {
  switch (type) {
    case "api":
      return "bg-brand/10 text-brand";
    case "admin":
      return "bg-amber-100 text-amber-700";
    case "oauth":
      return "bg-violet-100 text-violet-700";
    default:
      return "bg-canvas text-muted";
  }
}

function usageBarClass(percent: number): string {
  if (percent <= 10) return "bg-red-500";
  if (percent <= 30) return "bg-amber-500";
  return "bg-brand";
}
</script>

<template>
  <div class="flex flex-1 flex-col gap-4 overflow-auto">
    <div class="rounded-lg border border-divider bg-panel p-6 shadow-sm">
      <div class="flex items-center gap-3">
        <i class="pi pi-chart-bar text-2xl text-muted" />
        <div>
          <h2 class="text-lg font-semibold text-ink">AI Usage</h2>
          <p class="text-sm text-muted">Track AI usage, token consumption, and cost statistics across the team.</p>
        </div>
        <button class="ml-auto flex h-10 shrink-0 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90" type="button" @click="openDialog"><i class="pi pi-plus" />Add Account</button>
      </div>
    </div>

    <!-- Account cards -->
    <div v-if="ctrl.accounts.value.length" class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
      <div
        v-for="account in ctrl.accounts.value"
        :key="account.id"
        class="flex flex-col rounded-lg border border-divider bg-panel p-5 shadow-sm"
      >
        <div class="flex items-start gap-3">
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-brand/10 text-brand">
            <i class="pi pi-sparkles" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <h3 class="truncate font-semibold text-ink" :title="account.name">{{ account.name }}</h3>
              <span :class="['shrink-0 rounded-full px-2 py-0.5 text-[11px] font-bold', typeBadgeClass(account.account_type)]">
                {{ typeLabel(account.account_type) }}
              </span>
            </div>
            <p class="mt-0.5 font-mono text-xs text-muted">{{ account.api_key_masked }}</p>
          </div>
          <button
            class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted hover:bg-canvas hover:text-ink"
            type="button"
            title="Account settings"
            @click="openAccountSettings(account.name)"
          >
            <i class="pi pi-cog text-sm" />
          </button>
        </div>

        <!-- Usage remaining -->
        <div class="mt-4">
          <div class="flex items-center justify-between text-xs">
            <span class="font-bold text-muted">Usage remaining</span>
            <span class="font-bold text-ink">{{ Math.round(account.usage_percent) }}%</span>
          </div>
          <div class="mt-1.5 h-2 overflow-hidden rounded-full bg-canvas">
            <div
              :class="['h-full rounded-full transition-all', usageBarClass(account.usage_percent)]"
              :style="{ width: `${Math.min(100, Math.max(0, account.usage_percent))}%` }"
            />
          </div>
        </div>

        <!-- Stats -->
        <div class="mt-4 grid grid-cols-2 gap-3 border-t border-divider pt-3 text-xs text-muted">
          <div class="flex items-center gap-1.5" :title="`Resets at ${account.reset_at}`">
            <i class="pi pi-refresh" />
            <span class="truncate">Reset {{ account.reset_at }}</span>
          </div>
          <div class="flex items-center justify-end gap-1.5">
            <i class="pi pi-desktop" />
            <span>{{ account.session_count }} session{{ account.session_count === 1 ? "" : "s" }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="flex flex-1 items-center justify-center rounded-lg border border-dashed border-divider bg-panel/50 p-12">
      <p class="text-sm text-muted">
        {{ ctrl.isLoading.value ? "Loading accounts..." : "No accounts yet. Click \"Add Account\" to register one." }}
      </p>
    </div>

    <Dialog
      :visible="isDialogOpen"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="isDialogOpen = $event"
    >
      <template #header>
        <h3 class="font-bold text-ink">Add Account</h3>
      </template>

      <div class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Account Name <span class="text-red-500">*</span></span>
          <input
            v-model="accountName"
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            placeholder="e.g. team-claude"
            autofocus
          />
        </label>
        <label class="block">
          <span class="text-xs font-bold text-muted">API Key <span class="text-red-500">*</span></span>
          <div class="relative mt-1">
            <input
              v-model="apiKey"
              :type="showApiKey ? 'text' : 'password'"
              class="h-10 w-full rounded-md border border-divider bg-panel px-3 pr-10 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="sk-..."
              autocomplete="off"
            />
            <button
              class="absolute right-2 top-1/2 -translate-y-1/2 text-muted hover:text-ink"
              type="button"
              :title="showApiKey ? 'Hide API key' : 'Show API key'"
              @click="showApiKey = !showApiKey"
            >
              <i :class="`pi ${showApiKey ? 'pi-eye-slash' : 'pi-eye'}`" />
            </button>
          </div>
        </label>
        <p class="text-xs text-muted">Account type is detected automatically from the API key.</p>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <button
            class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            type="button"
            @click="isDialogOpen = false"
          >
            Cancel
          </button>
          <button
            class="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:opacity-50"
            type="button"
            :disabled="!accountName.trim() || !apiKey.trim() || ctrl.isSaving.value"
            @click="saveAccount"
          >
            {{ ctrl.isSaving.value ? "Saving..." : "Save" }}
          </button>
        </div>
      </template>
    </Dialog>
  </div>
</template>
