import type { App } from "vue";
import { createPinia } from "pinia";
import PrimeVue from "primevue/config";
import { definePreset } from "@primevue/themes";
import Aura from "@primevue/themes/aura";
import Tooltip from "primevue/tooltip";

// Thu nhỏ nút PrimeVue cỡ mặc định trong app — chỉnh riêng token của component
// `button`, không đụng `formField` nên input/select không đổi. Nút size="small"
// giữ nguyên giá trị gốc của Aura (không override `sm`).
const CompactAura = definePreset(Aura, {
  components: {
    button: {
      root: {
        paddingX: "0.6292rem",
        paddingY: "0.5333rem",
        gap: "0.2rem",
        iconOnlyWidth: "2.0667rem",
      },
    },
  },
});

export function registerPlugins(app: App) {
  app.use(createPinia());
  app.use(PrimeVue, {
    theme: {
      preset: CompactAura,
      options: {
        darkModeSelector: "[data-theme='dark']",
      },
    },
    ripple: true,
  });
  app.directive("tooltip", Tooltip);
}
