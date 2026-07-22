<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { open } from "@tauri-apps/plugin-dialog";
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
/** Kiểu account đang thêm: subscription (tool tự capture token) hay API key. */
const accountKind = ref<"subscription" | "key">("subscription");
/** Với subscription: capture login hiện tại, hay đăng ký từ một config dir khác. */
const subMode = ref<"current" | "dir">("current");
const configDir = ref("");

const showSettings = ref(false);
const thresholdInput = ref(10);
const intervalInput = ref(60);

const showTerminal = ref(false);
const terminalConfigDir = ref("");
const terminalWorkDir = ref("");
const terminalIsLogin = ref(false);

const showDetect = ref(false);

/** Mở dialog dò login local (dò trước rồi hiện). */
async function openDetect() {
  const ok = await ctrl.detectLocal();
  if (ok) showDetect.value = true;
}

/** Đổi provider — Claude luôn là subscription, Codex chỉ API key. */
function selectProvider(next: AiProvider) {
  provider.value = next;
  accountKind.value = next === "claude" ? "subscription" : "key";
  if (next === "claude" && subMode.value === "current") void loadPreviewAndPrefill();
}

/** Nạp login hiện tại + prefill tên (chỉ khi tên còn trống). */
async function loadPreviewAndPrefill() {
  await ctrl.loadCapturePreview();
  const preview = ctrl.capturePreview.value;
  if (preview && !accountName.value.trim()) {
    accountName.value = preview.display_name || preview.email;
  }
}

/** Đổi giữa "capture login hiện tại" và "đăng ký config dir khác". */
function selectSubMode(mode: "current" | "dir") {
  subMode.value = mode;
  if (mode === "current") void loadPreviewAndPrefill();
}

/** Mở dialog chọn folder → điền vào config dir. */
async function browseConfigDir() {
  const selected = await open({ directory: true, title: "Chọn CLAUDE_CONFIG_DIR" });
  if (typeof selected === "string") {
    configDir.value = selected;
    await onConfigDirInput();
  }
}

/** Preview login tại config dir + prefill tên. */
async function onConfigDirInput() {
  const dir = configDir.value.trim();
  if (!dir) return;
  await ctrl.previewConfigDir(dir);
  const preview = ctrl.configDirPreview.value;
  if (preview && !accountName.value.trim()) {
    accountName.value = preview.display_name || preview.email;
  }
}

/** Mở terminal dialog để login tại config dir hiện tại. */
function openLoginForConfigDir() {
  const dir = configDir.value.trim();
  if (!dir) return;
  terminalConfigDir.value = dir;
  terminalWorkDir.value = "";
  terminalIsLogin.value = true;
  showTerminal.value = true;
}

/** Re-check login sau khi đã login xong trong terminal. */
async function recheckConfigDir() {
  await onConfigDirInput();
}

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
  accountKind.value = "subscription";
  subMode.value = "current";
  configDir.value = "";
  ctrl.capturePreview.value = null;
  ctrl.configDirPreview.value = null;
  isDialogOpen.value = true;
  void loadPreviewAndPrefill();
}

/** Form hợp lệ: subscription cần đọc được login (hiện tại cần token; config dir chỉ cần identity). */
const canSaveAccount = computed(() => {
  if (!accountName.value.trim()) return false;
  if (accountKind.value !== "subscription") return !!apiKey.value.trim();
  return subMode.value === "current"
    ? !!ctrl.capturePreview.value?.has_token
    : !!ctrl.configDirPreview.value;
});

async function saveAccount() {
  if (!canSaveAccount.value) return;
  if (accountKind.value === "subscription") {
    // Subscription: capture login hiện tại, hoặc đăng ký từ một config dir khác.
    const ok =
      subMode.value === "current"
        ? await ctrl.captureAdd(accountName.value.trim())
        : await ctrl.addConfigDir(configDir.value.trim(), accountName.value.trim());
    if (ok) isDialogOpen.value = false;
    return;
  }
  const ok = await ctrl.addAccount({
    name: accountName.value.trim(),
    provider: provider.value,
    api_key: apiKey.value.trim(),
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
    poll_interval_secs: Math.max(60, Number(intervalInput.value) || 300),
  });
  if (ok) showSettings.value = false;
}

function openTerminalDialog(account: AiAccount) {
  terminalConfigDir.value = account.config_dir || "";
  terminalWorkDir.value = "";
  terminalIsLogin.value = false;
  showTerminal.value = true;
}

async function browseTerminalWorkDir() {
  const selected = await open({ directory: true, title: "Chọn working directory (thư mục project)" });
  if (typeof selected === "string") terminalWorkDir.value = selected;
}

async function confirmOpenTerminal() {
  const fn = terminalIsLogin.value ? ctrl.openLogin : ctrl.openTerminal;
  const ok = await fn(terminalConfigDir.value, terminalWorkDir.value);
  if (ok) showTerminal.value = false;
}

function onPriorityChange(account: AiAccount, event: Event) {
  const value = Math.max(1, Number((event.target as HTMLInputElement).value) || 1);
  if (value !== account.priority) {
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
    case "subscription":
      return "Subscription";
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
    case "subscription":
      return "bg-sky-100 text-sky-700";
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

/**
 * Diễn giải thời điểm reset (`YYYY-MM-DD HH:MM:SS`) thành chuỗi thân thiện,
 * ví dụ "còn 2h 15m · 11:10". Trả `"—"` nếu chưa có số liệu.
 */
function resetHint(resetAt: string): string {
  const raw = resetAt?.trim();
  if (!raw) return "—";
  // Chuỗi backend là giờ local; thêm khoảng trắng → ISO để Date parse ổn định.
  const target = new Date(raw.replace(" ", "T"));
  if (Number.isNaN(target.getTime())) return raw;
  const diffMs = target.getTime() - Date.now();
  const clock = raw.slice(11, 16) || "";
  if (diffMs <= 0) return `sắp reset · ${clock}`;
  const mins = Math.round(diffMs / 60000);
  const days = Math.floor(mins / 1440);
  const hours = Math.floor((mins % 1440) / 60);
  const rem = mins % 60;
  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (days === 0 && rem > 0) parts.push(`${rem}m`);
  const rel = parts.length ? parts.join(" ") : "<1m";
  return `còn ${rel} · ${clock}`;
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
            icon="pi pi-search"
            label="Detect local"
            severity="secondary"
            :loading="ctrl.isDetecting.value"
            title="Dò các login Claude đã đăng nhập trên máy"
            @click="openDetect"
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
                  <span
                    v-if="account.account_type !== 'subscription'"
                    :class="['shrink-0 rounded-full px-2 py-0.5 text-[11px] font-bold', typeBadgeClass(account.account_type)]"
                  >
                    {{ typeLabel(account.account_type) }}
                  </span>
                  <span
                    v-if="account.subscription_type"
                    class="shrink-0 rounded-full bg-canvas px-2 py-0.5 text-[11px] font-bold text-muted"
                  >
                    {{ account.subscription_type }}
                  </span>
                </div>
                <template v-if="account.account_type === 'subscription'">
                  <p v-if="account.email" class="mt-0.5 truncate text-xs text-muted" :title="account.email">
                    <i class="pi pi-envelope mr-1" />{{ account.email }}
                  </p>
                  <p class="mt-0.5 truncate font-mono text-[11px] text-muted" :title="account.config_dir">
                    <i class="pi pi-folder mr-1" />{{ account.config_dir || "—" }}
                  </p>
                </template>
                <p v-else class="mt-0.5 font-mono text-xs text-muted">{{ account.api_key_masked }}</p>
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

            <!-- Usage remaining (API / Codex — số liệu tổng hợp) -->
            <div v-if="account.account_type !== 'subscription'" class="mt-3">
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

            <!-- Subscription: session (5h) + weekly (7 ngày) từ OAuth usage endpoint -->
            <div v-else class="mt-3 space-y-3">
              <!-- Current session -->
              <div>
                <div class="flex items-center justify-between text-xs">
                  <span class="font-bold text-muted">Current session</span>
                  <span class="font-bold text-ink">{{ Math.round(account.session_percent) }}%</span>
                </div>
                <div class="mt-1.5 h-2 overflow-hidden rounded-full bg-canvas">
                  <div
                    :class="['h-full rounded-full transition-all', usageBarClass(account.session_percent)]"
                    :style="{ width: `${Math.min(100, Math.max(0, account.session_percent))}%` }"
                  />
                </div>
                <p class="mt-1 flex items-center gap-1 text-[11px] text-muted">
                  <i class="pi pi-clock" />reset {{ resetHint(account.session_reset_at) }}
                </p>
              </div>

              <!-- Weekly limit -->
              <div>
                <div class="flex items-center justify-between text-xs">
                  <span class="font-bold text-muted">Weekly limit</span>
                  <span class="font-bold text-ink">{{ Math.round(account.weekly_percent) }}%</span>
                </div>
                <div class="mt-1.5 h-2 overflow-hidden rounded-full bg-canvas">
                  <div
                    :class="['h-full rounded-full transition-all', usageBarClass(account.weekly_percent)]"
                    :style="{ width: `${Math.min(100, Math.max(0, account.weekly_percent))}%` }"
                  />
                </div>
                <p class="mt-1 flex items-center gap-1 text-[11px] text-muted">
                  <i class="pi pi-clock" />reset {{ resetHint(account.weekly_reset_at) }}
                </p>
              </div>

              <p
                v-if="!account.session_reset_at && !account.weekly_reset_at"
                class="text-[11px] text-muted"
              >
                Chưa có số liệu — bấm Refresh để đọc usage (login còn hạn).
              </p>
              <p v-if="account.last_checked_at" class="flex items-center gap-1 text-[11px] text-muted">
                <i class="pi pi-sync" />cập nhật {{ account.last_checked_at }}
              </p>
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
                  min="1"
                  class="w-14 rounded border border-divider bg-canvas px-1.5 py-0.5 text-right text-ink"
                  :value="account.priority"
                  @change="onPriorityChange(account, $event)"
                />
              </label>
            </div>

            <!-- Actions -->
            <div class="mt-3 flex flex-wrap items-center gap-2">
              <Button
                icon="pi pi-refresh"
                label="Refresh"
                size="small"
                severity="secondary"
                :loading="ctrl.refreshingId.value === account.id"
                @click="ctrl.refreshAccount(account.id)"
              />
              <Button
                icon="pi pi-check-circle"
                label="Set active"
                size="small"
                :severity="account.is_active ? 'secondary' : undefined"
                :disabled="account.is_active"
                @click="ctrl.setActive(account.id)"
              />
              <Button
                v-if="account.account_type !== 'subscription'"
                icon="pi pi-copy"
                label="Copy token"
                size="small"
                severity="secondary"
                outlined
                @click="ctrl.copyToken(account.id)"
              />
              <Button
                v-if="account.account_type === 'subscription'"
                icon="pi pi-terminal"
                label="Open terminal"
                size="small"
                severity="secondary"
                outlined
                @click="openTerminalDialog(account)"
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
              @click="selectProvider('claude')"
            />
            <Button
              label="Codex"
              size="small"
              :severity="provider === 'codex' ? undefined : 'secondary'"
              :outlined="provider !== 'codex'"
              @click="selectProvider('codex')"
            />
          </div>
        </label>

        <label class="block">
          <span class="text-xs font-bold text-muted">Account Name <span class="text-red-500">*</span></span>
          <InputText v-model="accountName" class="mt-1 w-full" placeholder="e.g. personal-claude" autofocus />
        </label>

        <!-- Subscription: tool tự capture token của login Claude đang active -->
        <div v-if="accountKind === 'subscription'" class="block space-y-2">
          <!-- Nguồn login: capture hiện tại hay từ config dir khác (thêm acc thứ 2) -->
          <div class="flex gap-2">
            <Button
              label="Login hiện tại"
              size="small"
              :severity="subMode === 'current' ? undefined : 'secondary'"
              :outlined="subMode !== 'current'"
              @click="selectSubMode('current')"
            />
            <Button
              label="Config dir khác"
              size="small"
              :severity="subMode === 'dir' ? undefined : 'secondary'"
              :outlined="subMode !== 'dir'"
              @click="selectSubMode('dir')"
            />
          </div>

          <!-- Config dir input (đăng ký account thứ 2 đã login ở dir riêng) -->
          <label v-if="subMode === 'dir'" class="block">
            <span class="text-xs font-bold text-muted">CLAUDE_CONFIG_DIR <span class="text-red-500">*</span></span>
            <div class="mt-1 flex gap-1.5">
              <InputText
                v-model="configDir"
                class="min-w-0 flex-1 font-mono"
                placeholder="~/.claude-work"
                @input="onConfigDirInput"
              />
              <Button
                icon="pi pi-folder-open"
                severity="secondary"
                title="Chọn folder"
                @click="browseConfigDir"
              />
            </div>
            <span class="text-xs text-muted">
              Login trước 1 lần: <code class="rounded bg-canvas px-1">CLAUDE_CONFIG_DIR=&lt;dir&gt; claude /login</code>
            </span>
          </label>

          <!-- Preview login đọc được (từ login hiện tại hoặc config dir) -->
          <div
            v-if="subMode === 'current' ? ctrl.capturePreview.value?.has_token : ctrl.configDirPreview.value"
            class="rounded-lg border border-divider bg-canvas/50 p-3 text-xs"
          >
            <div class="flex items-center gap-2">
              <i class="pi pi-user text-muted" />
              <span class="truncate font-semibold text-ink">
                {{ (subMode === 'current' ? ctrl.capturePreview.value : ctrl.configDirPreview.value)?.email }}
              </span>
              <span
                v-if="(subMode === 'current' ? ctrl.capturePreview.value : ctrl.configDirPreview.value)?.subscription_type"
                class="shrink-0 rounded-full bg-sky-100 px-2 py-0.5 text-[11px] font-bold text-sky-700"
              >
                {{ (subMode === 'current' ? ctrl.capturePreview.value : ctrl.configDirPreview.value)?.subscription_type }}
              </span>
            </div>
            <p
              v-if="(subMode === 'current' ? ctrl.capturePreview.value : ctrl.configDirPreview.value)?.token_expires_at"
              class="mt-1 text-[11px] text-muted"
            >
              <i class="pi pi-clock mr-1" />token hết hạn
              {{ (subMode === 'current' ? ctrl.capturePreview.value : ctrl.configDirPreview.value)?.token_expires_at }}
            </p>
            <p class="mt-1.5 text-[11px] text-muted">
              {{ subMode === 'current'
                ? 'Tool sẽ lưu token này vào profile riêng trong app data dir.'
                : 'Token đọc từ Keychain của config dir này (luôn mới).' }}
            </p>
            <p
              v-if="subMode === 'dir' && ctrl.configDirPreview.value && !ctrl.configDirPreview.value.has_token"
              class="mt-2 rounded border border-amber-300 bg-amber-50 px-2 py-1.5 text-[11px] text-amber-700"
            >
              <i class="pi pi-exclamation-triangle mr-1" />Chưa có token — account sẽ được thêm nhưng chưa dùng được.
              Chạy <code class="rounded bg-amber-100 px-1">CLAUDE_CONFIG_DIR=&lt;dir&gt; claude /login</code> để lấy token.
            </p>
          </div>
          <div
            v-else
            class="rounded-lg border border-dashed border-amber-300 bg-amber-50 p-3 text-xs text-amber-700"
          >
            <p>
              {{ subMode === 'current'
                ? 'Chưa có login Claude đang hoạt động. Chạy `claude /login` rồi mở lại dialog.'
                : 'Chưa đọc được login ở config dir này.' }}
            </p>
            <div v-if="subMode === 'dir' && configDir.trim()" class="mt-2 flex gap-2">
              <Button
                icon="pi pi-terminal"
                label="Mở terminal để login"
                size="small"
                severity="warn"
                @click="openLoginForConfigDir"
              />
              <Button
                icon="pi pi-refresh"
                label="Kiểm tra lại"
                size="small"
                severity="secondary"
                @click="recheckConfigDir"
              />
            </div>
          </div>
        </div>

        <!-- API key -->
        <label v-else class="block">
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
          <span class="text-xs text-muted">Loại (API / Admin / OAuth) được tự nhận từ prefix của key.</span>
        </label>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="isDialogOpen = false" />
          <Button
            :label="ctrl.isSaving.value || ctrl.isCapturing.value ? 'Đang lưu...' : 'Save'"
            :disabled="!canSaveAccount || ctrl.isSaving.value || ctrl.isCapturing.value"
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
            min="60"
            class="mt-1 w-full rounded border border-divider bg-canvas px-3 py-2 text-ink"
          />
          <span class="text-xs text-muted">
            Tối thiểu 60 giây (khuyến nghị 300). Endpoint usage của Claude bị rate-limit nếu gọi quá dày.
          </span>
        </label>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="showSettings = false" />
          <Button label="Save" @click="saveSettings" />
        </div>
      </template>
    </Dialog>

    <!-- Detect local logins dialog -->
    <Dialog
      :visible="showDetect"
      class="w-full max-w-lg rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="showDetect = $event"
    >
      <template #header>
        <h3 class="font-bold text-ink">Login Claude trên máy</h3>
      </template>

      <div class="space-y-3">
        <p class="text-xs text-muted">
          Dò từ <code class="rounded bg-canvas px-1">.claude.json</code> + Keychain. Usage % không có ở
          local nên chỉ hiển thị email, loại subscription và hạn token.
        </p>

        <div
          v-for="login in ctrl.detected.value"
          :key="login.config_dir + login.email"
          class="rounded-lg border border-divider bg-canvas/50 p-3"
        >
          <div class="flex items-center gap-2">
            <i class="pi pi-user text-muted" />
            <span class="truncate font-semibold text-ink" :title="login.email">{{ login.email || "(no email)" }}</span>
            <span
              v-if="login.subscription_type"
              class="shrink-0 rounded-full bg-sky-100 px-2 py-0.5 text-[11px] font-bold text-sky-700"
            >
              {{ login.subscription_type }}
            </span>
            <span
              class="ml-auto shrink-0 rounded-full px-2 py-0.5 text-[11px] font-bold"
              :class="login.already_added ? 'bg-emerald-100 text-emerald-700' : 'bg-amber-100 text-amber-700'"
            >
              {{ login.already_added ? "Đã thêm" : "Mới" }}
            </span>
          </div>
          <div class="mt-1.5 grid gap-0.5 text-[11px] text-muted">
            <span v-if="login.display_name">{{ login.display_name }}</span>
            <span class="truncate font-mono" :title="login.config_dir">
              <i class="pi pi-folder mr-1" />{{ login.config_dir }}
            </span>
            <span v-if="login.token_expires_at">
              <i class="pi pi-clock mr-1" />token hết hạn {{ login.token_expires_at }}
            </span>
          </div>
        </div>

        <p
          v-if="!ctrl.detected.value.length"
          class="rounded-lg border border-dashed border-divider p-6 text-center text-sm text-muted"
        >
          Không tìm thấy login Claude nào trên máy.
        </p>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Đóng" severity="secondary" @click="showDetect = false" />
          <Button
            icon="pi pi-download"
            :label="ctrl.isDetecting.value ? 'Đang thêm...' : 'Thêm login mới'"
            :disabled="ctrl.isDetecting.value || !ctrl.detected.value.some((l) => !l.already_added)"
            @click="ctrl.importDetected()"
          />
        </div>
      </template>
    </Dialog>

    <!-- Open terminal dialog -->
    <Dialog
      :visible="showTerminal"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="showTerminal = $event"
    >
      <template #header>
        <h3 class="font-bold text-ink">Open terminal</h3>
      </template>

      <div class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Working directory <span class="text-red-500">*</span></span>
          <div class="mt-1 flex gap-1.5">
            <input
              :value="terminalWorkDir"
              type="text"
              readonly
              placeholder="Chọn thư mục project..."
              class="min-w-0 flex-1 rounded border border-divider bg-canvas px-3 py-2 font-mono text-sm text-ink"
            />
            <Button
              icon="pi pi-folder-open"
              severity="secondary"
              title="Chọn folder"
              @click="browseTerminalWorkDir"
            />
          </div>
          <span class="text-xs text-muted">Thư mục project nơi terminal sẽ mở.</span>
        </label>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="showTerminal = false" />
          <Button label="Continue" :disabled="!terminalWorkDir.trim()" @click="confirmOpenTerminal" />
        </div>
      </template>
    </Dialog>
  </div>
</template>
