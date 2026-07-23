<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";

const props = defineProps<{
  visible: boolean;
  selected?: string;
}>();

const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
  (e: "select", icon: string): void;
}>();

const search = ref("");

watch(
  () => props.visible,
  (v) => {
    if (v) search.value = "";
  },
);

const ICONS: string[] = [
  "pi-home", "pi-search", "pi-user", "pi-users", "pi-cog", "pi-lock", "pi-unlock", "pi-lock-open",
  "pi-key", "pi-shield", "pi-eye", "pi-eye-slash", "pi-bell", "pi-bell-slash",
  "pi-envelope", "pi-phone", "pi-mobile", "pi-tablet", "pi-desktop",
  "pi-inbox", "pi-send", "pi-paperclip", "pi-link", "pi-image", "pi-images", "pi-video", "pi-camera",
  "pi-file", "pi-file-o", "pi-file-edit", "pi-file-plus", "pi-file-check", "pi-file-export", "pi-file-import",
  "pi-file-excel", "pi-file-pdf", "pi-file-word", "pi-file-arrow-up",
  "pi-folder", "pi-folder-open", "pi-folder-plus",
  "pi-copy", "pi-clone", "pi-clipboard", "pi-save", "pi-print", "pi-download", "pi-upload",
  "pi-cloud", "pi-cloud-upload", "pi-cloud-download",
  "pi-pencil", "pi-pen-to-square", "pi-trash", "pi-eraser",
  "pi-plus", "pi-plus-circle", "pi-minus", "pi-minus-circle",
  "pi-check", "pi-check-circle", "pi-check-square", "pi-times", "pi-times-circle",
  "pi-exclamation-triangle", "pi-exclamation-circle", "pi-info", "pi-info-circle",
  "pi-question", "pi-question-circle", "pi-ban",
  "pi-arrow-up", "pi-arrow-down", "pi-arrow-left", "pi-arrow-right",
  "pi-arrow-circle-up", "pi-arrow-circle-down", "pi-arrow-circle-left", "pi-arrow-circle-right",
  "pi-arrow-up-right", "pi-arrow-up-left", "pi-arrow-down-right", "pi-arrow-down-left",
  "pi-chevron-up", "pi-chevron-down", "pi-chevron-left", "pi-chevron-right",
  "pi-angle-up", "pi-angle-down", "pi-angle-left", "pi-angle-right",
  "pi-angle-double-up", "pi-angle-double-down", "pi-angle-double-left", "pi-angle-double-right",
  "pi-caret-up", "pi-caret-down", "pi-caret-left", "pi-caret-right",
  "pi-chevron-circle-up", "pi-chevron-circle-down", "pi-chevron-circle-left", "pi-chevron-circle-right",
  "pi-sort", "pi-sort-up", "pi-sort-down", "pi-sort-alt", "pi-sort-alt-slash",
  "pi-sort-amount-up", "pi-sort-amount-down", "pi-sort-amount-up-alt", "pi-sort-amount-down-alt",
  "pi-sort-alpha-up", "pi-sort-alpha-down", "pi-sort-alpha-up-alt", "pi-sort-alpha-down-alt",
  "pi-sort-numeric-up", "pi-sort-numeric-down", "pi-sort-numeric-up-alt", "pi-sort-numeric-down-alt",
  "pi-sort-up-fill", "pi-sort-down-fill",
  "pi-star", "pi-star-fill", "pi-star-half", "pi-star-half-fill",
  "pi-heart", "pi-heart-fill", "pi-thumbs-up", "pi-thumbs-down", "pi-thumbs-up-fill", "pi-thumbs-down-fill",
  "pi-bookmark", "pi-bookmark-fill", "pi-flag", "pi-flag-fill",
  "pi-tag", "pi-tags", "pi-filter", "pi-filter-fill", "pi-filter-slash",
  "pi-calendar", "pi-calendar-plus", "pi-calendar-minus", "pi-calendar-times", "pi-calendar-clock",
  "pi-clock", "pi-hourglass", "pi-stopwatch", "pi-history",
  "pi-chart-bar", "pi-chart-line", "pi-chart-pie", "pi-chart-scatter",
  "pi-table", "pi-list", "pi-list-check", "pi-th-large", "pi-bars", "pi-sitemap",
  "pi-map", "pi-map-marker", "pi-compass", "pi-directions", "pi-directions-alt",
  "pi-globe", "pi-sun", "pi-moon", "pi-bolt", "pi-lightbulb", "pi-sparkles",
  "pi-palette", "pi-sliders-h", "pi-sliders-v",
  "pi-code", "pi-database", "pi-server", "pi-microchip", "pi-microchip-ai",
  "pi-wifi", "pi-sync", "pi-refresh", "pi-replay", "pi-undo",
  "pi-external-link", "pi-window-maximize", "pi-window-minimize", "pi-expand",
  "pi-power-off", "pi-sign-in", "pi-sign-out",
  "pi-user-edit", "pi-user-plus", "pi-user-minus", "pi-id-card", "pi-address-book",
  "pi-briefcase", "pi-building", "pi-building-columns", "pi-warehouse", "pi-shop",
  "pi-shopping-cart", "pi-shopping-bag", "pi-cart-plus", "pi-cart-minus", "pi-cart-arrow-down",
  "pi-credit-card", "pi-wallet", "pi-money-bill", "pi-dollar", "pi-euro", "pi-pound",
  "pi-indian-rupee", "pi-turkish-lira", "pi-bitcoin", "pi-ethereum", "pi-percentage",
  "pi-book", "pi-graduation-cap", "pi-trophy", "pi-crown", "pi-gift",
  "pi-wrench", "pi-hammer", "pi-calculator", "pi-gauge",
  "pi-truck", "pi-car", "pi-ticket", "pi-receipt", "pi-barcode", "pi-qrcode",
  "pi-box", "pi-circle", "pi-circle-fill", "pi-circle-on", "pi-circle-off",
  "pi-ellipsis-h", "pi-ellipsis-v", "pi-hashtag", "pi-at", "pi-asterisk",
  "pi-language", "pi-megaphone", "pi-microphone", "pi-headphones",
  "pi-volume-up", "pi-volume-down", "pi-volume-off",
  "pi-play", "pi-play-circle", "pi-pause", "pi-pause-circle", "pi-stop", "pi-stop-circle",
  "pi-eject", "pi-forward", "pi-backward", "pi-fast-forward", "pi-fast-backward",
  "pi-step-forward", "pi-step-backward", "pi-step-forward-alt", "pi-step-backward-alt",
  "pi-comment", "pi-comments", "pi-share-alt", "pi-reply",
  "pi-thumbtack", "pi-verified", "pi-delete-left", "pi-wave-pulse",
  "pi-spinner", "pi-spinner-dotted",
  "pi-objects-column", "pi-arrows-h", "pi-arrows-v", "pi-arrows-alt",
  "pi-arrow-right-arrow-left",
  "pi-arrow-up-right-and-arrow-down-left-from-center",
  "pi-arrow-down-left-and-arrow-up-right-to-center",
  "pi-search-plus", "pi-search-minus",
  "pi-align-left", "pi-align-center", "pi-align-right", "pi-align-justify",
  "pi-print", "pi-prime",
  "pi-android", "pi-apple", "pi-microsoft", "pi-google",
  "pi-facebook", "pi-twitter", "pi-instagram", "pi-linkedin",
  "pi-youtube", "pi-vimeo", "pi-github", "pi-discord", "pi-slack",
  "pi-whatsapp", "pi-telegram", "pi-reddit", "pi-pinterest",
  "pi-twitch", "pi-tiktok", "pi-paypal", "pi-amazon",
  "pi-face-smile", "pi-mars", "pi-venus",
  "pi-bullseye", "pi-equals",
];

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return ICONS;
  return ICONS.filter((icon) => icon.includes(q));
});

function pick(icon: string) {
  emit("select", icon);
  emit("update:visible", false);
}
</script>

<template>
  <Dialog
    :visible="props.visible"
    class="w-full max-w-2xl rounded-lg bg-panel shadow-xl"
    :closable="true"
    modal
    @update:visible="emit('update:visible', $event)"
  >
    <template #header>
      <h3 class="font-bold text-ink">Choose Icon</h3>
    </template>

    <div class="space-y-3">
      <span class="flex items-center gap-2 rounded-md border border-divider bg-canvas px-3">
        <i class="pi pi-search text-xs text-muted" />
        <InputText
          v-model="search"
          class="embedded-input w-full border-0 !bg-transparent !py-2 !text-sm"
          placeholder="Search icons... (e.g. home, user, chart)"
          autofocus
        />
      </span>

      <p class="text-xs text-muted">{{ filtered.length }} icons</p>

      <div class="max-h-[400px] overflow-auto rounded-md border border-divider bg-canvas p-2">
        <div v-if="filtered.length === 0" class="py-8 text-center text-sm text-muted">No icons match "{{ search }}"</div>
        <div v-else class="grid grid-cols-8 gap-1">
          <button
            v-for="icon in filtered"
            :key="icon"
            class="group flex flex-col items-center gap-1 rounded-md p-2 transition-colors hover:bg-brand/10"
            :class="props.selected === icon || props.selected === 'pi ' + icon ? 'bg-brand/15 ring-2 ring-brand/30' : ''"
            :title="icon"
            @click="pick(icon)"
          >
            <i :class="['pi', icon, 'text-lg transition-colors group-hover:text-brand', props.selected === icon || props.selected === 'pi ' + icon ? 'text-brand' : 'text-ink']" />
            <span class="w-full truncate text-center text-[9px] text-muted">{{ icon.replace('pi-', '') }}</span>
          </button>
        </div>
      </div>
    </div>
  </Dialog>
</template>
