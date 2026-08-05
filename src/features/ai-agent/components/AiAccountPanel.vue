<script setup lang="ts">
import { computed } from "vue";
import Fieldset from "primevue/fieldset";
import RadioButton from "primevue/radiobutton";
import Select from "primevue/select";
import AiUsageMeter from "./AiUsageMeter.vue";
import { AI_ACCOUNT_STATUS_META, AI_ACCOUNT_TYPE_META, AI_USAGE_WINDOW_LABEL } from "@/_/types/ai-usage";
import type { AiAccount } from "@/_/types/ai-usage";
import type { AiModelResult } from "@/tauri/commands/ai-workflow";

const props = withDefaults(
  defineProps<{
    accounts: AiAccount[];
    isLoading: boolean;
    /** id account đang được set active (đang gọi API) — disable radio + hiện spinner. */
    settingActiveId: number | null;
    /** Model khả dụng (theo provider) cho 1 account — trả về raw, component tự format label. */
    modelOptionsFor: (account: AiAccount) => AiModelResult[];
    selectedModelIdFor: (account: AiAccount) => number | null;
    /** Tên model mặc định hiển thị trong placeholder, vd "Opus" hoặc "Sonnet". */
    defaultModelLabel: string;
    /** Số cột ở breakpoint xl (Cowork dùng 3, AI Translate Cowork dùng 4 vì không có cột Workflow). */
    columns?: 3 | 4;
  }>(),
  { columns: 3 },
);

const emit = defineEmits<{
  (e: "select-account", id: number): void;
  (e: "select-model", accountId: number, modelId: number | null): void;
}>();

const activeAccountId = computed(() => props.accounts.find((a) => a.is_active)?.id ?? null);

const gridClass = computed(() => (props.columns === 4 ? "sm:grid-cols-2 xl:grid-cols-4" : "sm:grid-cols-2 xl:grid-cols-3"));

function usagePercent(account: AiAccount): number {
  return account.usage_percent;
}

/** Còn usage để dùng — chỉ cho phép chọn active các account chưa cạn quota. */
function hasUsage(account: AiAccount): boolean {
  return account.status !== "exhausted" && usagePercent(account) > 0;
}

function selectAccount(id: number | null) {
  if (id == null || id === activeAccountId.value) return;
  emit("select-account", id);
}

/** Nhãn hiển thị model: "Opus 5" (viết hoa chữ đầu + version). */
function modelLabel(m: AiModelResult): string {
  const name = m.model.charAt(0).toUpperCase() + m.model.slice(1);
  return m.version ? `${name} ${m.version}` : name;
}

function modelOptions(account: AiAccount) {
  return props.modelOptionsFor(account).map((m) => ({ label: modelLabel(m), value: m.id }));
}

function selectAccountModel(account: AiAccount, modelId: number | null) {
  emit("select-model", account.id, modelId);
}

const modelSelectPt = {
  root: { class: "!bg-canvas !border-divider !min-h-0" },
  label: { class: "!text-[11px] !py-1" },
  option: { class: "!text-xs" },
};
</script>

<template>
  <Fieldset
    class="shrink-0 rounded-lg border border-divider bg-panel p-4 shadow-sm fieldset-nested"
    legend="Account AI"
    toggleable
  >
    <div class="mt-4 max-h-56 overflow-auto">
      <p v-if="isLoading" class="p-4 text-center text-xs text-muted">Loading accounts...</p>
      <div v-else-if="accounts.length" class="grid gap-3" :class="gridClass">
        <div
          v-for="account in accounts"
          :key="account.id"
          class="rounded-lg border p-3 transition-colors"
          :class="[
            account.is_active ? 'border-brand ring-1 ring-brand/40' : 'border-divider',
            hasUsage(account) ? 'cursor-pointer hover:border-brand/60' : 'opacity-60',
          ]"
          :title="hasUsage(account) ? 'Chọn account này để sử dụng' : 'Account đã hết usage'"
          @click="hasUsage(account) && selectAccount(account.id)"
        >
          <div class="flex flex-wrap items-center gap-2">
            <RadioButton
              :model-value="activeAccountId"
              :value="account.id"
              :disabled="!hasUsage(account) || settingActiveId !== null"
              :name="'ai-account'"
              class="shrink-0"
              @update:model-value="selectAccount"
              @click.stop
            />
            <i v-if="settingActiveId === account.id" class="pi pi-spinner pi-spin shrink-0 text-xs text-brand" />
            <span class="truncate font-semibold text-ink" :title="account.name">{{ account.name }}</span>
            <span :class="['shrink-0', AI_ACCOUNT_TYPE_META[account.account_type].badgeClass]">
              {{ AI_ACCOUNT_TYPE_META[account.account_type].label }}
            </span>
            <span :class="['ml-auto shrink-0', AI_ACCOUNT_STATUS_META[account.status].badgeClass]">
              {{ AI_ACCOUNT_STATUS_META[account.status].label }}
            </span>
          </div>

          <AiUsageMeter
            v-if="account.account_type === 'subscription'"
            class="mt-2"
            label="Current session"
            :remaining-percent="account.session_percent"
            :reset-at="account.session_reset_at"
          />

          <AiUsageMeter class="mt-2" label="Usage used" :remaining-percent="account.usage_percent" :reset-at="account.reset_at">
            <template #tag>
              <span
                v-if="AI_USAGE_WINDOW_LABEL[account.usage_window]"
                class="rounded-full bg-canvas px-1.5 py-0.5 text-[10px] font-bold text-muted"
              >
                {{ AI_USAGE_WINDOW_LABEL[account.usage_window] }}
              </span>
            </template>
          </AiUsageMeter>

          <label v-if="modelOptions(account).length" class="mt-2 block" @click.stop>
            <Select
              :model-value="selectedModelIdFor(account)"
              :options="modelOptions(account)"
              option-label="label"
              option-value="value"
              :placeholder="`Mặc định (${defaultModelLabel})`"
              class="mt-1 w-full"
              :pt="modelSelectPt"
              @update:model-value="(v) => selectAccountModel(account, v)"
            />
          </label>
        </div>
      </div>
      <p v-else class="p-4 text-center text-xs text-muted">Chưa có account AI nào. Thêm ở màn AI Usage.</p>
    </div>
  </Fieldset>
</template>
