<script setup lang="ts">
import { ref, onMounted } from "vue";
import Popover from "primevue/popover";
import InputSwitch from "primevue/inputswitch";
import { useAuthStore } from "@/app/stores/auth";
import { useMenuStore } from "@/app/stores/menu";
import { useSettings } from "@/features/settings/composables/useSettings";
import { useAppUpdater } from "@/shared/composables/useAppUpdater";
import { useNetworkStatus } from "@/shared/composables/useNetworkStatus";
import type { MenuKey } from "@/_/types/app";
import type { SystemInfo } from "@/_/types/system";

const props = defineProps<{
  info: SystemInfo;
  isSidebarCollapsed: boolean;
}>();

const emit = defineEmits<{
  logout: [];
  toggleSidebar: [];
  menuChange: [key: MenuKey];
}>();

const auth = useAuthStore();
const menu = useMenuStore();
const { settings, updateTheme, updateLanguage, updateTabMode } = useSettings();

const languageOptions = [
  { label: "Vietnamese", value: "vi" as const },
  { label: "English", value: "en" as const },
  { label: "Japanese", value: "ja" as const },
];
const updater = useAppUpdater();
const network = useNetworkStatus();
const userMenu = ref<InstanceType<typeof Popover>>();

const langTrigger = ref<HTMLElement>();
const langSubmenuOpen = ref(false);
const langSubmenuStyle = ref({ top: "0px", left: "0px" });
let langCloseTimer: number | undefined;

function openLangSubmenu(): void {
  if (langCloseTimer !== undefined) {
    window.clearTimeout(langCloseTimer);
    langCloseTimer = undefined;
  }
  const rect = langTrigger.value?.getBoundingClientRect();
  if (!rect) return;
  langSubmenuStyle.value = { top: `${rect.top}px`, left: `${rect.right + 4}px` };
  langSubmenuOpen.value = true;
}

function scheduleCloseLangSubmenu(): void {
  langCloseTimer = window.setTimeout(() => {
    langSubmenuOpen.value = false;
  }, 150);
}

function cancelCloseLangSubmenu(): void {
  if (langCloseTimer !== undefined) {
    window.clearTimeout(langCloseTimer);
    langCloseTimer = undefined;
  }
}

function onToggleAutoCheck(enabled: boolean): void {
  network.setAutoCheck(enabled);
}

function onToggleTabMode(enabled: boolean): void {
  updateTabMode(enabled);
}

onMounted(() => {
  updater.startPolling();
});

function toggleUserMenu(event: Event): void {
  userMenu.value?.toggle(event);
}

function toggleTheme(): void {
  updateTheme(settings.value.theme === "dark" ? "light" : "dark");
}

function handleLogout(): void {
  userMenu.value?.hide();
  emit("logout");
}

function handleSettings(): void {
  userMenu.value?.hide();
  emit("menuChange", (menu.settingsMenu?.key ?? "settings") as MenuKey);
}

function selectLanguage(value: (typeof languageOptions)[number]["value"]): void {
  langSubmenuOpen.value = false;
  userMenu.value?.hide();
  updateLanguage(value);
}

function onUpdateClick(): void {
  if (updater.status.value === "ready") {
    void updater.install();
  }
}

function onCheckUpdate(): void {
  void updater.checkNow();
}

function formatDateTime(value: string): string {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2}) (\d{2}:\d{2}:\d{2})$/);
  if (match) {
    return `${match[1]}/${match[2]}/${match[3]} ${match[4]}`;
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  const pad = (part: number) => part.toString().padStart(2, "0");
  return `${date.getFullYear()}/${pad(date.getMonth() + 1)}/${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}
</script>

<template>
  <footer
    class="flex items-center gap-6 overflow-hidden border-t border-divider px-4 py-2 text-sm text-muted"
  >
    <button
      type="button"
      class="status-item flex cursor-pointer items-center gap-2 rounded hover:text-brand"
      :title="isSidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'"
      @click="emit('toggleSidebar')"
    >
      <i :class="['pi shrink-0 text-brand', isSidebarCollapsed ? 'pi-chevron-right' : 'pi-chevron-left']" />
    </button>

    <div class="mx-0.5 h-4 w-px bg-divider" />

    <button
      type="button"
      class="status-item flex cursor-pointer items-center gap-2 rounded hover:text-brand"
      title="User menu"
      @click="toggleUserMenu"
    >
      <i class="pi pi-user shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ auth.user?.full_name || auth.user?.username || '-' }}</strong>
    </button>

    <div class="mx-0.5 h-4 w-px bg-divider" />
    <span class="status-item flex items-center gap-2" title="Date time">
      <i class="pi pi-clock shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ formatDateTime(props.info.timestamp) }}</strong>
    </span>

    <div class="mx-0.5 h-4 w-px bg-divider" />
    
    <span
      class="status-item flex items-center gap-2"
      title="Tự động kiểm tra kết nối mạng"
    >
      <InputSwitch
        :model-value="network.autoCheckEnabled.value"
        @update:model-value="onToggleAutoCheck"
      />
      <span class="min-w-0 truncate">Tự động kiểm tra mạng</span>
    </span>

    <span
      class="status-item flex items-center gap-2"
      title="Open pages in tabs to switch between them without losing state"
    >
      <InputSwitch
        :model-value="settings.tabMode"
        @update:model-value="onToggleTabMode"
      />
      <span class="min-w-0 truncate">Tabs</span>
    </span>

    <template v-if="updater.isTauri">
      <span
        v-if="updater.status.value === 'checking'"
        class="status-item ml-auto flex items-center gap-2"
        title="Đang kiểm tra bản cập nhật"
      >
        <i class="pi pi-spin pi-spinner shrink-0 text-brand" />
        <span class="min-w-0 truncate">Đang kiểm tra cập nhật…</span>
      </span>

      <span
        v-else-if="updater.status.value === 'downloading'"
        class="status-item ml-auto flex items-center gap-2"
        :title="`Đang tải bản cập nhật ${updater.version.value ?? ''}`"
      >
        <i class="pi pi-spin pi-spinner shrink-0 text-brand" />
        <span class="min-w-0 truncate">
          Đang tải bản cập nhật
          <template v-if="updater.downloadPercent.value !== null">
            {{ updater.downloadPercent.value }}%
          </template>
          <template v-else>…</template>
        </span>
      </span>

      <button
        v-else-if="updater.status.value === 'ready'"
        type="button"
        class="status-item update-ready ml-auto flex items-center gap-2 rounded font-medium text-brand hover:underline"
        title="Nhấn để cài đặt bản cập nhật"
        @click="onUpdateClick"
      >
        <i class="pi pi-download shrink-0" />
        <span class="min-w-0 truncate">Bản cập nhật sẵn sàng.</span>
      </button>

      <span
        v-else-if="updater.status.value === 'installing'"
        class="status-item ml-auto flex items-center gap-2"
        title="Đang cài đặt bản cập nhật"
      >
        <i class="pi pi-spin pi-spinner shrink-0 text-brand" />
        <span class="min-w-0 truncate">Đang cài đặt…</span>
      </span>

      <button
        v-else-if="updater.status.value === 'error'"
        type="button"
        class="status-item ml-auto flex items-center gap-2 rounded text-red-600 hover:underline"
        :title="updater.errorMessage.value ?? 'Không thể cập nhật'"
        @click="onCheckUpdate"
      >
        <i class="pi pi-exclamation-triangle shrink-0" />
        <span class="min-w-0 truncate">Cập nhật thất bại. Thử lại</span>
      </button>

      <button
        v-else
        type="button"
        class="status-item ml-auto flex items-center gap-2 rounded text-muted hover:text-brand"
        title="Kiểm tra bản cập nhật"
        @click="onCheckUpdate"
      >
        <i class="pi pi-sync shrink-0" />
        <span class="min-w-0 truncate">Kiểm tra cập nhật</span>
      </button>
    </template>

    <span
      class="status-item flex items-center gap-2"
      :class="updater.isTauri ? '' : 'ml-auto'"
      title="Version"
    >
      <i class="pi pi-desktop shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ props.info.version }}</strong>
    </span>
  </footer>
  
  <Popover ref="userMenu" @hide="langSubmenuOpen = false">
    <div class="flex min-w-[160px] flex-col gap-0.5 py-1">
      <button v-if="menu.settingsMenu" type="button" class="ctx-menu-item" @click="handleSettings">
        <i :class="`pi ${menu.settingsMenu.icon}`" />
        <span>Profile</span>
      </button>
      <div class="my-1 border-t border-divider" />
      <button type="button" class="ctx-menu-item" @click="toggleTheme">
        <i :class="['pi', settings.theme === 'dark' ? 'pi-sun' : 'pi-moon']" />
        <span>{{ settings.theme === 'dark' ? 'Light mode' : 'Dark mode' }}</span>
      </button>

      <div
        ref="langTrigger"
        @mouseenter="openLangSubmenu"
        @mouseleave="scheduleCloseLangSubmenu"
      >
        <button type="button" class="ctx-menu-item justify-between" :class="{ 'bg-canvas text-brand': langSubmenuOpen }">
          <span class="flex items-center gap-2">
            <i class="pi pi-language" />
            <span>Language</span>
          </span>
          <i class="pi pi-angle-right text-xs" />
        </button>
      </div>
      <div class="my-1 border-t border-divider" />
      <button type="button" class="ctx-menu-item-danger" @click="handleLogout">
        <i class="pi pi-sign-out" />
        <span>Logout</span>
      </button>
    </div>
  </Popover>

  <Teleport to="body">
    <div
      v-if="langSubmenuOpen"
      :style="{ position: 'fixed', top: langSubmenuStyle.top, left: langSubmenuStyle.left, zIndex: 99999 }"
      class="flex min-w-[160px] flex-col gap-0.5 rounded-md border border-divider bg-panel py-1 shadow-lg"
      @mouseenter="cancelCloseLangSubmenu"
      @mouseleave="scheduleCloseLangSubmenu"
    >
      <button
        v-for="opt in languageOptions"
        :key="opt.value"
        type="button"
        class="ctx-menu-item"
        @click="selectLanguage(opt.value)"
      >
        <i :class="['pi', settings.language === opt.value ? 'pi-check-circle text-brand' : 'pi-circle text-muted']" />
        <span>{{ opt.label }}</span>
      </button>
    </div>
  </Teleport>
</template>
