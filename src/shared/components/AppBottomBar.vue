<script setup lang="ts">
import { onMounted } from "vue";
import { useAuthStore } from "@/app/stores/auth";
import { useAppUpdater } from "@/shared/composables/useAppUpdater";
import type { SystemInfo } from "@/_/types/system";

const props = defineProps<{
  info: SystemInfo;
}>();

const auth = useAuthStore();

const updater = useAppUpdater();

onMounted(() => {
  updater.startPolling();
});

function onUpdateClick(): void {
  if (updater.status.value === "ready") {
    void updater.install();
  }
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
    <span class="status-item flex items-center gap-2" title="Login">
      <i class="pi pi-user shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ auth.user?.full_name || auth.user?.username || '-' }}</strong>
    </span>
    <span class="status-item flex items-center gap-2" title="Date time">
      <i class="pi pi-clock shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ formatDateTime(props.info.timestamp) }}</strong>
    </span>
    <span class="status-item flex items-center gap-2" title="IP">
      <i class="pi pi-globe shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ props.info.ip_address }}</strong>
    </span>
    <!-- Trạng thái cập nhật: đang tải (%) / sẵn sàng cài / lỗi -->
    <template v-if="updater.isActive.value || updater.status.value === 'error'">
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

      <span
        v-else-if="updater.status.value === 'error'"
        class="status-item ml-auto flex items-center gap-2 text-red-600"
        :title="updater.errorMessage.value ?? 'Không thể cập nhật'"
      >
        <i class="pi pi-exclamation-triangle shrink-0" />
        <span class="min-w-0 truncate">Cập nhật thất bại</span>
      </span>
    </template>

    <span
      class="status-item flex items-center gap-2"
      :class="updater.isActive.value || updater.status.value === 'error' ? '' : 'ml-auto'"
      title="Version"
    >
      <i class="pi pi-desktop shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ props.info.version }}</strong>
    </span>
  </footer>
</template>
