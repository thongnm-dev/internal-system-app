<script setup lang="ts">
import type { SystemInfo } from "@/shared/types/system";

const props = defineProps<{
  info: SystemInfo;
}>();

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
      <strong class="min-w-0 truncate text-ink">{{ props.info.username }}</strong>
    </span>
    <span class="status-item flex items-center gap-2" title="Date time">
      <i class="pi pi-clock shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ formatDateTime(props.info.timestamp) }}</strong>
    </span>
    <span class="status-item flex items-center gap-2" title="IP">
      <i class="pi pi-globe shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ props.info.ip_address }}</strong>
    </span>
    <span class="status-item ml-auto flex items-center gap-2" title="Version">
      <i class="pi pi-desktop shrink-0 text-brand" />
      <strong class="min-w-0 truncate text-ink">{{ props.info.version }}</strong>
    </span>
  </footer>
</template>
