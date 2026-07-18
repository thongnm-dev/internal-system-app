<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { useAiUsage } from "../composables/useAiUsage";
import type {
  AiAccount,
  AiAccountStatus,
  AiAccountType,
  AiProvider,
  AiUsageSource,
} from "@/_/types/ai-usage";

const ctrl = useAiUsage();

const isDialogOpen = ref(false);
const accountName = ref("");
const apiKey = ref("");
const provider = ref<AiProvider>("claude");
const showApiKey = ref(false);

const showSettings = ref(false);
const thresholdInput = ref(10);
const intervalInput = ref(60);

onMounted(() => {
  void ctrl.start();
});

/** Nhóm account theo provider để hiển thị. */
const groups = computed(() => {
  const map = new Map<AiProvider, AiAccount[]>();
  for (const account of ctrl.accounts.value) {
    const list = map.get(account.provider) ?? [];
    list.push(account);
    map.set(account.provider, list);
  }
  return Array.from(map.entries());
});

function openDialog() {
  accountName.value = "";
  apiKey.value = "";
  provider.value = "claude";
  showApiKey.value = false;
  isDialogOpen.value = true;
}

async function saveAccount() {
  if (!accountName.value.trim() || !apiKey.value.trim()) return;
  const ok = await ctrl.addAccount({
    name: accountName.value.trim(),
    api_key: apiKey.value.trim(),
    provider: provider.value,
  });
  if (ok) isDialogOpen.value = false;
}

function openSettings() {
  thresholdInput.value = ctrl.settings.value.switch_threshold_percent;
  intervalInput.value = ctrl.settings.value.poll_interval_secs;
  showSettings.value = true;
}

async function saveSettings() {
  const ok = await ctrl.saveSettings({
    switch_threshold_percent: Number(thresholdInput.value) || 0,
    poll_interval_secs: Math.max(15, Number(intervalInput.value) || 60),
  });
  if (ok) showSettings.value = false;
}

function onPriorityChange(account: AiAccount, event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value) && value !== account.priority) {
    void ctrl.updateAccount({ id: account.id, priority: value });
  }
}

function providerLabel(p: AiProvider): string {
  return p === "codex" ? "Codex" : "Claude";
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

function statusLabel(status: AiAccountStatus): string {
  switch (status) {
    case "healthy":
      return "Healthy";
    case "low":
      return "Low";
    case "exhausted":
      return "Exhausted";
    case "error":
      return "Error";
    default:
      return "Unknown";
  }
}

function statusClass(status: AiAccountStatus): string {
  switch (status) {
    case "healthy":
      return "bg-emerald-100 text-emerald-700";
    case "low":
      return "bg-amber-100 text-amber-700";
    case "exhausted":
      return "bg-red-100 text-red-700";
    case "error":
      return "bg-red-100 text-red-700";
    default:
      return "bg-canvas text-muted";
  }
}

function sourceLabel(source: AiUsageSource): string {
  switch (source) {
    case "billing_api":
      return "billing API";
    case "ratelimit_header":
      return "rate-limit header";
    case "error_signal":
      return "usage signal";
    case "manual":
      return "manual";
    default:
      return "not measured";
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
      <div class="flex flex-wrap items-center gap-3">
        <i class="pi pi-chart-bar text-2xl text-muted" />
        <div class="min-w-0">
          <h2 class="text-lg font-semibold text-ink">AI Usage</h2>
          <p class="text-sm text-muted">
            Theo dõi usage tài khoản AI và tự động chọn account ưu tiên khi tài khoản đang dùng cạn.
          </p>
        </div>
        <div class="ml-auto flex shrink-0 items-center gap-2">
          <Button
            icon="pi pi-refresh"
            label="Refresh"
            severity="secondary"
            :loading="ctrl.isRefreshing.value"
            @click="ctrl.refresh()"
          />
          <Button icon="pi pi-cog" label="Settings" severity="secondary" @click="openSettings" />
          <Button icon="pi pi-plus" label="Add Account" @click="openDialog" />
        </div>
      </div>
    </div>

    <!-- Account groups by provider -->
    <template v-if="ctrl.accounts.value.length">
      <div v-for="[groupProvider, list] in groups" :key="groupProvider" class="space-y-3">
        <h3 class="px-1 text-sm font-bold uppercase tracking-wide text-muted">
          {{ providerLabel(groupProvider) }}
        </h3>
        <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
          <div
            v-for="account in list"
            :key="account.id"
            class="flex flex-col rounded-lg border bg-panel p-5 shadow-sm"
            :class="account.is_active ? 'border-brand ring-1 ring-brand/40' : 'border-divider'"
          >
            <div class="flex items-start gap-3">
              <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-brand/10 text-brand">
                <i class="pi pi-sparkles" />
              </div>
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-2">
                  <h3 class="truncate font-semibold text-ink" :title="account.name">{{ account.name }}</h3>
                  <span v-if="account.is_active" class="shrink-0 rounded-full bg-brand px-2 py-0.5 text-[11px] font-bold text-white">
                    ACTIVE
                  </span>
                  <span :class="['shrink-0 rounded-full px-2 py-0.5 text-[11px] font-bold', typeBadgeClass(account.account_type)]">
                    {{ typeLabel(account.account_type) }}
                  </span>
                </div>
                <p class="mt-0.5 font-mono text-xs text-muted">{{ account.api_key_masked }}</p>
              </div>
              <Button
                icon="pi pi-trash"
                severity="danger"
                text
                rounded
                size="small"
                title="Delete account"
                @click="ctrl.deleteAccount(account.id)"
              />
            </div>

            <!-- Status + usage source -->
            <div class="mt-3 flex flex-wrap items-center gap-2 text-[11px]">
              <span :class="['rounded-full px-2 py-0.5 font-bold', statusClass(account.status)]">
                {{ statusLabel(account.status) }}
              </span>
              <span class="text-muted">source: {{ sourceLabel(account.usage_source) }}</span>
            </div>

            <!-- Usage remaining -->
            <div class="mt-3">
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
            <div class="mt-3 grid grid-cols-2 gap-3 border-t border-divider pt-3 text-xs text-muted">
              <div class="flex items-center gap-1.5" :title="`Resets at ${account.reset_at || 'unknown'}`">
                <i class="pi pi-refresh" />
                <span class="truncate">Reset {{ account.reset_at || "—" }}</span>
              </div>
              <label class="flex items-center justify-end gap-1.5" title="Priority (số nhỏ = ưu tiên cao)">
                <i class="pi pi-sort-amount-down" />
                <span>Priority</span>
                <input
                  type="number"
                  class="w-14 rounded border border-divider bg-canvas px-1.5 py-0.5 text-right text-ink"
                  :value="account.priority"
                  @change="onPriorityChange(account, $event)"
                />
              </label>
            </div>

            <!-- Actions -->
            <div class="mt-3 flex flex-wrap items-center gap-2">
              <Button
                icon="pi pi-check-circle"
                label="Set active"
                size="small"
                :severity="account.is_active ? 'secondary' : undefined"
                :disabled="account.is_active"
                @click="ctrl.setActive(account.id)"
              />
              <Button
                icon="pi pi-copy"
                label="Copy token"
                size="small"
                severity="secondary"
                outlined
                @click="ctrl.copyToken(account.id)"
              />
              <Button
                icon="pi pi-exclamation-triangle"
                label="Mark exhausted"
                size="small"
                severity="warn"
                text
                title="Báo account hết usage → auto-switch"
                @click="ctrl.reportExhausted(account.id)"
              />
            </div>
          </div>
        </div>
      </div>
    </template>

    <div v-else class="flex flex-1 items-center justify-center rounded-lg border border-dashed border-divider bg-panel/50 p-12">
      <p class="text-sm text-muted">
        {{ ctrl.isLoading.value ? "Loading accounts..." : "No accounts yet. Click \"Add Account\" to register one." }}
      </p>
    </div>

    <!-- Add account dialog -->
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
          <span class="text-xs font-bold text-muted">Provider</span>
          <div class="mt-1 flex gap-2">
            <Button
              label="Claude"
              size="small"
              :severity="provider === 'claude' ? undefined : 'secondary'"
              :outlined="provider !== 'claude'"
              @click="provider = 'claude'"
            />
            <Button
              label="Codex"
              size="small"
              :severity="provider === 'codex' ? undefined : 'secondary'"
              :outlined="provider !== 'codex'"
              @click="provider = 'codex'"
            />
          </div>
        </label>
        <label class="block">
          <span class="text-xs font-bold text-muted">Account Name <span class="text-red-500">*</span></span>
          <InputText v-model="accountName" class="mt-1 w-full" placeholder="e.g. team-claude" autofocus />
        </label>
        <label class="block">
          <span class="text-xs font-bold text-muted">API Key / Token <span class="text-red-500">*</span></span>
          <div class="relative mt-1">
            <InputText
              v-model="apiKey"
              :type="showApiKey ? 'text' : 'password'"
              class="w-full pr-10"
              placeholder="sk-..."
              autocomplete="off"
            />
            <Button
              :icon="`pi ${showApiKey ? 'pi-eye-slash' : 'pi-eye'}`"
              text
              rounded
              size="small"
              class="absolute right-2 top-1/2 -translate-y-1/2"
              :title="showApiKey ? 'Hide API key' : 'Show API key'"
              @click="showApiKey = !showApiKey"
            />
          </div>
        </label>
        <p class="text-xs text-muted">Loại tài khoản (API / Admin / OAuth) được tự nhận từ prefix của key.</p>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="isDialogOpen = false" />
          <Button
            :label="ctrl.isSaving.value ? 'Saving...' : 'Save'"
            :disabled="!accountName.trim() || !apiKey.trim() || ctrl.isSaving.value"
            @click="saveAccount"
          />
        </div>
      </template>
    </Dialog>

    <!-- Settings dialog -->
    <Dialog
      :visible="showSettings"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="showSettings = $event"
    >
      <template #header>
        <h3 class="font-bold text-ink">Auto-switch settings</h3>
      </template>

      <div class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Ngưỡng switch (% còn lại)</span>
          <input
            v-model="thresholdInput"
            type="number"
            min="0"
            max="100"
            class="mt-1 w-full rounded border border-divider bg-canvas px-3 py-2 text-ink"
          />
          <span class="text-xs text-muted">Dưới ngưỡng này account bị coi là "low" và sẽ được thay bằng account ưu tiên kế tiếp.</span>
        </label>
        <label class="block">
          <span class="text-xs font-bold text-muted">Chu kỳ poll (giây)</span>
          <input
            v-model="intervalInput"
            type="number"
            min="15"
            class="mt-1 w-full rounded border border-divider bg-canvas px-3 py-2 text-ink"
          />
          <span class="text-xs text-muted">Tối thiểu 15 giây.</span>
        </label>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="showSettings = false" />
          <Button label="Save" @click="saveSettings" />
        </div>
      </template>
    </Dialog>
  </div>
</template>
