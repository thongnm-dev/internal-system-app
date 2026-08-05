<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    /** % còn lại (0–100) từ backend — component tự đổi sang % đã dùng để bar tăng dần 0→100%. */
    remainingPercent: number;
    /** Thời điểm reset (`YYYY-MM-DD HH:MM:SS`). Bỏ trống (undefined) → không hiển thị dòng reset. */
    resetAt?: string;
    size?: "sm" | "md";
  }>(),
  { resetAt: undefined, size: "sm" },
);

function usedPercent(remainingPercent: number): number {
  return Math.min(100, Math.max(0, 100 - remainingPercent));
}

function usageBarClass(usedPercentValue: number): string {
  if (usedPercentValue >= 90) return "bg-red-500";
  if (usedPercentValue >= 70) return "bg-amber-500";
  return "bg-brand";
}

/** Diễn giải reset_at (`YYYY-MM-DD HH:MM:SS`) thành chuỗi ngắn, vd "còn 2h 15m · 11:10". */
function resetHint(resetAt: string): string {
  const raw = resetAt?.trim();
  if (!raw) return "—";
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
  <div>
    <div class="flex items-center justify-between" :class="size === 'md' ? 'text-xs' : 'text-[11px]'">
      <span class="font-bold text-muted">{{ label }}</span>
      <span class="font-bold text-ink">{{ Math.round(usedPercent(remainingPercent)) }}%</span>
    </div>
    <div class="overflow-hidden rounded-full bg-canvas" :class="size === 'md' ? 'mt-1.5 h-2' : 'mt-1 h-1.5'">
      <div
        :class="['h-full rounded-full transition-all', usageBarClass(usedPercent(remainingPercent))]"
        :style="{ width: `${usedPercent(remainingPercent)}%` }"
      />
    </div>
    <p v-if="resetAt !== undefined" class="mt-1 flex items-center gap-1 text-[11px] text-muted">
      <i class="pi pi-clock" />reset {{ resetHint(resetAt) }}
      <slot name="tag" />
    </p>
  </div>
</template>
